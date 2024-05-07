use crate::my_imports::*;

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#podspec-v1-core
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PodSpec {
    pub load: LoadType,

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

    #[serde(default)]
    pub node_selector: BTreeMap<String, String>,
    #[serde(default)]
    pub tolerations: Vec<Toleration>,
    #[serde(default)]
    pub node_affinity: NodeAffinity,
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

        assert!(self.spec.limit_cpu >= self.spec.request_cpu);
        assert!(self.spec.limit_memory >= self.spec.request_memory);
        assert!(self.spec.request_cpu >= 0);
        assert!(self.spec.request_memory >= 0);
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

