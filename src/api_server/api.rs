use crate::my_imports::*;


pub struct APIServer {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    scheduler_sim_id: dsc::Id,
    ca_sim_id: dsc::Id,
    hpa_sim_id: dsc::Id,

    // subscriptions: HashMap<, Vec<dsc::Id>>,

    // ############## ETCD ##############
    kubelets: HashMap<u64, dsc::Id>,                            // HashMap<node_uid, kubelet_sim_id>

    pod2group: HashMap<u64, u64>,                               // HashMap<pod_uid, group_uid>
    group_members: HashMap<u64, HashSet<u64>>,                  // HashMap<group_uid, HashSet<pod_uid>
    pod_metrics: HashMap<u64, (PodPhase, f64, f64)>             // HashMap<pod_uid, current_(PodPhase, cpu, memory)>
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
            kubelets: HashMap::new(),
            pod2group: HashMap::new(),
            group_members: HashMap::new(),
            pod_metrics: HashMap::new(),
        }
    }

    pub fn prepare(&mut self, scheduler_sim_id: dsc::Id, ca_sim_id: dsc::Id, hpa_sim_id: dsc::Id) {
        self.scheduler_sim_id = scheduler_sim_id;
        self.ca_sim_id = ca_sim_id;
        self.hpa_sim_id = hpa_sim_id;
    }

    // pub fn subscribe(&mut self, event: dsc::Event, sim_id: dsc::Id) {
    //     self.subscriptions.entry(event).or_default().push(sim_id);
    // }
}


impl dsc::EventHandler for APIServer {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventUpdatePodFromScheduler { pod_uid , pod, preempt_uids, new_phase, node_uid } => {
                dp_api_server!("{:.12} api_server EventUpdatePodFromScheduler pod_uid:{:?} preempt_uids:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod_uid, preempt_uids, node_uid, new_phase);

                // Get kubelet sim_id
                let to = self.kubelets.get(&node_uid);
                match to {
                    Some(&kubelet_id) => {
                        // If kubelet turned on (routing exists) -> Notify kubelet
                        self.ctx.emit(
                            EventUpdatePodFromScheduler { pod_uid, pod, preempt_uids, new_phase, node_uid },
                            kubelet_id,
                            self.cluster_state.borrow().network_delays.api2kubelet
                        );
                    }
                    None => {
                        // If kubelet turned off (not in routing) -> Notify scheduler returning this pod
                        self.ctx.emit(
                            EventPodUpdateToScheduler { pod_uid, current_phase: PodPhase::Pending },
                            self.scheduler_sim_id,
                            self.cluster_state.borrow().network_delays.api2scheduler
                        );
                        dp_api_server!("{:.12} api_server INNER EventUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?} NOT IN ROUTE", self.ctx.time(), pod_uid, node_uid, new_phase);
                    }
                }
            }

            EventPodUpdateFromKubelet { pod_uid, current_phase, current_cpu, current_memory} => {
                dp_api_server!("{:.12} api_server EventUpdatePodFromKubelet pod_uid:{:?} current_phase:{:?} current_cpu:{:?} current_memory:{:?}", self.ctx.time(), pod_uid, current_phase, current_cpu, current_memory);

                // Update pod's metrics
                self.pod_metrics.insert(pod_uid, (current_phase.clone(), current_cpu, current_memory));

                // Notify scheduler if pod not Running
                if current_phase != PodPhase::Running {
                    self.ctx.emit(
                        EventPodUpdateToScheduler { pod_uid, current_phase },
                        self.scheduler_sim_id,
                        self.cluster_state.borrow().network_delays.api2scheduler
                    );
                }
            }

            EventAddPod { pod } => {
                dp_api_server!("{:.12} api_server EventAddPod pod:{:?}", self.ctx.time(), pod);

                // Bind pod_uid to pod_group
                self.pod2group.insert(pod.metadata.uid, pod.metadata.group_uid);

                // Notify scheduler
                self.ctx.emit(
                    EventAddPod { pod },
                    self.scheduler_sim_id,
                    self.cluster_state.borrow().network_delays.api2scheduler
                );
            }

            EventAddNode { kubelet_sim_id, node } => {
                dp_api_server!("{:.12} api_server EventAddNode node:{:?}", self.ctx.time(), node);

                // Add routing [node_uid] -> [kubelet_sim_id]
                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);

                // Notify scheduler
                self.ctx.emit(
                    EventAddNode { kubelet_sim_id, node },
                    self.scheduler_sim_id,
                    self.cluster_state.borrow().network_delays.api2scheduler
                );
            }

            EventRemoveNode { node_uid } => {
                dp_api_server!("{:.12} api_server EventRemoveNode node:{:?}", self.ctx.time(), node_uid);

                // Remove node_uid from routing
                match self.kubelets.remove(&node_uid) {
                    Some(kubelet_sim_id) => {
                        // Notify scheduler
                        self.ctx.emit(
                            EventRemoveNode { node_uid },
                            self.scheduler_sim_id,
                            self.cluster_state.borrow().network_delays.api2scheduler
                        );

                        // Notify kubelet
                        self.ctx.emit(
                            EventRemoveNode { node_uid },
                            kubelet_sim_id,
                            self.cluster_state.borrow().network_delays.api2kubelet
                        );
                    }
                    None => {
                        dp_api_server!("{:.12} api_server INNER EventRemoveNode node:{:?} NOT IN ROUTE", self.ctx.time(), node_uid);
                    }
                }
            }

            EventRemoveNodeAck { node_uid } => {
                dp_api_server!("{:.12} api_server EventRemoveNodeAck node_uid:{:?}", self.ctx.time(), node_uid);

                // Notify CA
                self.ctx.emit(
                    EventRemoveNodeAck { node_uid },
                    self.ca_sim_id,
                    self.cluster_state.borrow().network_delays.api2ca
                );
            }

            EventGetCAMetrics { used_nodes, available_nodes } => {
                dp_api_server!("{:.12} api_server EventGetCAMetrics used_nodes:{:?} available_nodes:{:?}", self.ctx.time(), used_nodes, available_nodes);

                // Send metrics request to scheduler
                self.ctx.emit(
                    EventGetCAMetrics { used_nodes, available_nodes },
                    self.scheduler_sim_id,
                    self.cluster_state.borrow().network_delays.api2scheduler
                );
            }

            EventPostCAMetrics { pending_pod_count, used_nodes_utilization, may_help } => {
                dp_api_server!("{:.12} api_server EventPostCAMetrics pending_pod_count:{:?} used_nodes_utilization:{:?} may_help:{:?}", self.ctx.time(), pending_pod_count, used_nodes_utilization, may_help);

                // Send metrics to CA
                self.ctx.emit(
                    EventPostCAMetrics { pending_pod_count, used_nodes_utilization, may_help },
                    self.ca_sim_id,
                    self.cluster_state.borrow().network_delays.api2ca
                );
            }

            EventGetHPAMetrics { pod_groups } => {
                dp_api_server!("{:.12} api_server EventGetHPAMetrics pod_groups:{:?}", self.ctx.time(), pod_groups);

                // let mut group_utilization = Vec::with_capacity(pod_groups.len());
                // for group_uid in pod_groups {
                //     match self.pod_consumptions.get(&group_uid) {
                //         Some(pods) => {
                //             // Sum all group consumed resources
                //             let (mut group_cpu, mut group_memory): (f64, f64) = (0.0, 0.0);
                //             for (_, &(pod_cpu, pod_memory)) in pods {
                //                 group_cpu += pod_cpu;
                //                 group_memory += pod_memory;
                //             }
                //
                //             // Add group utilization
                //             group_utilization.push((
                //                 pods.len() as u64,
                //                 group_cpu / pods.len() as f64,
                //                 group_memory / pods.len() as f64
                //             ));
                //         }
                //         None => {
                //             // Add zero utilization
                //             group_utilization.push((0, 0.0, 0.0));
                //         }
                //     }
                // }
                //
                // // Send metrics to HPA
                // self.ctx.emit(
                //     EventPostHPAMetrics { group_utilization },
                //     self.hpa_sim_id,
                //     self.cluster_state.borrow().network_delays.api2hpa
                // );
            }

            // EventRemoveAnyPodInGroup { group_uid } => {
            //     dp_api_server!("{:.12} api_server EventRemoveAnyPodInGroup group_uid:{:?}", self.ctx.time(), group_uid);
            //
            //     // Try to locate group pods
            //     match self.pod_consumptions.get_mut(&group_uid) {
            //         Some(index) => {
            //             // Try to get any(first) pod from group
            //             match index.iter().next() {
            //                 Some((&pod_uid, _)) => {
            //                     // Candidate to removal found. Notify scheduler
            //                     self.ctx.emit(
            //                         EventRemovePod { pod_uid },
            //                         self.scheduler_sim_id,
            //                         self.cluster_state.borrow().network_delays.api2scheduler
            //                     );
            //                 }
            //                 None => {} // Group is empty, nothing to remove
            //             }
            //         }
            //         None => {} // Group not found, nothing to remove
            //     }
            // }
        });
    }
}
