use std::process::abort;
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
        let pod_1 = Pod::from_yaml("./data/pod_1.yaml");
        let pod_2 = Pod::from_yaml("./data/pod_1.yaml");
        let mut pod_3 = Pod::from_yaml("./data/pod_1.yaml");
        let mut pod_4 = Pod::from_yaml("./data/pod_1.yaml");

        pod_3.spec.arrival_time = pod_1.spec.arrival_time + 1.0;
        pod_4.spec.arrival_time = pod_3.spec.arrival_time;

        self.ctx.emit(APIAddPod{pod: pod_1.clone()}, self.api_sim_id, pod_1.spec.arrival_time);
        self.ctx.emit(APIAddPod{pod: pod_2.clone()}, self.api_sim_id, pod_2.spec.arrival_time);
        self.ctx.emit(APIAddPod{pod: pod_3.clone()}, self.api_sim_id, pod_3.spec.arrival_time);
        self.ctx.emit(APIAddPod{pod: pod_4.clone()}, self.api_sim_id, pod_4.spec.arrival_time);
    }

    pub fn submit_nodes(&self, sim: &mut dsc::Simulation) {
        let kubelet_1 = Rc::new(RefCell::new(Kubelet::new(
            sim.create_context("kubelet_1"),
            Node::from_yaml("./data/node_1.yaml")
        )));
        let kubelet_1_id = sim.add_handler("kubelet_1", kubelet_1.clone());
        kubelet_1.borrow_mut().presimulation_init(self.api_sim_id);

        self.ctx.emit_now(APIAddNode{
            kubelet_sim_id: kubelet_1_id,
            node: kubelet_1.borrow().node.clone()
        }, self.api_sim_id);
    }
}

impl dsc::EventHandler for Init {
    fn on(&mut self, _: dsc::Event) {
        println!("Init EventHandler -> Abort");
        abort();
    }
}
