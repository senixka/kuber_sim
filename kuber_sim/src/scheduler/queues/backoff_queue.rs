use crate::my_imports::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IBackOffQ {
    fn push(&mut self, pod_uid: u64, backoff_attempts: u64, current_time: f64);
    fn try_pop(&mut self, current_time: f64) -> Option<u64>;
    fn try_remove(&mut self, pod_uid: u64) -> bool;
    fn len(&self) -> usize;
    fn clone(&self) -> Box<dyn IBackOffQ + Send>;
}

pub type BackOffQDefault = BackOffQExponential;

///////////////////////////////////////// Item Wrapper /////////////////////////////////////////////

#[derive(Debug)]
struct ItemWrapper {
    pub pod_uid: u64,
    pub exit_time: f64,
}

impl PartialOrd for ItemWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.exit_time
            .total_cmp(&other.exit_time)
            .then(self.pod_uid.cmp(&other.pod_uid))
    }
}

impl PartialEq for ItemWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.pod_uid == other.pod_uid
    }
}

impl Eq for ItemWrapper {}

////////////////////////////////////// BackOff Exponential /////////////////////////////////////////

pub struct BackOffQExponential {
    initial_backoff: f64,
    max_backoff: f64,
    exit_time: HashMap<u64, f64>,
    queue: BTreeSet<ItemWrapper>,
}

impl BackOffQExponential {
    pub fn new(initial_backoff: f64, max_backoff: f64) -> Self {
        Self {
            initial_backoff,
            max_backoff,
            queue: BTreeSet::new(),
            exit_time: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        BackOffQExponential::new(1.0, 10.0)
    }
}

impl IBackOffQ for BackOffQExponential {
    fn push(&mut self, pod_uid: u64, backoff_attempts: u64, current_time: f64) {
        let unlimited_timeout = self.initial_backoff * 2.0f64.powf(backoff_attempts as f64);
        let backoff_timeout = self.max_backoff.min(unlimited_timeout);

        self.queue.insert(ItemWrapper {
            pod_uid,
            exit_time: current_time + backoff_timeout,
        });
        self.exit_time.insert(pod_uid, current_time + backoff_timeout);
    }

    fn try_pop(&mut self, current_time: f64) -> Option<u64> {
        let top = self.queue.first();
        if top.is_none() || top.unwrap().exit_time > current_time {
            return None;
        }

        let pod_uid = self.queue.pop_first().unwrap().pod_uid;
        self.exit_time.remove(&pod_uid);
        return Some(pod_uid);
    }

    fn try_remove(&mut self, pod_uid: u64) -> bool {
        return match self.exit_time.get(&pod_uid) {
            Some(&exit_time) => {
                self.exit_time.remove(&pod_uid);

                let _was_present = self.queue.remove(&ItemWrapper { pod_uid, exit_time });
                assert_eq!(_was_present, true);

                true
            }
            None => false,
        };
    }

    fn len(&self) -> usize {
        return self.queue.len();
    }

    fn clone(&self) -> Box<dyn IBackOffQ + Send> {
        return Box::new(BackOffQExponential::new(self.initial_backoff, self.max_backoff));
    }
}

//////////////////////////////////////// BackOff Constant //////////////////////////////////////////

pub struct BackOffQConstant {
    backoff_delay: f64,
    exit_time: HashMap<u64, f64>,
    queue: BTreeSet<ItemWrapper>,
}

impl BackOffQConstant {
    pub fn new(backoff_delay: f64) -> Self {
        Self {
            backoff_delay,
            queue: BTreeSet::new(),
            exit_time: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        BackOffQConstant::new(30.0)
    }
}

impl IBackOffQ for BackOffQConstant {
    fn push(&mut self, pod_uid: u64, _: u64, current_time: f64) {
        self.queue.insert(ItemWrapper {
            pod_uid,
            exit_time: current_time + self.backoff_delay,
        });
        self.exit_time.insert(pod_uid, current_time + self.backoff_delay);
    }

    fn try_pop(&mut self, current_time: f64) -> Option<u64> {
        let top = self.queue.first();
        if top.is_none() || top.unwrap().exit_time > current_time {
            return None;
        }

        let pod_uid = self.queue.pop_first().unwrap().pod_uid;
        self.exit_time.remove(&pod_uid);
        return Some(pod_uid);
    }

    fn try_remove(&mut self, pod_uid: u64) -> bool {
        return match self.exit_time.get(&pod_uid) {
            Some(&exit_time) => {
                self.exit_time.remove(&pod_uid);

                let _was_present = self.queue.remove(&ItemWrapper { pod_uid, exit_time });
                assert_eq!(_was_present, true);

                true
            }
            None => false,
        };
    }

    fn len(&self) -> usize {
        return self.queue.len();
    }

    fn clone(&self) -> Box<dyn IBackOffQ + Send> {
        return Box::new(BackOffQConstant::new(self.backoff_delay));
    }
}

///////////////////////////////////////////// Test /////////////////////////////////////////////////

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_backoff_queue_many_times() {
        for _ in 0..10 {
            test_backoff_queue_constant();
            test_backoff_queue_exponential();
        }
    }

    #[test]
    fn test_backoff_queue_constant() {
        let mut q = BackOffQConstant::new(30.0);
        q.push(1, 0, 0.0);

        assert_eq!(q.try_pop(29.95), None);
        assert_eq!(q.try_pop(30.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(1, 1000, 1.0);

        assert_eq!(q.try_pop(30.95), None);
        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(1, 123, 5.0);

        assert_eq!(q.try_pop(34.95), None);
        assert_eq!(q.try_pop(35.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(1, 3, 1.0);
        q.push(2, 2, 2.0);
        q.push(3, 2, 1.0);

        assert_eq!(q.try_pop(30.95), None);
        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(31.05), Some(3));
        assert_eq!(q.try_pop(31.05), None);
        assert_eq!(q.try_pop(32.05), Some(2));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_pop(30.95), None);
        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(31.05), Some(2));
        assert_eq!(q.try_pop(31.05), Some(3));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_remove(1), true);
        assert_eq!(q.try_remove(1), false);
        assert_eq!(q.try_pop(31.05), Some(2));
        assert_eq!(q.try_remove(2), false);
        assert_eq!(q.try_remove(3), true);
        assert_eq!(q.try_pop(10000.0), None);
        assert_eq!(q.len(), 0);

        // Insert many pods with uids in [1..1000] and current_time in [0..10]
        let mut rng = rand::thread_rng();
        for i in 1..12000 {
            assert_eq!(q.len(), i as usize - 1);
            q.push(i, rng.gen_range(0..=10000), rng.gen_range(0..=10) as f64);
        }

        // Remove random pods
        let len_before = q.len();
        let mut rm_cnt = 0;
        for _ in 0..1000 {
            rm_cnt += q.try_remove(rng.gen_range(1..=1000)) as usize;
        }
        assert_eq!(q.len() + rm_cnt, len_before);

        // Pop remaining pods
        assert_eq!(q.try_pop(29.95), None);
        let mut uids: HashSet<u64> = HashSet::new();
        for time_shift in 0..=10 {
            let mut last_uid = 0;
            while let Some(pod_uid) = q.try_pop(time_shift as f64 + 30.05) {
                let newly_inserted = uids.insert(pod_uid);
                assert!(newly_inserted);

                assert!(pod_uid > last_uid);
                last_uid = pod_uid;
            }
        }
        assert_eq!(q.try_pop(10000000.0), None);
        assert_eq!(q.len(), 0);
    }

    #[test]
    pub fn test_backoff_queue_exponential() {
        let mut q = BackOffQExponential::new(1.0, 10.0);
        q.push(1, 0, 0.0);

        assert_eq!(q.try_pop(0.95), None);
        assert_eq!(q.try_pop(1.05), Some(1));
        assert_eq!(q.try_pop(1.05), None);

        q.push(1, 1, 1.0);

        assert_eq!(q.try_pop(2.95), None);
        assert_eq!(q.try_pop(3.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(1, 4, 1.0);

        assert_eq!(q.try_pop(10.95), None);
        assert_eq!(q.try_pop(11.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(1, 3, 1.0);
        q.push(2, 1, 1.0);
        q.push(3, 2, 1.0);

        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_pop(9.05), Some(3));
        assert_eq!(q.try_pop(9.05), Some(1));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_pop(9.05), Some(1));
        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_pop(9.05), Some(3));
        assert_eq!(q.try_pop(10000.0), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_remove(1), true);
        assert_eq!(q.try_remove(1), false);
        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_remove(2), false);
        assert_eq!(q.try_remove(3), true);
        assert_eq!(q.try_pop(10000.0), None);
    }
}
