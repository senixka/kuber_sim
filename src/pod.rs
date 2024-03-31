use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use crate::{types};

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podspec-v1-core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpec {
    pub arrival_time: types::Time,
    pub load_profile: Vec<Spike>,
    pub request_cpu: u64,
    pub request_memory: u64,

    #[serde(default)]
    pub limit_cpu: u64,
    #[serde(default)]
    pub limit_memory: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spike {
    pub cpu: u64,
    pub memory: u64,
    pub duration: types::Time,
}

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
    pub fn from_yaml(path: &str) -> Self {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        assert_eq!(s.starts_with("kind: Pod"), true);

        let mut pod: Pod = serde_yaml::from_str(s.as_str()).unwrap();
        pod.metadata.uid = UID_COUNTER.load(Ordering::SeqCst);
        UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        pod.status.phase = PodPhase::Pending;
        pod.status.node_uid = None;

        if pod.spec.limit_cpu == 0 {
            pod.spec.limit_cpu = u64::MAX;
        }
        if pod.spec.limit_memory == 0 {
            pod.spec.limit_memory = u64::MAX;
        }

        assert_eq!((pod.spec.limit_cpu >= pod.spec.request_cpu), true);
        assert_eq!((pod.spec.limit_memory >= pod.spec.request_memory), true);

        return pod;
    }
}
