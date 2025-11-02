use std::time::Duration;
use tracing::info;


#[mockall::automock]
pub trait Metrics: Send + Sync {
    fn record_latest_slot(&self, slot: u64);

    fn record_get_blocks_elapsed(&self, elapsed: Duration);

    fn record_is_slot_confirmed_elapsed(&self, elapsed: Duration);
}


pub struct AppMetrics;


impl Metrics for AppMetrics {
    fn record_latest_slot(&self, slot: u64) {
        info!(target: "metrics", latest_slot = slot, "Poller: latest slot");
    }

    fn record_get_blocks_elapsed(&self, elapsed: Duration) {
        info!(
            target: "metrics",
            elapsed_ms = elapsed.as_millis(),
            "Poller: get_blocks duration"
        );
    }

    fn record_is_slot_confirmed_elapsed(&self, elapsed: Duration) {
        info!(
            target: "metrics",
            elapsed_us = elapsed.as_micros(),
            "Handler: is_slot_confirmed duration"
        );
    }
}