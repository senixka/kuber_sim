use crate::my_imports::*;


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PodGroup {
    #[serde(skip)]
    pub group_uid: u64,

    pub amount: u64,
    pub pod: Pod,
}


impl PodGroup {
    pub fn prepare(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.group_uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}
