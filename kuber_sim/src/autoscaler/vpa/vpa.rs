use crate::my_imports::*;

/// The component of the Kubernetes responsible for vertical pod autoscaling.
pub struct VPA {
    /// DSLab-Core simulation context of VPA.
    ctx: dsc::SimulationContext,
    /// Configuration constants of components in simulation.
    init_config: Rc<RefCell<InitConfig>>,
    /// API-Server simulation DSlab-Core Id.
    api_sim_id: dsc::Id,

    /// Is VPA turned on
    is_turned_on: bool,

    /// Managed pod groups and their metrics
    managed_groups: HashMap<u64, VPAGroupInfo>,
}

impl VPA {
    pub fn new(ctx: dsc::SimulationContext, init_config: Rc<RefCell<InitConfig>>, api_sim_id: dsc::Id) -> Self {
        Self {
            ctx,
            init_config: init_config.clone(),
            api_sim_id,

            // VPA is created in turned off state
            is_turned_on: false,

            // Inner state
            managed_groups: HashMap::new(),
        }
    }

    ////////////////// VPA Turn On/Off //////////////////

    pub fn turn_on(&mut self) {
        if !self.is_turned_on {
            self.is_turned_on = true;
            self.ctx
                .emit_self(EventSelfUpdate {}, self.init_config.borrow().vpa.self_update_period);
        }
    }

    pub fn turn_off(&mut self) {
        if self.is_turned_on {
            self.is_turned_on = false;
            self.ctx
                .cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());
        }
    }

    ////////////////// Process metrics //////////////////

    pub fn make_decisions(&mut self) {
        // Look for all managed pod groups
        for (group_uid, group_info) in self.managed_groups.iter_mut() {
            // Get VPA profile
            let profile = group_info.vpa_profile.clone();

            // Remove and store all finished uids from group
            let finished = group_info.remove_all_finished();
            dp_vpa!(
                "VPA removed finished:{:?}",
                finished.iter().map(|x| x.0).collect::<Vec<_>>()
            );
            for (_pod_uid, pod_info) in finished {
                dp_vpa!(
                    "-----> VPA watch pod_uid:{:?} is_finished:{:?}",
                    _pod_uid,
                    pod_info.is_finished()
                );
                // Process only Failed pods
                if !pod_info.is_failed() {
                    continue;
                }
                // If finished pod failed -> reschedule it with new suggested resources

                // Get suggested spec resources
                let (request_cpu, request_memory, limit_cpu, limit_memory) =
                    pod_info.suggest(&self.init_config.borrow(), &profile);
                dp_vpa!(
                    "VPA reschedule failed pod_uid:{:?} with r_cpu:{:?} r_mem:{:?} l_cpu:{:?} l_mem:{:?}",
                    _pod_uid,
                    request_cpu,
                    request_memory,
                    limit_cpu,
                    limit_memory
                );

                // Locate pod template
                let mut pod = group_info.pod_template.clone();

                // Configure limits
                pod.spec.limit_cpu = limit_cpu;
                pod.spec.limit_memory = limit_memory;
                // Configure requests
                pod.spec.request_cpu = request_cpu;
                pod.spec.request_memory = request_memory;
                // Prepare pod
                pod.prepare(*group_uid);

                // Emit AddPod event
                dp_vpa!("VPA emit AddPod pod:{:?}", pod);
                self.ctx.emit(
                    EventAddPod { pod },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.vpa2api,
                );
            }

            // Update all remained uids in group with current time
            group_info.update_all_with_time(&self.init_config.borrow(), self.ctx.time());

            // If pod's consumption very differs from its request -> reschedule pod with new spec
            for (&pod_uid, pod_info) in group_info.uids.iter_mut() {
                // If pod already rescheduled -> skip it
                if pod_info.is_rescheduled {
                    continue;
                }

                // If request and usage differs not too much -> skip
                if !pod_info.need_reschedule(&self.init_config.borrow(), &profile, self.ctx.time()) {
                    continue;
                }

                // Emit RemovePod event
                dp_vpa!("VPA emit RemovePod pod_uid:{:?}", pod_uid);
                self.ctx.emit(
                    EventRemovePod { pod_uid },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.vpa2api,
                );

                // Get suggested spec resources
                let (request_cpu, request_memory, limit_cpu, limit_memory) =
                    pod_info.suggest(&self.init_config.borrow(), &profile);

                // Locate pod template
                let mut pod = group_info.pod_template.clone();

                // Configure limits
                pod.spec.limit_cpu = limit_cpu;
                pod.spec.limit_memory = limit_memory;
                // Configure requests
                pod.spec.request_cpu = request_cpu;
                pod.spec.request_memory = request_memory;
                // Prepare pod
                pod.prepare(*group_uid);

                // Emit AddPod event
                dp_vpa!("VPA emit AddPod pod:{:?}", pod);
                self.ctx.emit(
                    EventAddPod { pod },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.vpa2api,
                );

                // Update pod info
                pod_info.is_rescheduled = true;
            }
        }
    }

    ////////////////// Send Events //////////////////

    // pub fn send_add_pod(&self, pod: Pod) {
    //     self.ctx.emit(
    //         EventAddPod { pod },
    //         self.api_sim_id,
    //         self.cluster_state.borrow().network_delays.vpa2api
    //     );
    // }
    //
    // pub fn send_remove_pod(&self, pod_uid: u64) {
    //     self.ctx.emit(
    //         EventRemovePod { pod_uid },
    //         self.api_sim_id,
    //         self.cluster_state.borrow().network_delays.vpa2api
    //     );
    // }
}

impl dsc::EventHandler for VPA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventTurnOn {} => {
                dp_vpa!("{:.12} vpa EventTurnOn", self.ctx.time());

                self.turn_on();
            }

            EventTurnOff {} => {
                dp_vpa!("{:.12} vpa EventTurnOff", self.ctx.time());

                self.turn_off();
            }

            EventSelfUpdate {} => {
                dp_vpa!("{:.12} vpa EventSelfUpdate", self.ctx.time());

                assert!(
                    self.is_turned_on,
                    "Logic error. Self update should be canceled for VPA."
                );

                // Scale up/down groups managed by VPA
                self.make_decisions();

                // Emit Self-Update
                self.ctx
                    .emit_self(EventSelfUpdate {}, self.init_config.borrow().vpa.self_update_period);
            }

            EventPodMetricsPost {
                group_uid,
                pod_uid,
                current_phase,
                current_cpu,
                current_memory,
            } => {
                dp_vpa!(
                    "{:.12} vpa EventVPAPodMetricsPost group_uid:{:?} pod_uid:{:?} current_phase:{:?} current_cpu:{:?} current_memory:{:?}",
                    self.ctx.time(), group_uid, pod_uid, current_phase, current_cpu, current_memory
                );

                // If this group is not managed by VPA -> return
                if !self.managed_groups.contains_key(&group_uid) {
                    return;
                }

                // Locate group info
                let group_info = self.managed_groups.get_mut(&group_uid).unwrap();
                // Update group info
                group_info.update_with_pod_metrics(
                    &self.init_config.borrow(),
                    pod_uid,
                    current_phase,
                    current_cpu,
                    current_memory,
                    self.ctx.time(),
                );
            }

            EventAddPod { pod } => {
                dp_vpa!("{:.12} vpa EventAddPod pod:{:?}", self.ctx.time(), pod);

                // If this pod should not be managed by VPA -> return
                if !self.managed_groups.contains_key(&pod.metadata.group_uid) {
                    return;
                }

                // Locate pod's group info
                let group_info = self.managed_groups.entry(pod.metadata.group_uid).or_default();
                // Update group info
                group_info.update_with_new_pod(&pod, self.ctx.time());
            }

            EvenAddPodGroup { pod_group } => {
                dp_vpa!("{:.12} vpa EvenAddPodGroup pod_group:{:?}", self.ctx.time(), pod_group);
                assert!(!self.managed_groups.contains_key(&pod_group.group_uid));

                // If this group should not be managed by VPA -> return
                if pod_group.vpa_profile.is_none() {
                    return;
                }

                // Locate pod's group info
                let group_info = self.managed_groups.entry(pod_group.group_uid).or_default();
                // Update group info
                group_info.update_with_new_group(&pod_group);
            }

            EventRemovePodGroup { group_uid } => {
                dp_vpa!(
                    "{:.12} vpa EventRemovePodGroup group_uid:{:?}",
                    self.ctx.time(),
                    group_uid
                );

                // Remove managed group
                self.managed_groups.remove(&group_uid);
            }
        });
    }
}
