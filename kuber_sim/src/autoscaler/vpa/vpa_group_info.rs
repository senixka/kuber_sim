use crate::my_imports::*;

#[derive(Debug, Default)]
pub struct VPAGroupInfo {
    // Pods VPA info
    pub uids: HashMap<u64, VPAPodInfo>,

    // Submitted with EventAddPod
    pub pod_template: Pod,
}

impl VPAGroupInfo {
    pub fn update_with_new_pod(&mut self, pod: &Pod, current_time: f64) {
        // It is truly new pod
        assert!(!self.uids.contains_key(&pod.metadata.uid));

        // Update pod template if necessary
        if self.pod_template.vpa_profile.is_none() {
            assert!(pod.vpa_profile.is_some());
            self.pod_template = pod.clone();
        }

        // Add info to uids
        assert!(!self.uids.contains_key(&pod.metadata.uid));
        self.uids.insert(pod.metadata.uid, VPAPodInfo::new(pod, current_time));
    }

    pub fn update_with_pod_metrics(
        &mut self,
        pod_uid: u64,
        current_phase: PodPhase,
        current_cpu: f64,
        current_memory: f64,
        current_time: f64,
    ) {
        // Locate pod info
        let pod_info = self.uids.get_mut(&pod_uid).unwrap();
        // Update pod info
        pod_info.update_with_metrics(
            &self.pod_template.vpa_profile.clone().unwrap(),
            current_time,
            current_phase,
            current_cpu,
            current_memory,
        );
    }

    pub fn remove_all_finished(&mut self) -> Vec<(u64, VPAPodInfo)> {
        // Find all finished uids
        let finished: Vec<(u64, VPAPodInfo)> = self
            .uids
            .iter()
            .filter_map(|x| match x.1.is_finished() {
                true => Some((x.0.clone(), x.1.clone())),
                false => None,
            })
            .collect();

        // Remove all finished uids
        for (uid, _) in &finished {
            self.uids.remove(&uid);
        }

        return finished;
    }

    pub fn update_all_with_time(&mut self, profile: &VPAProfile, current_time: f64) {
        for (_, info) in self.uids.iter_mut() {
            info.update_with_time(profile, current_time);
        }
    }
}
