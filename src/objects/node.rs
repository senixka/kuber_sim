use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use crate::my_imports::Pod;

use crate::ObjectMeta;


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSpec {
    pub installed_cpu: u64,    // in milli-CPU (1000 milli-CPU = 1 CPU = 1 vCPU)
    pub installed_memory: u64, // in bytes

    #[serde(skip_deserializing)]
    pub available_cpu: u64,
    #[serde(skip_deserializing)]
    pub available_memory: u64,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeStatus {
    pub pods: HashSet<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

    pub fn is_consumable(&self, cpu: u64, memory: u64) -> bool {
        return self.spec.available_cpu >= cpu && self.spec.available_memory >= memory;
    }

    pub fn consume(&mut self, cpu: u64, memory: u64) {
        assert!(self.spec.available_cpu >= cpu);
        assert!(self.spec.available_memory >= memory);
        self.spec.available_cpu -= cpu;
        self.spec.available_memory -= memory;
    }

    pub fn restore(&mut self, cpu: u64, memory: u64) {
        self.spec.available_cpu += cpu;
        self.spec.available_memory += memory;
        assert!(self.spec.available_cpu <= self.spec.installed_cpu);
        assert!(self.spec.available_memory <= self.spec.installed_memory);
    }
}
