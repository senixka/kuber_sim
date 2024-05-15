use crate::my_imports::*;

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podspec-v1-core
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PodSpec {
    #[serde(default)]
    pub request_cpu: i64,
    #[serde(default)]
    pub request_memory: i64,

    #[serde(default)]
    pub limit_cpu: i64,
    #[serde(default)]
    pub limit_memory: i64,

    #[serde(default)]
    pub priority: i64,

    pub load: LoadType,

    #[serde(default)]
    pub node_selector: BTreeMap<String, String>,
    #[serde(default)]
    pub tolerations: Vec<Toleration>,
    #[serde(default)]
    pub node_affinity: NodeAffinity,
}

impl FromStr for PodSpec {
    type Err = ();

    /// Expects "<i64>;<i64>;<i64>;<i64>;<i64>;{<LoadType>};{<node_selector>};{<tolerations>};{<NodeAffinity>}"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (request_cpu_str, other) = s.split_once(';').unwrap();
        let (request_memory_str, other) = other.split_once(';').unwrap();
        let (limit_cpu_str, other) = other.split_once(';').unwrap();
        let (limit_memory_str, other) = other.split_once(';').unwrap();
        let (priority_str, other) = other.split_once(';').unwrap();

        let mut request_cpu = 0;
        if !request_cpu_str.is_empty() {
            request_cpu = str::parse(request_cpu_str).unwrap();
        }

        let mut request_memory = 0;
        if !request_memory_str.is_empty() {
            request_memory = str::parse(request_memory_str).unwrap();
        }

        let mut limit_cpu = 0;
        if !limit_cpu_str.is_empty() {
            limit_cpu = str::parse(limit_cpu_str).unwrap();
        }

        let mut limit_memory = 0;
        if !limit_memory_str.is_empty() {
            limit_memory = str::parse(limit_memory_str).unwrap();
        }

        let mut priority = 0;
        if !priority_str.is_empty() {
            priority = str::parse(priority_str).unwrap();
        }

        let load_end = InitTrace::find_matching_bracket(other, 0).unwrap();
        let load_str = &other[1..load_end];

        let node_selector_end = InitTrace::find_matching_bracket(other, load_end + 2).unwrap();
        let node_selector_str = &other[load_end + 3..node_selector_end];

        let tolerations_end = InitTrace::find_matching_bracket(other, node_selector_end + 2).unwrap();
        let tolerations_str = &other[node_selector_end + 3..tolerations_end];

        let node_affinity_end = InitTrace::find_matching_bracket(other, tolerations_end + 2).unwrap();
        let node_affinity_str = &other[tolerations_end + 3..node_affinity_end];
        assert_eq!(node_affinity_end + 1, other.len());

        let mut node_selector = BTreeMap::<String, String>::new();
        if !node_selector_str.is_empty() {
            for key_value in node_selector_str.split(',') {
                let (key, value) = key_value.split_once(':').unwrap();
                node_selector.insert(key.to_string(), value.to_string());
            }
        }

        let mut node_affinity = NodeAffinity::default();
        if !node_affinity_str.is_empty() {
            node_affinity = str::parse(node_affinity_str).unwrap();
        }

        let mut tolerations: Vec<Toleration> = Vec::new();
        if !tolerations_str.is_empty() {
            for toleration_str in tolerations_str.split(';') {
                tolerations.push(str::parse(toleration_str).unwrap());
            }
        }

        Ok(Self {
            request_cpu,
            request_memory,
            limit_cpu,
            limit_memory,
            priority,
            load: str::parse(load_str).unwrap(),
            node_selector,
            tolerations,
            node_affinity,
        })
    }
}

// https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PodPhase {
    // [Kubelet IN]:    {}
    // [Kubelet OUT]:   { When pod is removed because kubelet turns off }
    #[default]
    Pending = 0,

    // [Kubelet IN]:    { Scheduler asks to run pod on node }
    // [Kubelet OUT]:   {}
    Running = 1,

    // [Kubelet IN]:    {}
    // [Kubelet OUT]:   { When pod's load successfully finished }
    Succeeded = 2,

    // [Kubelet IN]:    {}
    // [Kubelet OUT]:   { When pod is finished because its usage exceeds its limits }
    Failed = 3,

    // [Kubelet IN]:    {}
    // [Kubelet OUT]:   { When pod is evicted because inner node resource pressure }
    Evicted = 4,

    // [Kubelet IN]:    { Scheduler asks to preempt pod }
    // [Kubelet OUT]:   { When pod is preempted because scheduler asked }
    Preempted = 5,

    // [Kubelet IN]:    { Scheduler asks to remove pod }
    // [Kubelet OUT]:   { When pod is asked to be removed }
    Removed = 6,
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

    #[serde(skip)]
    pub cluster_resource_starvation: bool,
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

impl FromStr for Pod {
    type Err = ();

    /// Expects "{<ObjectMeta>};{<PodSpec>}"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let metadata_end = InitTrace::find_matching_bracket(s, 0).unwrap();

        let metadata_str = &s[1..metadata_end];
        let spec_str = &s[metadata_end + 3..s.len() - 1];

        let mut metadata = ObjectMeta::default();
        if !metadata_str.is_empty() {
            metadata = str::parse(metadata_str).unwrap();
        }

        Ok(Self {
            spec: str::parse(spec_str).unwrap(),
            metadata,
            status: PodStatus::default(),
        })
    }
}

impl Pod {
    pub fn prepare(&mut self, group_uid: u64) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.metadata.uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        self.metadata.group_uid = group_uid;
        sim_assert!(group_uid != 0, "Pod. group_uid must be != 0.");

        self.status.phase = PodPhase::Pending;
        self.status.node_uid = None;

        if self.spec.limit_cpu == 0 {
            self.spec.limit_cpu = i64::MAX;
        }
        if self.spec.limit_memory == 0 {
            self.spec.limit_memory = i64::MAX;
        }

        // For a Pod to be given a QoS class of Guaranteed:
        //   - Pod must have a memory limit and a memory request.
        //   - Pod's memory limit must equal the memory request.
        //   - Pod must have a CPU limit and a CPU request.
        //   - Pod's CPU limit must equal the CPU request.
        if self.spec.request_cpu == self.spec.limit_cpu && self.spec.request_memory == self.spec.limit_memory {
            self.status.qos_class = QoSClass::Guaranteed;
        }
        // A Pod is given a QoS class of Burstable if:
        //   - The Pod does not meet the criteria for QoS class Guaranteed.
        //   - Pod has a memory or CPU request or limit.
        else if self.spec.request_cpu != 0
            || self.spec.request_memory != 0
            || self.spec.limit_cpu < i64::MAX
            || self.spec.limit_memory < i64::MAX
        {
            self.status.qos_class = QoSClass::Burstable;
        }
        // A Pod has a QoS class of BestEffort if:
        //   - It doesn't meet the criteria for either Guaranteed or Burstable.
        else {
            self.status.qos_class = QoSClass::BestEffort;
        }

        sim_assert!(
            self.spec.limit_cpu >= self.spec.request_cpu,
            "Pod.spec.limit_cpu must be >= Pod.spec.request_cpu."
        );
        sim_assert!(
            self.spec.limit_memory >= self.spec.request_memory,
            "Pod.spec.limit_memory must be >= Pod.spec.request_memory."
        );
        sim_assert!(self.spec.request_cpu > 0, "Pod.spec.request_cpu must be > 0.");
        sim_assert!(self.spec.request_memory > 0, "Pod.spec.request_memory must be > 0.");
        sim_assert!(self.spec.request_cpu >= 0, "Pod.spec.request_cpu must be >= 0.");
        sim_assert!(self.spec.request_memory >= 0, "Pod.spec.request_memory must be >= 0.");
    }

    pub fn is_usage_matches_limits(&self, cpu: i64, memory: i64) -> bool {
        return cpu <= self.spec.limit_cpu && memory <= self.spec.limit_memory;
    }

    pub fn is_usage_matches_requests(&self, cpu: i64, memory: i64) -> bool {
        return cpu <= self.spec.request_cpu && memory <= self.spec.request_memory;
    }
}

///////////////////////////////////////////// Test /////////////////////////////////////////////////

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_many_times() {
        for _ in 0..10 {
            test_pod_qos_class();
        }
    }

    pub fn test_pod_qos_class() {
        let mut q: BTreeSet<(QoSClass, i64, u64)> = BTreeSet::new();
        q.insert((QoSClass::Guaranteed, 2, 2));
        q.insert((QoSClass::BestEffort, 0, 1));
        q.insert((QoSClass::Guaranteed, 1, 0));
        q.insert((QoSClass::BestEffort, 0, 0));
        q.insert((QoSClass::Burstable, 0, 0));

        assert_eq!(q.pop_first(), Some((QoSClass::BestEffort, 0, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::BestEffort, 0, 1)));
        assert_eq!(q.pop_first(), Some((QoSClass::Burstable, 0, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::Guaranteed, 1, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::Guaranteed, 2, 2)));
        assert_eq!(q.pop_first(), None);
    }
}
