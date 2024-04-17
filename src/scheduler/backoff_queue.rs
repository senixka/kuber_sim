use std::collections::{BinaryHeap};
use std::cmp::Ordering;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait TraitBackOffQ {
    fn new(initial_backoff: f64, max_backoff: f64) -> Self;
    fn push(&mut self, pod_uid: u64, failed_attempts: u64, current_time: f64);
    fn try_pop(&mut self, current_time: f64) -> Option<u64>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct BackOffQExponential {
    initial_backoff: f64,
    max_backoff: f64,
    queue: BinaryHeap<ItemWrapper>,
}

impl TraitBackOffQ for BackOffQExponential {
    #[inline]
    fn new(initial_backoff: f64, max_backoff: f64) -> Self {
        Self {
            initial_backoff,
            max_backoff,
            queue: BinaryHeap::new(),
        }
    }

    #[inline]
    fn push(&mut self, pod_uid: u64, failed_attempts: u64, current_time: f64) {
        let unlimited_timeout = self.initial_backoff * 2.0f64.powf(failed_attempts as f64);
        let backoff_timeout = self.max_backoff.min(unlimited_timeout);
        self.queue.push(ItemWrapper {
            pod_uid,
            exit_time: current_time + backoff_timeout
        });
    }

    #[inline]
    fn try_pop(&mut self, current_time: f64) -> Option<u64> {
        let top = self.queue.peek();
        if top.is_none() || top.unwrap().exit_time > current_time {
            return None;
        }

        return Some(self.queue.pop().unwrap().pod_uid);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ItemWrapper {
    pub pod_uid: u64,
    pub exit_time: f64,
}

impl PartialOrd for ItemWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        other.exit_time.total_cmp(&self.exit_time)
    }
}

impl PartialEq for ItemWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.exit_time == other.exit_time
    }
}

impl Eq for ItemWrapper {}

////////////////////////////////////////////////////////////////////////////////////////////////////