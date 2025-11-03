use solana_caching_service::{
    cache::SlotCache,
    http::{run_server, AppState},
    metrics::AppMetrics, 
    poller::run_cache_poller,
    rpc::RpcClientWrapper,
};
use std::{sync::Arc, time::Duration};
use tracing::info;


const API_KEY: &str = "2msa5HeufC6XcfHpRk8xZiiNV7EKKTN689KVVgorwUiYrEPMMbv6oJe3Pfx9vzew1aavyNicMFThPVh9asyhW8bggdxdgKJUd4A";
const RPC_ENDPOINT: &str = "https://solana-mainnet.api.syndica.io/api-key/";
const CACHE_CAPACITY: usize = 1000;
const POLLER_INTERVAL: Duration = Duration::from_secs(2);
const SERVER_ADDR: &str = "0.0.0.0:3000";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("Starting Solana Caching Web Service...");

    let full_rpc_url = format!("{}{}", RPC_ENDPOINT, API_KEY);
    let rpc_client = Arc::new(RpcClientWrapper::new(full_rpc_url));
    let cache = Arc::new(SlotCache::new(CACHE_CAPACITY));
    let metrics = Arc::new(AppMetrics);
    let app_state = AppState {
        cache: cache.clone(),
        rpc: rpc_client.clone(),
        metrics: metrics.clone(),
    };

    tokio::spawn(run_cache_poller(
        cache,
        rpc_client,
        metrics,
        POLLER_INTERVAL,
    ));

    info!("Listening on {}", SERVER_ADDR);
    run_server(app_state, SERVER_ADDR).await?;

    Ok(())
}
