use crate::{
    cache::SlotCache,
    metrics::Metrics,
    rpc::SolanaRpc,
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Runs the background poller task to continuously update the slot cache.
pub async fn run_cache_poller(
    cache: Arc<SlotCache>,
    rpc: Arc<dyn SolanaRpc>,
    metrics: Arc<dyn Metrics>,
    interval: Duration,
) {
    info!("Cache poller started.");

    let mut last_fetched_slot = match rpc.get_slot().await {
        Ok(slot) => {
            info!("Initial slot fetched: {}", slot);
            // Starts 10 slots in the past to ensure we have a good baseline 
            slot.saturating_sub(10)
        }
        Err(e) => {
            error!("Failed to get initial slot: {}. Retrying in 5s.", e);
            sleep(Duration::from_secs(5)).await;
            // Exit if we can't even get the first slot.
            // A real-world app might retry indefinitely.
            return; 
        }
    };

    loop {
        sleep(interval).await;

        let get_blocks_start = std::time::Instant::now();
        
        let current_latest_slot = match rpc.get_slot().await {
            Ok(slot) => slot,
            Err(e) => {
                warn!("Poller: Failed to get latest slot: {}. Skipping this poll.", e);
                continue; 
            }
        };

        if current_latest_slot <= last_fetched_slot {
            continue;
        }

        let start_slot = last_fetched_slot + 1;
        let end_slot = current_latest_slot;

        match rpc.get_blocks(start_slot, end_slot).await {
            Ok(new_slots) => {
                let elapsed = get_blocks_start.elapsed();
                metrics.record_get_blocks_elapsed(elapsed);
                metrics.record_latest_slot(end_slot);
                
                info!("Poller: Fetched {} new confirmed slots (range {}-{}).", new_slots.len(), start_slot, end_slot);
                
                cache.add_slots(new_slots).await;
 
                last_fetched_slot = end_slot;
            }
            Err(e) => {
                warn!("Poller: Failed to get blocks ({}-{}): {}. Retrying on next poll.", start_slot, end_slot, e);
                // We don't update `last_fetched_slot`, so we'll retry this range
            }
        }
    }
}