use std::collections::{HashSet, VecDeque};
use tokio::sync::RwLock;
use tracing::debug;


pub struct SlotCache {
    capacity: usize,
    slots: RwLock<HashSet<u64>>,
    order: RwLock<VecDeque<u64>>,
}


impl SlotCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            slots: RwLock::new(HashSet::with_capacity(capacity)),
            order: RwLock::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub async fn contains(&self, slot: &u64) -> bool {
        self.slots.read().await.contains(slot)
    }

    pub async fn add_slots(&self, new_slots: Vec<u64>) {
        if new_slots.is_empty() {
            return;
        }

        let mut slots = self.slots.write().await;
        let mut order = self.order.write().await;

        for slot in new_slots {
            if slots.insert(slot) {
                order.push_back(slot);
            }
        }

        // Evict old slots if we're over capacity 
        while order.len() > self.capacity {
            if let Some(oldest_slot) = order.pop_front() {
                slots.remove(&oldest_slot);
            }
        }
        debug!("Cache updated. New size: {}", slots.len());
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_add_and_contains() {
        let cache = SlotCache::new(3);
        cache.add_slots(vec![10, 20]).await;

        assert!(cache.contains(&10).await);
        assert!(cache.contains(&20).await);
        assert!(!cache.contains(&30).await);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = SlotCache::new(3);
        cache.add_slots(vec![10, 20, 30]).await;

        assert!(cache.contains(&10).await);
        assert!(cache.contains(&20).await);
        assert!(cache.contains(&30).await);

        // Add a new slot, which should evict the oldest 
        cache.add_slots(vec![40]).await;

        assert!(!cache.contains(&10).await); // Evicted
        assert!(cache.contains(&20).await);
        assert!(cache.contains(&30).await);
        assert!(cache.contains(&40).await); // Added
    }

    #[tokio::test]
    async fn test_cache_add_duplicates() {
        let cache = SlotCache::new(3);
        cache.add_slots(vec![10, 20]).await;
        cache.add_slots(vec![10, 20, 30]).await;

        assert!(cache.contains(&10).await);
        assert!(cache.contains(&20).await);
        assert!(cache.contains(&30).await);

        cache.add_slots(vec![40]).await;
        assert!(!cache.contains(&10).await);
        assert!(cache.contains(&20).await);
    }
}