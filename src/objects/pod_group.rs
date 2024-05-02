use crate::my_imports::*;


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PodGroup {
    pub submit_time: f64,
    pub pod_count: u64,
    pub pod: Pod,

    // #[serde(default)]
    // pub group_duration: f64,

    #[serde(skip)]
    pub group_uid: u64,
}


impl PodGroup {
    pub fn prepare(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.group_uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}
