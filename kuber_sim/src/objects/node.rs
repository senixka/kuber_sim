use crate::my_imports::*;

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSpec {
    pub installed_cpu: i64,    // in milli-CPU (1000 milli-CPU = 1 CPU = 1 vCPU)
    pub installed_memory: i64, // in bytes

    #[serde(skip)]
    pub available_cpu: i64,
    #[serde(skip)]
    pub available_memory: i64,

    #[serde(default)]
    pub taints: Vec<Taint>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeStatus {
    pub pods: HashSet<u64>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub spec: NodeSpec,

    #[serde(default)]
    pub metadata: ObjectMeta,
    #[serde(skip)]
    pub status: NodeStatus,
}

impl Node {
    pub fn prepare(&mut self, group_uid: u64) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);
        self.metadata.uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        self.spec.available_cpu = self.spec.installed_cpu;
        self.spec.available_memory = self.spec.installed_memory;

        self.metadata.group_uid = group_uid;

        assert!(self.spec.installed_cpu > 0);
        assert!(self.spec.installed_memory > 0);
    }

    pub fn is_both_consumable(&self, cpu: i64, memory: i64) -> bool {
        return self.spec.available_cpu >= cpu && self.spec.available_memory >= memory;
    }

    pub fn is_memory_consumable(&self, memory: i64) -> bool {
        return self.spec.available_memory >= memory;
    }

    pub fn consume(&mut self, cpu: i64, memory: i64) {
        self.spec.available_cpu -= cpu;
        self.spec.available_memory -= memory;
    }

    pub fn restore(&mut self, cpu: i64, memory: i64) {
        self.spec.available_cpu += cpu;
        self.spec.available_memory += memory;
    }
}
