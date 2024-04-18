use crate::my_imports::*;


pub struct APIServer {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    scheduler_sim_id: dsc::Id,

    subscriptions: HashMap<APIServerEvent, Vec<dsc::Id>>,

    // ############## ETCD ##############
    pods: HashMap<u64, Pod>, // Pod uid -> Pod
    kubelets: HashMap<u64, dsc::Id>, // Node uid -> Kubelet uid
}


impl APIServer {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>) -> Self {
        Self {
            ctx,
            cluster_state,
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
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                debug_print!("{:.12} api_server APIUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod.metadata.uid, node_uid, new_phase);

                self.pods.get_mut(&pod.metadata.uid).unwrap().status.phase = new_phase.clone();
                self.ctx.emit(APIUpdatePodFromScheduler { pod, new_phase, node_uid }, self.kubelets[&node_uid], self.cluster_state.borrow().network_delays.api2kubelet);
            }
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid} => {
                debug_print!("{:.12} api_server APIUpdatePodFromKubelet pod_uid:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod_uid, node_uid, new_phase);

                self.pods.get_mut(&pod_uid).unwrap().status.phase = new_phase.clone();
                self.ctx.emit(APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIAddPod { pod } => {
                debug_print!("{:.12} api_server APIAddPod pod:{:?}", self.ctx.time(), pod);

                self.pods.insert(pod.metadata.uid, pod.clone());
                self.ctx.emit(APIAddPod { pod }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIAddNode { kubelet_sim_id, node } => {
                debug_print!("{:.12} api_server APIAddNode node:{:?}", self.ctx.time(), node);

                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);
                self.ctx.emit(APIAddNode { kubelet_sim_id, node }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
        });
    }
}
