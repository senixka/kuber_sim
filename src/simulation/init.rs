use crate::my_imports::*;


pub struct Init {
    ctx: dsc::SimulationContext,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    cluster_state: Rc<RefCell<ClusterState>>,
    workload: Rc<RefCell<WorkLoad>>,
}


impl Init {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               workload: Rc<RefCell<WorkLoad>>,
               monitoring: Rc<RefCell<Monitoring>>,
               api_sim_id: dsc::Id) -> Self {
        Self {
            ctx,
            api_sim_id,
            monitoring,
            cluster_state,
            workload,
        }
    }

    pub fn submit_pods(&self) {
        let mut pod_count: u64 = 0;
        let mut last_time: f64 = 0.0;

        let binding = self.workload.borrow();
        let mut pod_group_iter = binding.pods.iter().peekable();
        // let mut hpa_pod_group_iter = binding.hpa_pods.iter().peekable();
        while pod_group_iter.peek().is_some() /*|| hpa_pod_group_iter.peek().is_some()*/ {
            let pod_group = (*pod_group_iter.peek().unwrap()).clone();
            pod_group_iter.next();
            // let pod_group: PodGroup = match (pod_group_iter.peek(), hpa_pod_group_iter.peek()) {
            //     (None, Some(&&ref hpa_pod_group)) => {
            //         hpa_pod_group_iter.next();
            //         hpa_pod_group.pod_group.clone()
            //     }
            //     (Some(&&ref pod_group), None) => {
            //         pod_group_iter.next();
            //         pod_group.clone()
            //     }
            //     (Some(&&ref pod_group), Some(&&ref hpa_pod_group)) => {
            //         if pod_group.pod.spec.arrival_time <= hpa_pod_group.pod_group.pod.spec.arrival_time {
            //             pod_group_iter.next();
            //             pod_group.clone()
            //         } else {
            //             hpa_pod_group_iter.next();
            //             hpa_pod_group.pod_group.clone()
            //         }
            //     }
            //     (None, None) => {
            //         panic!("Bad loop");
            //     }
            // };

            pod_count += pod_group.amount;

            for _ in 0..pod_group.amount {
                let mut pod = pod_group.pod.clone();
                pod.prepare(pod_group.group_uid);

                assert!(last_time <= pod.spec.arrival_time);
                self.ctx.emit_ordered(EventAddPod { pod: pod.clone() },
                                      self.api_sim_id,
                                      pod.spec.arrival_time
                );
                last_time = pod.spec.arrival_time;
            }
        }
        self.monitoring.borrow_mut().set_n_pod_in_simulation(pod_count);
    }

    pub fn submit_nodes(&self, sim: &mut dsc::Simulation) {
        for node_group in self.cluster_state.borrow().nodes.iter() {
            for _ in 0..node_group.amount {
                let mut node = node_group.node.clone();
                node.prepare();

                let name = "kubelet_".to_owned() + &*node.metadata.uid.to_string();

                let kubelet = Rc::new(RefCell::new(Kubelet::new(
                    sim.create_context(name.clone()),
                    self.cluster_state.clone(),
                    self.monitoring.clone(),
                    self.api_sim_id,
                    node.clone(),
                )));
                kubelet.borrow_mut().turn_on();

                let kubelet_id = sim.add_handler(name, kubelet.clone());
                self.ctx.emit_now(EventAddNode { kubelet_sim_id: kubelet_id, node: node.clone() }, self.api_sim_id);
            }
        }
    }
}


impl dsc::EventHandler for Init {
    fn on(&mut self, _: dsc::Event) {
        panic!("Init EventHandler -> Panic");
    }
}
