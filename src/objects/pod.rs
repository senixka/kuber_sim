use crate::my_imports::*;


// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podspec-v1-core
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PodSpec {
    pub arrival_time: f64,
    pub load: LoadType,

    pub request_cpu: u64,
    pub request_memory: u64,

    #[serde(default)]
    pub limit_cpu: u64,
    #[serde(default)]
    pub limit_memory: u64,

    #[serde(default)]
    pub priority: i64,

    #[serde(default)]
    pub node_selector: BTreeMap<String, String>,
}

impl Eq for PodSpec {}


// https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PodPhase {
    #[default]
    Pending = 0,
    Running = 1,
    Succeeded = 2,
    Failed = 3,
    Unknown = 4,
}


// https://kubernetes.io/docs/concepts/workloads/pods/pod-qos/#quality-of-service-classes
#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum QoSClass {
    #[default]
    BestEffort = 0,
    Burstable = 1,
    Guaranteed = 2,
}


// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podstatus-v1-core
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PodStatus {
    #[serde(skip)]
    pub phase: PodPhase,

    #[serde(skip)]
    pub node_uid: Option<u64>,

    #[serde(skip)]
    pub qos_class: QoSClass,
}


// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#pod-v1-core
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pod {
    pub spec: PodSpec,

    #[serde(default)]
    pub metadata: ObjectMeta,
    #[serde(skip)]
    pub status: PodStatus,
}

impl Pod {
    pub fn prepare(&mut self, group_uid: u64) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.metadata.uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        self.metadata.group_uid = group_uid;
        assert_ne!(group_uid, 0);

        self.status.phase = PodPhase::Pending;
        self.status.node_uid = None;

        if self.spec.limit_cpu == 0 {
            self.spec.limit_cpu = u64::MAX;
        }
        if self.spec.limit_memory == 0 {
            self.spec.limit_memory = u64::MAX;
        }

        if self.spec.request_cpu == self.spec.limit_cpu
            && self.spec.request_memory == self.spec.limit_memory {
            self.status.qos_class = QoSClass::Guaranteed;
        } else if self.spec.limit_cpu < u64::MAX && self.spec.limit_memory < u64::MAX {
            self.status.qos_class = QoSClass::Burstable;
        } else {
            self.status.qos_class = QoSClass::BestEffort;
        }

        assert!(self.spec.limit_cpu >= self.spec.request_cpu);
        assert!(self.spec.limit_memory >= self.spec.request_memory);
        assert!(self.spec.load.get_duration() > 0.0);
    }
}
