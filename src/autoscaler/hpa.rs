use crate::my_imports::*;


pub struct HPA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,

    is_turned_on: bool,
    groups: Vec<HPAPodGroup>,
}


impl HPA {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               work_load: Rc<RefCell<WorkLoad>>,
               api_sim_id: dsc::Id) -> Self {
        Self {
            ctx,
            cluster_state: cluster_state.clone(),
            api_sim_id,
            is_turned_on: false,
            groups: work_load.borrow().hpa_pods.clone(),
        }
    }

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

    pub fn process_metrics(&mut self, pod_groups: &Vec<(u64, f64, f64)>) {
        for (i, &(size, cpu, memory)) in pod_groups.iter().enumerate() {
            let hpa_pg = &self.groups[i];
            if hpa_pg.min_size > size {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (size)", self.ctx.time(), hpa_pg.pod_group.group_uid);

                let mut pod = hpa_pg.pod_group.pod.clone();
                pod.prepare(hpa_pg.pod_group.group_uid);
                self.ctx.emit( EventAddPod { pod }, self.api_sim_id, self.cluster_state.borrow().network_delays.hpa2api);
            } else if hpa_pg.max_size < size {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (size)", self.ctx.time(), hpa_pg.pod_group.group_uid);

                self.ctx.emit( APIRemoveAnyPodInGroup { group_uid: hpa_pg.pod_group.group_uid}, self.api_sim_id, self.cluster_state.borrow().network_delays.hpa2api);
            }
            else if hpa_pg.min_size < size && (cpu <= hpa_pg.min_group_cpu_percent && memory <= hpa_pg.min_group_memory_percent) {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) remove <- cluster (resources)", self.ctx.time(), hpa_pg.pod_group.group_uid);

                self.ctx.emit( APIRemoveAnyPodInGroup { group_uid: hpa_pg.pod_group.group_uid}, self.api_sim_id, self.cluster_state.borrow().network_delays.hpa2api);
            }
            else if hpa_pg.max_size > size && (cpu >= hpa_pg.max_group_cpu_percent || memory >= hpa_pg.max_group_memory_percent) {
                dp_hpa!("{:.12} hpa pod(group_uid:{:?}) add -> cluster (resources)", self.ctx.time(), hpa_pg.pod_group.group_uid);

                let mut pod = hpa_pg.pod_group.pod.clone();
                pod.prepare(hpa_pg.pod_group.group_uid);
                self.ctx.emit( EventAddPod { pod }, self.api_sim_id, self.cluster_state.borrow().network_delays.hpa2api);
            }
        }
    }
}


impl dsc::EventHandler for HPA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            // APIHPATurnOn {} => {
            //     dp_hpa!("{:.12} hpa APIHPATurnOn", self.ctx.time());
            //
            //     self.turn_on();
            // }
            APIHPATurnOff {} => {
                dp_hpa!("{:.12} hpa APIHPATurnOff", self.ctx.time());

                self.turn_off();
            }
            EventSelfUpdate {} => {
                dp_hpa!("{:.12} hpa EventSelfUpdate", self.ctx.time());

                if !self.is_turned_on {
                    panic!("Bad logic. Self update should be canceled.");
                }

                self.ctx.emit(APIGetHPAMetrics { pod_groups: self.groups.iter().map(|x| x.pod_group.group_uid).collect() }, self.api_sim_id, self.cluster_state.borrow().network_delays.hpa2api);
                self.ctx.emit_self(EventSelfUpdate {}, self.cluster_state.borrow().constants.hpa_self_update_period);
            }
            APIPostHPAMetrics { pod_groups } => {
                dp_hpa!("{:.12} hpa APIPostHPAMetrics pod_info:{:?}", self.ctx.time(), pod_groups);

                self.process_metrics(&pod_groups);
            }
        });
    }
}
