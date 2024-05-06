use crate::my_imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    #[serde(skip)]
    pub group_uid: u64,

    pub amount: u64,
    pub node: Node,
}

impl NodeGroup {
    pub fn prepare(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.group_uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}
