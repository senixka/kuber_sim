use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use std::option::Option;
use crate::my_imports::dsc;
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

    #[serde(default)]
    pub kubelet_sim_id: Option<dsc::Id>
}


impl Node {
    pub fn from_yaml(path: &str) -> Self {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        assert_eq!(s.starts_with("kind: Node"), true);

        let mut node: Node = serde_yaml::from_str(s.as_str()).unwrap();
        node.metadata.uid = UID_COUNTER.load(Ordering::SeqCst);
        UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        node.kubelet_sim_id = None;

        node.spec.available_cpu = node.spec.installed_cpu;
        node.spec.available_memory = node.spec.installed_memory;

        return node;
    }
}
