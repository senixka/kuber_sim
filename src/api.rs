use serde::Deserialize;
use crate::my_imports::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum APIServerEvent {
    InsertPod = 0,
    RemovePod = 1,
    InsertNode = 2,
    RemoveNode = 3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddPod {
    pub pod: Pod,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemovePod {
    pub pod_uid: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddNode {
    pub node: Node,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveKubelet {
    pub node_uid: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromScheduler {
    pub pod: Pod,
    pub new_phase: PodPhase,
    pub kubelet_sim_id: dsc::Id,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromKubelet {
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}

pub struct APIServer {
    ctx: dsc::SimulationContext,
    etcd_sim_id: dsc::Id,
    scheduler_sim_id: dsc::Id,

    subscriptions: HashMap<APIServerEvent, Vec<dsc::Id>>,
}

impl APIServer {
    pub fn new(ctx: dsc::SimulationContext) -> Self {
        Self {
            ctx,
            etcd_sim_id: dsc::Id::MAX,
            scheduler_sim_id: dsc::Id::MAX,
            subscriptions: HashMap::new(),
        }
    }

    pub fn presimulation_init(&mut self, etcd_sim_id: dsc::Id, scheduler_sim_id: dsc::Id) {
        self.etcd_sim_id = etcd_sim_id;
        self.scheduler_sim_id = scheduler_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.scheduler_sim_id, dsc::Id::MAX);
        assert_ne!(self.etcd_sim_id, dsc::Id::MAX);
    }

    pub fn subscribe(&mut self, event: APIServerEvent, sim_id: dsc::Id) {
        self.subscriptions.entry(event).or_default().push(sim_id);
    }
}

impl dsc::EventHandler for APIServer {
    fn on(&mut self, event: dsc::Event) {
        println!("API EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, kubelet_sim_id } => {
                println!("API route <Update Pod From Scheduler>");
                let data = APIUpdatePodFromScheduler { pod, new_phase, kubelet_sim_id };

                self.ctx.emit_now(data.clone(), kubelet_sim_id);
            }
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid} => {
                println!("API route <Update Pod From Kubelet>");
                let data = APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid };

                self.ctx.emit_now(data.clone(), self.scheduler_sim_id);
            }
            APIAddPod { pod } => {
                println!("API route <Add Pod>");
                let data = APIAddPod { pod };

                // self.ctx.emit_now(data.clone(), self.etcd_sim_id);
                self.ctx.emit_now(data.clone(), self.scheduler_sim_id);
            }
            // APIRemovePod { pod_uid } => {
            //     println!("API route <Remove Pod>");
            //     let data = APIRemovePod { uid };
            //
            //     self.ctx.emit_now(data.clone(), self.etcd_sim_id);
            //
            //     for sim_id in self.subscriptions.entry(APIServerEvent::RemovePod).or_default().iter() {
            //         self.ctx.emit_now(data.clone(), *sim_id);
            //     }
            // }
            APIAddNode { node } => {
                println!("API route <Insert Node>");
                let data = APIAddNode { node };

                // self.ctx.emit_now(data.clone(), self.etcd_sim_id);
                self.ctx.emit_now(data.clone(), self.scheduler_sim_id);
            }
            // APIRemoveKubelet { node_uid } => {
            //     println!("API route <Remove Node>");
            //     let data = APIRemoveNode { uid };
            //
            //     for sim_id in self.subscriptions.entry(APIServerEvent::RemoveNode).or_default().iter() {
            //         self.ctx.emit_now(data.clone(), *sim_id);
            //     }
            // }
        });
        println!("API EventHandler <------");
    }
}
