use crate::my_imports::*;

pub struct Workload {
    ctx: dsc::SimulationContext,
    api_sim_id: dsc::Id,
}

impl Workload {
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
}

impl dsc::EventHandler for Workload {
    fn on(&mut self, event: dsc::Event) {
        println!("Workload EventHandler ------>");
        println!("Workload EventHandler <------");
    }
}
