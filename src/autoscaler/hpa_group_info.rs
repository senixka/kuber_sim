use crate::my_imports::*;


#[derive(Debug, Default)]
pub struct HPAGroupInfo {
    // Group cpu numerator
    pub numerator_cpu: f64,
    // Group memory numerator
    pub numerator_memory: f64,

    // Counts currently running pods
    pub running_pod_count: u64,
    // Contains all not finished pods
    pub alive_uids: BTreeSet<u64>,

    // Never removes its uids
    pub last_metrics: HashMap<u64, (PodPhase, f64, f64)>,

    // Submitted with EventAddPod
    pub pod_template: Pod,
}


impl HPAGroupInfo {
    pub fn update_with_new_pod(&mut self, pod: &Pod) {
        // It is truly new pod
        assert!(!self.alive_uids.contains(&pod.metadata.uid));

        // Update pod template
        if self.pod_template.hpa_profile.is_none() {
            assert!(pod.hpa_profile.is_some());
            self.pod_template = pod.clone();
        }

        // Update last metrics
        self.last_metrics.insert(pod.metadata.uid, (PodPhase::Pending, 0.0, 0.0));
        // Update alive uids
        let _newly_inserted = self.alive_uids.insert(pod.metadata.uid);
        assert!(_newly_inserted);
    }

    pub fn update_with_pod_metrics(&mut self, pod_uid: u64, new_phase: PodPhase, new_cpu: f64, new_memory: f64) {
        // Get last pod metrics
        let (last_phase, last_cpu, last_memory) = self.last_metrics.get(&pod_uid).unwrap().clone();

        dp_hpa!("HPAGroupInfo update with pod_uid:{:?} new_phase:{:?} new_cpu:{:?} new_memory:{:?}", pod_uid, new_phase, new_cpu, new_memory);

        match last_phase {
            PodPhase::Running => {
                // Remove pod's previous utilization from group
                assert!(self.numerator_cpu >= last_cpu);
                assert!(self.numerator_memory >= last_memory);
                self.numerator_cpu -= last_cpu;
                self.numerator_memory -= last_memory;

                match new_phase {
                    PodPhase::Running => {
                        // Changing: Running -> Running

                        // Update utilization
                        self.numerator_cpu += new_cpu;
                        self.numerator_memory += new_memory;
                    }
                    PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                        // Running -> Finished

                        // Decrease running count
                        assert!(self.running_pod_count > 0);
                        self.running_pod_count -= 1;

                        // Remove uid from alive uids
                        let _was_present = self.alive_uids.remove(&pod_uid);
                        assert!(_was_present);
                    }
                    PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                        // Running -> OnReschedule

                        // Decrease running count
                        assert!(self.running_pod_count > 0);
                        self.running_pod_count -= 1;
                    }
                }
            }
            PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                match new_phase {
                    PodPhase::Running => {
                        // Changing: OnReschedule -> Running

                        // Increase utilization
                        self.numerator_cpu += new_cpu;
                        self.numerator_memory += new_memory;

                        // Increase running pod count
                        self.running_pod_count += 1;
                    }
                    PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                        // OnReschedule -> Finished

                        // Remove uid from alive uids
                        let _was_present = self.alive_uids.remove(&pod_uid);
                        assert!(_was_present);
                    }
                    PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                        // OnReschedule -> OnReschedule
                        // Do nothing
                    }
                }
            }
            PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                // Finished pod should not get new phase updates
                panic!("Logic error in HPA metric update. Bad pod phase change:({:?} -> {:?})", last_phase, new_phase);
            }
        }

        // Update last metrics
        self.last_metrics.insert(pod_uid, (new_phase, new_cpu, new_memory));
    }
}
