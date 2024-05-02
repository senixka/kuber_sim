use crate::my_imports::*;


pub struct HPA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,

    // Is CA turned on
    is_turned_on: bool,

    // Managed pod groups
    pod_groups: HashMap<u64, PodGroup>,
}


impl HPA {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               work_load: Rc<RefCell<WorkLoad>>,
               api_sim_id: dsc::Id) -> Self {
        let mut hpa = Self {
            ctx,
            cluster_state: cluster_state.clone(),
            api_sim_id,

            // HPA is created in turned off state
            is_turned_on: false,

            // Inner state
            pod_groups: HashMap::new(),
        };

        // Parse trace, store all hpa profiles
        for pod_group in &work_load.borrow().pods {
            if pod_group.pod.hpa_profile.is_some() {
                hpa.pod_groups.insert(pod_group.group_uid, pod_group.clone());
            }
        }

        return hpa;
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

    // pub fn process_metrics(&mut self, group_utilization: &Vec<(u64, Option(u64, f64, f64))>) {
    //     // Note: This function relies on the fact that the order and number of elements
    //     //       in group_utilization and self.groups are the same
    //
    //     for (i, &(group_size, cpu, memory)) in group_utilization.iter().enumerate() {
    //         // Locate current hpa pod group
    //         let hpa_pg = &self.pod_groups[i];
    //         // Locate current hpa profile
    //         let profile = hpa_pg.pod.hpa_profile.unwrap();
    //
    //         // If group is too small -> AddPod
    //         if hpa_pg.min_size > group_size {
    //             dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (size)", self.ctx.time(), hpa_pg.pod_group.group_uid);
    //
    //             // Locate pod template
    //             let mut pod = hpa_pg.pod_group.pod.clone();
    //             // Prepare pod
    //             pod.prepare(hpa_pg.pod_group.group_uid);
    //             // Emit AddPod event
    //             self.send_add_pod(pod);
    //         }
    //
    //         // If group size is too big -> RemovePod
    //         else if hpa_pg.max_size < group_size {
    //             dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (size)", self.ctx.time(), hpa_pg.pod_group.group_uid);
    //
    //             self.send_remove_any_pod_in_group(hpa_pg.pod_group.group_uid);
    //         }
    //
    //         // If group utilization is low and group size allows -> RemovePod
    //         else if hpa_pg.min_size < group_size
    //             && (cpu <= hpa_pg.min_group_cpu_percent && memory <= hpa_pg.min_group_memory_percent) {
    //             dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (resources)", self.ctx.time(), hpa_pg.pod_group.group_uid);
    //
    //             self.send_remove_any_pod_in_group(hpa_pg.pod_group.group_uid);
    //         }
    //
    //         // If group utilization is high and group size allows -> AddPod
    //         else if hpa_pg.max_size > group_size
    //             && (cpu >= hpa_pg.max_group_cpu_percent || memory >= hpa_pg.max_group_memory_percent) {
    //             dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (resources)", self.ctx.time(), hpa_pg.pod_group.group_uid);
    //
    //             // Locate pod template
    //             let mut pod = hpa_pg.pod_group.pod.clone();
    //             // Prepare pod
    //             pod.prepare(hpa_pg.pod_group.group_uid);
    //             // Emit AddPod event
    //             self.send_add_pod(pod);
    //         }
    //     }
    // }


    ////////////////// Send Events //////////////////

    pub fn send_add_pod(&self, pod: Pod) {
        self.ctx.emit(
            EventAddPod { pod },
            self.api_sim_id,
            self.cluster_state.borrow().network_delays.hpa2api
        );
    }

    pub fn send_remove_any_pod_in_group(&self, group_uid: u64) {
        self.ctx.emit(
            EventRemoveAnyPodInGroup { group_uid },
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

                // Request metrics for future process
                self.ctx.emit(
                    EventGetHPAMetrics { pod_groups: self.pod_groups.keys().map(|x| *x).collect() },
                    self.api_sim_id,
                    self.cluster_state.borrow().network_delays.hpa2api
                );

                // Emit Self-Update
                self.ctx.emit_self(
                    EventSelfUpdate {},
                    self.cluster_state.borrow().constants.hpa_self_update_period
                );
            }

            EventPostHPAMetrics { group_utilization } => {
                dp_hpa!("{:.12} hpa EventPostHPAMetrics pod_info:{:?}", self.ctx.time(), group_utilization);

                // self.process_metrics(&group_utilization);
            }
        });
    }
}
