use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadType {
    // fn start(&mut self, current_time: f64) -> (u64, u64, bool);
    // fn update(&mut self, current_time: f64) -> (u64, u64, bool);

    Constant(Constant),
    BusyBox(BusyBox),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    #[serde(skip_deserializing)]
    pub start_time: f64,
    #[serde(skip_deserializing)]
    pub next_spike_time: f64,

    pub duration: f64,
    pub cpu: u64,
    pub memory: u64,
}

impl Constant {
    fn start(&mut self, current_time: f64) -> (u64, u64, bool) {
        self.start_time = current_time;
        self.next_spike_time = current_time + self.duration;
        return (self.cpu, self.memory, self.duration == 0.0);
    }

    fn update(&mut self, current_time: f64) -> (u64, u64, bool) {
        return (self.cpu, self.memory, current_time - self.start_time >= self.duration);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusyBox {
    #[serde(skip_deserializing)]
    pub start_time: f64,
    #[serde(skip_deserializing)]
    pub next_spike_time: f64,

    pub duration: f64,
    pub shift_time: f64,
    pub cpu_down: u64,
    pub memory_down: u64,
    pub cpu_up: u64,
    pub memory_up: u64,
}

impl BusyBox {
    fn start(&mut self, current_time: f64) -> (u64, u64, bool) {
        self.start_time = current_time;
        self.next_spike_time = current_time + self.shift_time;
        return (self.cpu_down, self.memory_down, self.duration == 0.0);
    }

    fn update(&mut self, current_time: f64) -> (u64, u64, bool) {
        let epoch: u64 = ((current_time - self.start_time) / self.shift_time) as u64;
        self.next_spike_time = (epoch as f64 + 1.0) * self.shift_time;

        if epoch % 2 == 0 {
            return (self.cpu_down, self.memory_down, current_time - self.start_time >= self.duration);
        }
        return (self.cpu_up, self.memory_up, current_time - self.start_time >= self.duration);
    }
}

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podspec-v1-core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpec {
    pub arrival_time: f64,
    pub load_profile: Vec<LoadType>,
    pub request_cpu: u64,
    pub request_memory: u64,

    #[serde(default)]
    pub limit_cpu: u64,
    #[serde(default)]
    pub limit_memory: u64,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Spike {
//     pub cpu: u64,
//     pub memory: u64,
//     pub duration: f64,
// }

// https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum PodPhase {
    #[default]
    Pending = 0,
    Running = 1,
    Succeeded = 2,
    Failed = 3,
    Unknown = 4,
}

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podstatus-v1-core
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PodStatus {
    #[serde(default)]
    pub phase: PodPhase,

    #[serde(default)]
    pub node_uid: Option<u64>
}

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#objectmeta-v1-meta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub labels: HashMap<String, String>,

    #[serde(skip_deserializing)]
    pub uid: u64,
}

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#pod-v1-core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub spec: PodSpec,

    #[serde(default)]
    pub metadata: ObjectMeta,
    #[serde(default)]
    pub status: PodStatus,
}

impl Pod {
    pub fn init(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.metadata.uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        self.status.phase = PodPhase::Pending;
        self.status.node_uid = None;

        if self.spec.limit_cpu == 0 {
            self.spec.limit_cpu = u64::MAX;
        }
        if self.spec.limit_memory == 0 {
            self.spec.limit_memory = u64::MAX;
        }

        assert!(self.spec.limit_cpu >= self.spec.request_cpu);
        assert!(self.spec.limit_memory >= self.spec.request_memory);
    }
}
