use crate::my_imports::*;

pub struct Init {
    ctx: dsc::SimulationContext,
    api_sim_id: dsc::Id,
}

impl Init {
    pub fn new(ctx: dsc::SimulationContext) -> Self {
        Self {
            ctx,
            api_sim_id: dsc::Id::MAX,
        }
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.api_sim_id, dsc::Id::MAX);
    }

    pub fn submit_pods(&self) {
        let mut last_time: f64 = 0.0;
        for pod_group in SimConfig::pods().iter() {
            for _ in 0..pod_group.amount {
                let mut pod = pod_group.pod.clone();
                pod.init();

                assert!(last_time <= pod.spec.arrival_time);
                self.ctx.emit_ordered(APIAddPod{ pod: pod.clone() }, self.api_sim_id, pod.spec.arrival_time);
                last_time = pod.spec.arrival_time;
            }
        }
    }

    pub fn submit_nodes(&self, sim: &mut dsc::Simulation) {
        for node_group in SimConfig::nodes().iter() {
            for _ in 0..node_group.amount {
                let mut node = node_group.node.clone();
                node.init();

                let name = "kubelet_".to_owned() + &*node.metadata.uid.to_string();
                println!("{0}", name);

                let kubelet = Rc::new(RefCell::new(Kubelet::new(
                    sim.create_context(name.clone()),
                    node.clone(),
                )));
                kubelet.borrow_mut().presimulation_init(self.api_sim_id);

                let kubelet_id = sim.add_handler(name, kubelet.clone());
                self.ctx.emit_now(APIAddNode{ kubelet_sim_id: kubelet_id, node: node.clone(), }, self.api_sim_id);
            }
        }
    }
}

impl dsc::EventHandler for Init {
    fn on(&mut self, _: dsc::Event) {
        panic!("Init EventHandler -> Panic");
    }
}
