use crate::my_imports::*;


pub struct APIServer {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    scheduler_sim_id: dsc::Id,
    ca_sim_id: dsc::Id,
    hpa_sim_id: dsc::Id,

    // subscriptions: HashMap<APIServerEvent, Vec<dsc::Id>>,

    // ############## ETCD ##############
    // pods: HashMap<u64, Pod>, // Pod uid -> Pod
    pod2group: HashMap<u64, u64>, // pod_uid -> group_uid
    kubelets: HashMap<u64, dsc::Id>, // node_uid -> kubelet_sim_id
    pod_consumptions: HashMap<u64, HashMap<u64, (f64, f64)>>, // group_uid -> HashMap<node_uid -> (current_cpu, current_memory)>
}


impl APIServer {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>) -> Self {
        Self {
            ctx,
            cluster_state,
            scheduler_sim_id: dsc::Id::MAX,
            ca_sim_id: dsc::Id::MAX,
            hpa_sim_id: dsc::Id::MAX,
            // subscriptions: HashMap::new(),
            // pods: HashMap::new(),
            kubelets: HashMap::new(),
            pod_consumptions: HashMap::new(),
            pod2group: HashMap::new(),
        }
    }

    pub fn prepare(&mut self, scheduler_sim_id: dsc::Id, ca_sim_id: dsc::Id, hpa_sim_id: dsc::Id) {
        self.scheduler_sim_id = scheduler_sim_id;
        self.ca_sim_id = ca_sim_id;
        self.hpa_sim_id = hpa_sim_id;
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

                let group_uid = self.pod2group.get(&pod_uid).unwrap();
                match self.pod_consumptions.get_mut(group_uid) {
                    Some(index) => {
                        index.remove(&pod_uid);
                    }
                    None => {}
                }
                self.ctx.emit(APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
            }
            APIAddPod { pod } => {
                dp_api_server!("{:.12} api_server APIAddPod pod:{:?}", self.ctx.time(), pod);

                self.pod2group.insert(pod.metadata.uid, pod.metadata.group_uid);
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
                dp_api_server!("{:.12} api_server APICommitCANodeRemove node_uid:{:?}", self.ctx.time(), node_uid);

                self.ctx.emit(APICommitCANodeRemove { node_uid }, self.ca_sim_id, self.cluster_state.borrow().network_delays.api2ca);
            }
            APIUpdatePodMetricsFromKubelet { pod_uid, current_cpu, current_memory } => {
                dp_api_server!("{:.12} api_server APIUpdatePodMetricsFromKubelet pod_uid:{:?} current_cpu:{:?} current_memory:{:?}", self.ctx.time(), pod_uid, current_cpu, current_memory);

                let &group_uid = self.pod2group.get(&pod_uid).unwrap();
                match self.pod_consumptions.get_mut(&group_uid) {
                    Some(index) => {
                        index.insert(pod_uid, (current_cpu, current_memory));
                    }
                    None => {
                        self.pod_consumptions.insert(group_uid, HashMap::from([(pod_uid, (current_cpu, current_memory))]));
                    }
                }
            }
            APIGetHPAMetrics { pod_groups } => {
                dp_api_server!("{:.12} api_server APIGetHPAMetrics pod_groups:{:?}", self.ctx.time(), pod_groups);

                // dp_api_server!("{:?}\n{:?}", self.pod2group, self.pod_consumptions);

                let mut result: Vec<(u64, f64, f64)> = Vec::with_capacity(pod_groups.len());
                for group_uid in pod_groups {
                    match self.pod_consumptions.get(&group_uid) {
                        Some(pods) => {
                            let (mut group_cpu, mut group_memory): (f64, f64) = (0.0, 0.0);
                            for (_, (pod_cpu, pod_memory)) in pods {
                                group_cpu += pod_cpu;
                                group_memory += pod_memory;
                            }
                            result.push((pods.len() as u64, group_cpu / pods.len() as f64, group_memory / pods.len() as f64));
                        }
                        None => {
                            result.push((0, 0.0, 0.0));
                        }
                    }
                }

                self.ctx.emit(APIPostHPAMetrics { pod_groups: result }, self.hpa_sim_id, self.cluster_state.borrow().network_delays.api2hpa);
            }
            APIRemoveAnyPodInGroup { group_uid } => {
                dp_api_server!("{:.12} api_server APIRemoveAnyPodInGroup group_uid:{:?}", self.ctx.time(), group_uid);

                match self.pod_consumptions.get_mut(&group_uid) {
                    // If group is not empty
                    Some(index) => {
                        match index.iter().next() {
                            // If index is not empty
                            Some((&pod_uid, _)) => {
                                self.ctx.emit(APIRemovePod { pod_uid }, self.scheduler_sim_id, self.cluster_state.borrow().network_delays.api2scheduler);
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
            }
        });
    }
}
