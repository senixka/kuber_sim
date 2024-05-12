use crate::my_imports::*;

#[derive(Debug, Default)]
pub struct VPAGroupInfo {
    /// Known pods of this group and their VPA info
    pub uids: HashMap<u64, VPAPodInfo>,

    /// Pod to add when reschedule
    pub pod_template: Pod,
    /// Group VPA profile
    pub vpa_profile: VPAProfile,
}

impl VPAGroupInfo {
    pub fn update_with_new_group(&mut self, pod_group: &PodGroup) {
        // It is truly new group
        assert!(self.uids.is_empty());

        // Set pod template if necessary
        self.pod_template = pod_group.pod.clone();
        // Prepare template
        self.pod_template.prepare(pod_group.group_uid);

        // Set local VPA profile
        self.vpa_profile = pod_group.vpa_profile.clone().unwrap();
    }

    pub fn update_with_new_pod(&mut self, pod: &Pod, current_time: f64) {
        // It is truly new pod
        assert!(!self.uids.contains_key(&pod.metadata.uid));

        // Add info to uids
        assert!(!self.uids.contains_key(&pod.metadata.uid));
        self.uids.insert(pod.metadata.uid, VPAPodInfo::new(pod, current_time));
    }

    pub fn update_with_pod_metrics(
        &mut self,
        init_config: &InitConfig,
        pod_uid: u64,
        current_phase: PodPhase,
        current_cpu: f64,
        current_memory: f64,
        current_time: f64,
    ) {
        // Locate pod info
        let pod_info = self.uids.get_mut(&pod_uid).unwrap();
        // Update pod info
        pod_info.update_with_metrics(init_config, current_time, current_phase, current_cpu, current_memory);
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
        for (uid, _) in finished.iter() {
            self.uids.remove(uid);
        }

        return finished;
    }

    pub fn update_all_with_time(&mut self, init_config: &InitConfig, current_time: f64) {
        for (_, info) in self.uids.iter_mut() {
            info.update_with_time(init_config, current_time);
        }
    }
}
