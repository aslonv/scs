use crate::{
    cache::SlotCache,
    error::AppError,
    metrics::Metrics,
    rpc::SolanaRpc,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::sync::Arc; 
use tracing::{info, warn};


#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<SlotCache>,
    pub rpc: Arc<dyn SolanaRpc>,
    pub metrics: Arc<dyn Metrics>,
}


pub async fn run_server(state: AppState, addr: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/isSlotConfirmed/:slot", get(is_slot_confirmed))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}


async fn is_slot_confirmed(
    State(state): State<AppState>,
    Path(slot): Path<u64>,
) -> Result<StatusCode, AppError> {
    info!("Request: /isSlotConfirmed/{}", slot);
    let start_time = std::time::Instant::now();

    let result = async {
        if state.cache.contains(&slot).await {
            info!("Cache HIT for slot {}", slot);
            return Ok(StatusCode::OK);
        }
        info!("Cache MISS for slot {}", slot);

        match state.rpc.get_blocks(slot, slot).await {
            Ok(slots) => {
                if !slots.is_empty() {
                    info!("RPC HIT for slot {}", slot);
                    Ok(StatusCode::OK)
                } else {
                    info!("RPC MISS for slot {}", slot);
                    Ok(StatusCode::NOT_FOUND)
                }
            }
            Err(e) => {
                warn!("RPC error checking slot {}: {}", slot, e);
                Err(AppError::Rpc(e))
            }
        }
    }
    .await;

    state
        .metrics
        .record_is_slot_confirmed_elapsed(start_time.elapsed());

    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::boxed::Box;
    use crate::{
        metrics::{MockMetrics, Metrics},
        rpc::{MockSolanaRpc, SolanaRpc},
    };
    use axum::http::StatusCode;
    use mockall::predicate::eq;
    use solana_client::client_error::ClientError;

    fn create_test_state(
        cache: Arc<SlotCache>,
        rpc: Arc<dyn SolanaRpc>,
        metrics: Arc<dyn Metrics>,
    ) -> AppState {
        AppState { cache, rpc, metrics }
    }

    #[tokio::test]
    async fn test_slot_in_cache_hit() {
        let cache = Arc::new(SlotCache::new(10));
        cache.add_slots(vec![12345]).await;

        let mut rpc = MockSolanaRpc::new();
        rpc.expect_get_blocks().times(0);

        let mut metrics = MockMetrics::new();
        metrics
            .expect_record_is_slot_confirmed_elapsed()
            .times(1)
            .return_const(());
        let state = create_test_state(cache, Arc::new(rpc), Arc::new(metrics));

        let result = is_slot_confirmed(State(state), Path(12345)).await;
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_slot_not_in_cache_rpc_hit() {
        let cache = Arc::new(SlotCache::new(10));

        let mut rpc = MockSolanaRpc::new();
        rpc.expect_get_blocks()
            .with(eq(54321), eq(54321))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(vec![54321]) })); // <--- FIX: Box::pin()

        let mut metrics = MockMetrics::new();
        metrics
            .expect_record_is_slot_confirmed_elapsed()
            .times(1)
            .return_const(());
        let state = create_test_state(cache, Arc::new(rpc), Arc::new(metrics));

        let result = is_slot_confirmed(State(state), Path(54321)).await;
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_slot_not_in_cache_rpc_miss() {
        let cache = Arc::new(SlotCache::new(10));

        let mut rpc = MockSolanaRpc::new();
        rpc.expect_get_blocks()
            .with(eq(99999), eq(99999))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(vec![]) })); // <--- FIX: Box::pin()

        let mut metrics = MockMetrics::new();
        metrics
            .expect_record_is_slot_confirmed_elapsed()
            .times(1)
            .return_const(());
        let state = create_test_state(cache, Arc::new(rpc), Arc::new(metrics));

        let result = is_slot_confirmed(State(state), Path(99999)).await;
        assert_eq!(result.unwrap(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_slot_rpc_error() {
        let cache = Arc::new(SlotCache::new(10));

        let mut rpc = MockSolanaRpc::new();
        rpc.expect_get_blocks()
            .with(eq(11111), eq(11111))
            .times(1)
            .returning(|_, _| Box::pin(async { 
                Err(ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "timeout",
                )))
            }));

        let mut metrics = MockMetrics::new();
        metrics
            .expect_record_is_slot_confirmed_elapsed()
            .times(1)
            .return_const(());
        let state = create_test_state(cache, Arc::new(rpc), Arc::new(metrics));

        let result = is_slot_confirmed(State(state), Path(11111)).await;
        assert!(result.is_err());
        use axum::response::IntoResponse;
        assert_eq!(
            result.unwrap_err().into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}