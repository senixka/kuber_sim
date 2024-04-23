use crate::my_imports::*;


pub struct APIServer {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    scheduler_sim_id: dsc::Id,
    ca_sim_id: dsc::Id,

    // subscriptions: HashMap<APIServerEvent, Vec<dsc::Id>>,

    // ############## ETCD ##############
    // pods: HashMap<u64, Pod>, // Pod uid -> Pod
    kubelets: HashMap<u64, dsc::Id>, // Node uid -> Kubelet uid
}


impl APIServer {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>) -> Self {
        Self {
            ctx,
            cluster_state,
            scheduler_sim_id: dsc::Id::MAX,
            ca_sim_id: dsc::Id::MAX,
            // subscriptions: HashMap::new(),
            // pods: HashMap::new(),
            kubelets: HashMap::new(),
        }
    }

    pub fn presimulation_init(&mut self, scheduler_sim_id: dsc::Id, ca_sim_id: dsc::Id) {
        self.scheduler_sim_id = scheduler_sim_id;
        self.ca_sim_id = ca_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.scheduler_sim_id, dsc::Id::MAX);
    }

    // pub fn subscribe(&mut self, event: APIServerEvent, sim_id: dsc::Id) {
    //     self.subscriptions.entry(event).or_default().push(sim_id);
    // }
}


impl dsc::EventHandler for APIServer {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                dp_api_server!("{:.12} api_server APIUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod.metadata.uid, node_uid, new_phase);

                let to = self.kubelets.get(&node_uid);
                match to {
                    Some(kubelet_id) => {
                        self.ctx.emit(APIUpdatePodFromScheduler { pod, new_phase, node_uid }, *kubelet_id, self.cluster_state.borrow().network_delays.api2kubelet);
                    }
                    None => {
                        self.ctx.emit(APIUpdatePodFromKubelet { pod_uid: pod.metadata.uid, new_phase: PodPhase::Pending, node_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
                        dp_api_server!("{:.12} api_server INNER APIUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?} NOT IN ROUTE", self.ctx.time(), pod.metadata.uid, node_uid, new_phase);
                    }
                }
            }
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid} => {
                dp_api_server!("{:.12} api_server APIUpdatePodFromKubelet pod_uid:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod_uid, node_uid, new_phase);

                self.ctx.emit(APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIAddPod { pod } => {
                dp_api_server!("{:.12} api_server APIAddPod pod:{:?}", self.ctx.time(), pod);

                self.ctx.emit(APIAddPod { pod }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIAddNode { kubelet_sim_id, node } => {
                dp_api_server!("{:.12} api_server APIAddNode node:{:?}", self.ctx.time(), node);

                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);
                self.ctx.emit(APIAddNode { kubelet_sim_id, node }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIRemoveNode { node_uid } => {
                dp_api_server!("{:.12} api_server APIRemoveNode node:{:?}", self.ctx.time(), node_uid);

                match self.kubelets.remove(&node_uid) {
                    Some(kubelet_sim_id) => {
                        self.ctx.emit(APIRemoveNode { node_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
                        self.ctx.emit(APIRemoveNode { node_uid }, kubelet_sim_id, self.cluster_state.borrow().network_delays.api2kubelet);
                    }
                    None => {
                        dp_api_server!("{:.12} api_server INNER APIRemoveNode node:{:?} NOT IN ROUTE", self.ctx.time(), node_uid);
                    }
                }
            }
            APIPostCAMetrics { insufficient_resources_pending, requests, node_info } => {
                dp_api_server!("{:.12} api_server APIPostCAMetrics insufficient_resources_pending:{:?} requests:{:?}", self.ctx.time(), insufficient_resources_pending, requests);

                self.ctx.emit(
                    APIPostCAMetrics { insufficient_resources_pending, requests, node_info },
                    self.ca_sim_id, self.cluster_state.borrow().network_delays.api2ca
                );
            }
            APIGetCAMetrics { node_list } => {
                dp_api_server!("{:.12} api_server APIGetCAMetrics", self.ctx.time());
                self.ctx.emit(APIGetCAMetrics { node_list }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APICommitCANodeRemove { node_uid } => {
                dp_api_server!("{:.12} ca APICommitCANodeRemove node_uid:{:?}", self.ctx.time(), node_uid);
                self.ctx.emit(APICommitCANodeRemove { node_uid }, self.ca_sim_id, self.cluster_state.borrow().network_delays.api2ca);
            }
        });
    }
}
