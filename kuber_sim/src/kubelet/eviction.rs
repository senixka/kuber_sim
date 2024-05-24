use crate::objects::pod::{Pod, QoSClass};
use std::collections::BTreeSet;

pub struct EvictionOrder {
    // (BestEffort) or (Burstable with usage > requests) pods
    pub primary: BTreeSet<(i64, i64, u64)>, // BTreeSet<(priority, memory_request - memory_usage, u64::MAX - pod_uid)>
    // (Guaranteed) or (Burstable with usage <= requests) pods
    pub secondary: BTreeSet<(i64, u64)>, // BTreeSet<(priority, u64::MAX - pod_uid)>
}

impl EvictionOrder {
    pub fn new() -> Self {
        Self {
            primary: BTreeSet::new(),
            secondary: BTreeSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        return self.primary.len() + self.secondary.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.primary.len() + self.secondary.len() == 0;
    }

    pub fn first(&self) -> Option<u64> {
        return if self.primary.is_empty() {
            match self.secondary.first() {
                Some(&(_, pod_uid)) => Some(u64::MAX - pod_uid),
                None => None,
            }
        } else {
            Some(u64::MAX - self.primary.first().unwrap().2)
        };
    }

    pub fn add(&mut self, pod: &Pod, used_memory: i64) {
        let (mut newly_inserted_1, mut newly_inserted_2) = (false, false);

        if pod.status.qos_class == QoSClass::BestEffort || pod.spec.request_memory - used_memory < 0 {
            newly_inserted_1 = self.primary.insert((
                pod.spec.priority,
                pod.spec.request_memory - used_memory,
                u64::MAX - pod.metadata.uid,
            ));
        } else {
            newly_inserted_2 = self.secondary.insert((pod.spec.priority, u64::MAX - pod.metadata.uid));
        }

        assert!(newly_inserted_1 ^ newly_inserted_2);
    }

    pub fn remove(&mut self, pod: &Pod, used_memory: i64) {
        let (mut was_present_1, mut was_present_2) = (false, false);

        if pod.status.qos_class == QoSClass::BestEffort || pod.spec.request_memory - used_memory < 0 {
            was_present_1 = self.primary.remove(&(
                pod.spec.priority,
                pod.spec.request_memory - used_memory,
                u64::MAX - pod.metadata.uid,
            ));
        } else {
            was_present_2 = self.secondary.remove(&(pod.spec.priority, u64::MAX - pod.metadata.uid));
        }

        assert!(was_present_1 ^ was_present_2);
    }

    pub fn clear(&mut self) {
        self.primary.clear();
        self.secondary.clear();
    }
}
