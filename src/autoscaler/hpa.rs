use crate::my_imports::*;


#[derive(Debug, Default)]
struct HPAGroupInfo {
    // Group cpu numerator
    numerator_cpu: f64,
    // Group memory numerator
    numerator_memory: f64,

    // Counts currently running pods
    running_pod_count: u64,
    // Contains all not finished pods
    alive_uids: BTreeSet<u64>,

    // Never removes its uids
    last_metrics: HashMap<u64, (PodPhase, f64, f64)>,

    // Submitted with EventAddPod
    pod_template: Pod,
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

pub struct HPA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,

    // Is CA turned on
    is_turned_on: bool,

    // Managed pod groups and their metrics
    managed_groups: HashMap<u64, HPAGroupInfo>,
}


impl HPA {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               api_sim_id: dsc::Id) -> Self {
        Self {
            ctx,
            cluster_state: cluster_state.clone(),
            api_sim_id,

            // HPA is created in turned off state
            is_turned_on: false,

            // Inner state
            managed_groups: HashMap::new(),
        }
    }

    ////////////////// HPA Turn On/Off //////////////////

    pub fn turn_on(&mut self) {
        if !self.is_turned_on {
            self.is_turned_on = true;
            self.ctx.emit_self(EventSelfUpdate {}, self.cluster_state.borrow().constants.hpa_self_update_period);
        }
    }

    pub fn turn_off(&mut self) {
        if self.is_turned_on {
            self.is_turned_on = false;
            self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());
        }
    }


    ////////////////// Process metrics //////////////////

    pub fn make_decisions(&mut self) {
        for (group_uid, info) in &self.managed_groups {
            // Get current group size
            let group_size = info.alive_uids.len() as u64;

            // Count current cpu and memory mean utilization
            let cpu = info.numerator_cpu / (info.running_pod_count as f64);
            let memory = info.numerator_memory / (info.running_pod_count as f64);

            // Locate current HPA profile
            let profile = info.pod_template.clone().hpa_profile.unwrap();

            // If group is too small -> AddPod
            if profile.min_size > group_size {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (size)", self.ctx.time(), group_uid);

                // Locate pod template
                let mut pod = info.pod_template.clone();
                // Prepare pod
                pod.prepare(*group_uid);
                // Emit AddPod event
                self.send_add_pod(pod);
            }

            // If group size is too big -> RemovePod
            else if profile.max_size < group_size {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (size)", self.ctx.time(), group_uid);

                self.send_remove_pod(*info.alive_uids.last().unwrap());
            }

            // If group utilization is low and group size allows -> RemovePod
            else if profile.min_size < group_size
                && (cpu <= profile.min_group_cpu_percent && memory <= profile.min_group_memory_percent) {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (resources)", self.ctx.time(), group_uid);

                self.send_remove_pod(*info.alive_uids.last().unwrap());
            }

            // If group utilization is high and group size allows -> AddPod
            else if profile.max_size > group_size
                && (cpu >= profile.max_group_cpu_percent || memory >= profile.max_group_memory_percent) {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (resources)", self.ctx.time(), group_uid);

                // Locate pod template
                let mut pod = info.pod_template.clone();
                // Prepare pod
                pod.prepare(*group_uid);
                // Emit AddPod event
                self.send_add_pod(pod);
            }
        }
    }


    ////////////////// Send Events //////////////////

    pub fn send_add_pod(&self, pod: Pod) {
        self.ctx.emit(
            EventAddPod { pod },
            self.api_sim_id,
            self.cluster_state.borrow().network_delays.hpa2api
        );
    }

    pub fn send_remove_pod(&self, pod_uid: u64) {
        self.ctx.emit(
            EventRemovePod { pod_uid },
            self.api_sim_id,
            self.cluster_state.borrow().network_delays.hpa2api
        );
    }
}


impl dsc::EventHandler for HPA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventTurnOn {} => {
                dp_hpa!("{:.12} hpa EventTurnOn", self.ctx.time());

                self.turn_on();
            }

            EventTurnOff {} => {
                dp_hpa!("{:.12} hpa EventTurnOff", self.ctx.time());

                self.turn_off();
            }

            EventSelfUpdate {} => {
                dp_hpa!("{:.12} hpa EventSelfUpdate", self.ctx.time());

                assert!(self.is_turned_on, "Logic error. Self update should be canceled for HPA.");

                // Scale up/down groups managed by HPA
                self.make_decisions();

                // Emit Self-Update
                self.ctx.emit_self(
                    EventSelfUpdate {},
                    self.cluster_state.borrow().constants.hpa_self_update_period
                );
            }

            EventHPAPodMetricsPost { group_uid, pod_uid, current_phase, current_cpu, current_memory } => {
                dp_hpa!("{:.12} hpa EventHPAPodMetricsPost group_uid:{:?} pod_uid:{:?} current_phase:{:?} current_cpu:{:?} current_memory:{:?}", self.ctx.time(), group_uid, pod_uid, current_phase, current_cpu, current_memory);

                // If this group is not managed by HPA -> return
                if !self.managed_groups.contains_key(&group_uid) {
                    return;
                }

                // Locate group info
                let group_info = self.managed_groups.get_mut(&group_uid).unwrap();
                // Update group info
                group_info.update_with_pod_metrics(pod_uid, current_phase, current_cpu, current_memory);
            }

            EventAddPod { pod } => {
                dp_hpa!("{:.12} hpa EventAddPod pod:{:?}", self.ctx.time(), pod);

                // If this pod should not be managed by HPA -> return
                if pod.hpa_profile.is_none() {
                    return;
                }

                // Locate pod's group info
                let group_info = self.managed_groups.entry(pod.metadata.group_uid).or_default();
                // Update group info
                group_info.update_with_new_pod(&pod);
            }
        });
    }
}
