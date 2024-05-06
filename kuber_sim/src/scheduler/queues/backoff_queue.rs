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

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

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
