use serde::Deserialize;
use crate::my_imports::*;

// ################# API EVENTS #################

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
    pub kubelet_sim_id: dsc::Id,
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
    pub node_uid: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromKubelet {
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}

// ################# API SERVER #################

pub struct APIServer {
    ctx: dsc::SimulationContext,
    scheduler_sim_id: dsc::Id,

    subscriptions: HashMap<APIServerEvent, Vec<dsc::Id>>,

    // ############## ETCD ##############
    pods: HashMap<u64, Pod>, // Pod uid -> Pod
    kubelets: HashMap<u64, dsc::Id>, // Node uid -> Kubelet uid
}

impl APIServer {
    pub fn new(ctx: dsc::SimulationContext) -> Self {
        Self {
            ctx,
            scheduler_sim_id: dsc::Id::MAX,
            subscriptions: HashMap::new(),
            pods: HashMap::new(),
            kubelets: HashMap::new(),
        }
    }

    pub fn presimulation_init(&mut self, scheduler_sim_id: dsc::Id) {
        self.scheduler_sim_id = scheduler_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.scheduler_sim_id, dsc::Id::MAX);
    }

    pub fn subscribe(&mut self, event: APIServerEvent, sim_id: dsc::Id) {
        self.subscriptions.entry(event).or_default().push(sim_id);
    }
}

impl dsc::EventHandler for APIServer {
    fn on(&mut self, event: dsc::Event) {
        println!("API EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                println!("API route <Update Pod From Scheduler>");

                self.pods.get_mut(&pod.metadata.uid).unwrap().status.phase = new_phase.clone();
                self.ctx.emit(APIUpdatePodFromScheduler { pod, new_phase, node_uid }, self.kubelets[&node_uid], NetworkDelays::api2kubelet());
            }
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid} => {
                println!("API route <Update Pod From Kubelet>");

                self.pods.get_mut(&pod_uid).unwrap().status.phase = new_phase.clone();
                self.ctx.emit(APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid }, self.scheduler_sim_id, NetworkDelays::api2scheduler());
            }
            APIAddPod { pod } => {
                println!("API route <Add Pod>");

                self.pods.insert(pod.metadata.uid, pod.clone());
                self.ctx.emit(APIAddPod { pod }, self.scheduler_sim_id, NetworkDelays::api2scheduler());
            }
            APIAddNode { kubelet_sim_id, node } => {
                println!("API route <Insert Node>");

                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);
                self.ctx.emit(APIAddNode { kubelet_sim_id, node }, self.scheduler_sim_id, NetworkDelays::api2scheduler());
            }
        });
        println!("API EventHandler <------");
    }
}
