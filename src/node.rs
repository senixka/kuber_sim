use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use crate::pod::{ObjectMeta, Pod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    pub installed_cpu: u64,    // in milli-CPU (1000 milli-CPU = 1 CPU = 1 vCPU)
    pub installed_memory: u64, // in bytes

    #[serde(skip_deserializing)]
    pub available_cpu: u64,
    #[serde(skip_deserializing)]
    pub available_memory: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeStatus {
    pub pods: Vec<Pod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub spec: NodeSpec,

    #[serde(default)]
    pub metadata: ObjectMeta,
    #[serde(default)]
    pub status: NodeStatus,
}


impl Node {
    pub fn init(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);
        self.metadata.uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        self.spec.available_cpu = self.spec.installed_cpu;
        self.spec.available_memory = self.spec.installed_memory;
    }
}
