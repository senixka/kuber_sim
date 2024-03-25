use crate::my_imports::*;

pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub api_sim_id: dsc::Id,
    pub node: Node,
}

impl Kubelet {
    pub fn new(ctx: dsc::SimulationContext, node: Node) -> Self {
        Self {
            ctx,
            api_sim_id: dsc::Id::MAX,
            node,
        }
    }

    pub fn presimulation_init(&mut self, kubelet_sim_id: dsc::Id, api_sim_id: dsc::Id) {
        self.node.kubelet_sim_id = Some(kubelet_sim_id);
        self.api_sim_id = api_sim_id;
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        println!("Kubelet_{0} Node_{1} EventHandler ------>", self.node.kubelet_sim_id.unwrap(), self.node.metadata.uid);
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, kubelet_sim_id } => {
                assert_eq!(new_phase, PodPhase::Running);

                let data = APIUpdatePodFromKubelet {
                    pod_uid: pod.metadata.uid,
                    new_phase: PodPhase::Succeeded,
                    node_uid: self.node.metadata.uid,
                };
                self.ctx.emit(data, self.api_sim_id, pod.spec.load_profile[0].duration);
            }
        });
        println!("Kubelet_{0} Node_{1} EventHandler <------", self.node.kubelet_sim_id.unwrap(), self.node.metadata.uid);
    }
}
