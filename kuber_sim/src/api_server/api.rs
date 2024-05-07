use crate::my_imports::*;

pub struct APIServer {
    pub ctx: dsc::SimulationContext,
    init_config: Rc<RefCell<InitConfig>>,

    scheduler_sim_id: dsc::Id,
    ca_sim_id: dsc::Id,
    hpa_sim_id: dsc::Id,
    vpa_sim_id: dsc::Id,

    is_ca_enabled: bool,
    is_hpa_enabled: bool,
    is_vpa_enabled: bool,

    // Inner state
    kubelets: HashMap<u64, dsc::Id>, // HashMap<node_uid, kubelet_sim_id>
    pod2group: HashMap<u64, u64>,    // HashMap<pod_uid, group_uid>
}

impl APIServer {
    pub fn new(ctx: dsc::SimulationContext, init_config: Rc<RefCell<InitConfig>>) -> Self {
        Self {
            ctx,
            init_config,

            scheduler_sim_id: dsc::Id::MAX,
            ca_sim_id: dsc::Id::MAX,
            hpa_sim_id: dsc::Id::MAX,
            vpa_sim_id: dsc::Id::MAX,

            is_ca_enabled: false,
            is_hpa_enabled: false,
            is_vpa_enabled: false,

            kubelets: HashMap::new(),
            pod2group: HashMap::new(),
        }
    }

    pub fn prepare(
        &mut self,
        scheduler_sim_id: dsc::Id,
        ca_sim_id: Option<dsc::Id>,
        hpa_sim_id: Option<dsc::Id>,
        vpa_sim_id: Option<dsc::Id>,
    ) {
        self.scheduler_sim_id = scheduler_sim_id;

        // Init CA info
        match ca_sim_id {
            Some(ca_id) => {
                self.ca_sim_id = ca_id;
                self.is_ca_enabled = true;
            }
            None => {
                self.ca_sim_id = dsc::Id::MAX;
                self.is_ca_enabled = false;
            }
        }

        // Init HPA info
        match hpa_sim_id {
            Some(hpa_id) => {
                self.hpa_sim_id = hpa_id;
                self.is_hpa_enabled = true;
            }
            None => {
                self.hpa_sim_id = dsc::Id::MAX;
                self.is_hpa_enabled = false;
            }
        }

        // Init VPA info
        match vpa_sim_id {
            Some(vpa_id) => {
                self.vpa_sim_id = vpa_id;
                self.is_vpa_enabled = true;
            }
            None => {
                self.vpa_sim_id = dsc::Id::MAX;
                self.is_vpa_enabled = false;
            }
        }
    }
}

impl dsc::EventHandler for APIServer {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventUpdatePodFromScheduler {
                pod_uid,
                pod,
                preempt_uids,
                new_phase,
                node_uid,
            } => {
                dp_api_server!(
                    "{:.12} api_server EventUpdatePodFromScheduler pod_uid:{:?} preempt_uids:{:?} node_uid:{:?} new_phase:{:?}",
                    self.ctx.time(), pod_uid, preempt_uids, node_uid, new_phase
                );

                // Get kubelet sim_id
                let to = self.kubelets.get(&node_uid);
                match to {
                    Some(&kubelet_id) => {
                        // If kubelet turned on (routing exists) -> Notify kubelet
                        self.ctx.emit(
                            EventUpdatePodFromScheduler {
                                pod_uid,
                                pod,
                                preempt_uids,
                                new_phase,
                                node_uid,
                            },
                            kubelet_id,
                            self.init_config.borrow().network_delays.api2kubelet,
                        );
                    }
                    None => {
                        // If kubelet turned off (not in routing) -> Notify scheduler returning this pod
                        self.ctx.emit(
                            EventPodUpdateToScheduler {
                                pod_uid,
                                current_phase: PodPhase::Pending,
                            },
                            self.scheduler_sim_id,
                            self.init_config.borrow().network_delays.api2scheduler,
                        );
                        dp_api_server!(
                            "{:.12} api_server INNER EventUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?} NOT IN ROUTE",
                            self.ctx.time(), pod_uid, node_uid, new_phase
                        );
                    }
                }
            }

            EventPodUpdateFromKubelet {
                pod_uid,
                current_phase,
                current_cpu,
                current_memory,
            } => {
                dp_api_server!(
                    "{:.12} api_server EventUpdatePodFromKubelet pod_uid:{:?} current_phase:{:?} current_cpu:{:?} current_memory:{:?}",
                    self.ctx.time(), pod_uid, current_phase, current_cpu, current_memory
                );

                // Notify scheduler if pod not in Running phase
                if current_phase != PodPhase::Running {
                    self.ctx.emit(
                        EventPodUpdateToScheduler {
                            pod_uid,
                            current_phase: current_phase.clone(),
                        },
                        self.scheduler_sim_id,
                        self.init_config.borrow().network_delays.api2scheduler,
                    );
                }

                // Post pod metrics to HPA if HPA enabled
                if self.is_hpa_enabled {
                    self.ctx.emit(
                        EventHPAPodMetricsPost {
                            group_uid: *self.pod2group.get(&pod_uid).unwrap(),
                            pod_uid,
                            current_phase: current_phase.clone(),
                            current_cpu,
                            current_memory,
                        },
                        self.hpa_sim_id,
                        self.init_config.borrow().network_delays.api2hpa,
                    );
                }

                // Post pod metrics to VPA if VPA enabled
                if self.is_vpa_enabled {
                    self.ctx.emit(
                        EventVPAPodMetricsPost {
                            group_uid: *self.pod2group.get(&pod_uid).unwrap(),
                            pod_uid,
                            current_phase,
                            current_cpu,
                            current_memory,
                        },
                        self.vpa_sim_id,
                        self.init_config.borrow().network_delays.api2vpa,
                    );
                }
            }

            EventRemovePod { pod_uid } => {
                dp_api_server!(
                    "{:.12} api_server EventRemovePod pod_uid:{:?}",
                    self.ctx.time(),
                    pod_uid
                );

                // Notify scheduler
                self.ctx.emit(
                    EventRemovePod { pod_uid },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler,
                );

                // Here we only have to notify scheduler. Scheduler will notify kubelet.
            }

            EventRemovePodGroup { group_uid } => {
                dp_api_server!(
                    "{:.12} api_server EventRemovePodGroup group_uid:{:?}",
                    self.ctx.time(),
                    group_uid
                );

                // Notify HPA
                if self.is_hpa_enabled {
                    self.ctx.emit(
                        EventRemovePodGroup { group_uid },
                        self.hpa_sim_id,
                        self.init_config.borrow().network_delays.api2hpa,
                    );
                }

                // Notify VPA
                if self.is_vpa_enabled {
                    self.ctx.emit(
                        EventRemovePodGroup { group_uid },
                        self.vpa_sim_id,
                        self.init_config.borrow().network_delays.api2vpa,
                    );
                }

                // Notify scheduler
                self.ctx.emit(
                    EventRemovePodGroup { group_uid },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler,
                );
                // Here we only have to notify scheduler. Scheduler will notify kubelets.

                // Notify scheduler again with delay
                self.ctx.emit(
                    EventRemovePodGroup { group_uid },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler
                        + self.init_config.borrow().network_delays.max_delay * 4.0,
                );
            }

            EvenAddPodGroup { pod_group } => {
                // Update local caches

                // Notify HPA
                if self.is_hpa_enabled && pod_group.hpa_profile.is_some() {
                    self.ctx.emit(
                        EvenAddPodGroup {
                            pod_group: pod_group.clone(),
                        },
                        self.hpa_sim_id,
                        self.init_config.borrow().network_delays.api2hpa,
                    );
                }

                // Notify VPA
                if self.is_vpa_enabled && pod_group.vpa_profile.is_some() {
                    self.ctx.emit(
                        EvenAddPodGroup {
                            pod_group: pod_group.clone(),
                        },
                        self.vpa_sim_id,
                        self.init_config.borrow().network_delays.api2vpa,
                    );
                }

                // Emit AddPod events
                for _ in 0..pod_group.pod_count {
                    // Get pod group template
                    let mut pod = pod_group.pod.clone();
                    // Prepare pod group from template
                    pod.prepare(pod_group.group_uid);

                    self.ctx.emit_self_now(EventAddPod { pod: pod.clone() });
                }
            }

            EventAddPod { pod } => {
                dp_api_server!("{:.12} api_server EventAddPod pod:{:?}", self.ctx.time(), pod);

                // Check that pod was properly prepared
                assert_ne!(pod.metadata.uid, 0);
                assert_ne!(pod.metadata.group_uid, 0);

                // Create mapping pod_uid to group_uid
                self.pod2group.insert(pod.metadata.uid, pod.metadata.group_uid);

                // Notify HPA if HPA enabled and pod contains HPA profile
                if self.is_hpa_enabled {
                    self.ctx.emit(
                        EventAddPod { pod: pod.clone() },
                        self.hpa_sim_id,
                        self.init_config.borrow().network_delays.api2hpa,
                    );
                }

                // Notify VPA if VPA enabled and pod contains VPA profile
                if self.is_vpa_enabled {
                    self.ctx.emit(
                        EventAddPod { pod: pod.clone() },
                        self.vpa_sim_id,
                        self.init_config.borrow().network_delays.api2vpa,
                    );
                }

                // Notify scheduler
                self.ctx.emit(
                    EventAddPod { pod },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler,
                );
            }

            EventAddNode { kubelet_sim_id, node } => {
                dp_api_server!("{:.12} api_server EventAddNode node:{:?}", self.ctx.time(), node);

                // Check that node was properly prepared
                assert_ne!(node.metadata.uid, 0);
                assert_ne!(node.metadata.group_uid, 0);

                // Add routing [node_uid] -> [kubelet_sim_id]
                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);

                // Notify scheduler
                self.ctx.emit(
                    EventAddNode { kubelet_sim_id, node },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler,
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
                            self.init_config.borrow().network_delays.api2scheduler,
                        );

                        // Notify kubelet
                        self.ctx.emit(
                            EventRemoveNode { node_uid },
                            kubelet_sim_id,
                            self.init_config.borrow().network_delays.api2kubelet,
                        );
                    }
                    None => {
                        dp_api_server!(
                            "{:.12} api_server INNER EventRemoveNode node:{:?} NOT IN ROUTE",
                            self.ctx.time(),
                            node_uid
                        );
                    }
                }
            }

            EventRemoveNodeAck { node_uid } => {
                dp_api_server!(
                    "{:.12} api_server EventRemoveNodeAck node_uid:{:?}",
                    self.ctx.time(),
                    node_uid
                );

                // Notify CA
                self.ctx.emit(
                    EventRemoveNodeAck { node_uid },
                    self.ca_sim_id,
                    self.init_config.borrow().network_delays.api2ca,
                );
            }

            EventGetCAMetrics {
                used_nodes,
                available_nodes,
            } => {
                dp_api_server!(
                    "{:.12} api_server EventGetCAMetrics used_nodes:{:?} available_nodes:{:?}",
                    self.ctx.time(),
                    used_nodes,
                    available_nodes
                );

                // Send metrics request to scheduler
                self.ctx.emit(
                    EventGetCAMetrics {
                        used_nodes,
                        available_nodes,
                    },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.api2scheduler,
                );
            }

            EventPostCAMetrics {
                pending_pod_count,
                used_nodes_utilization,
                may_help,
            } => {
                dp_api_server!(
                    "{:.12} api_server EventPostCAMetrics pending_pod_count:{:?} used_nodes_utilization:{:?} may_help:{:?}",
                    self.ctx.time(), pending_pod_count, used_nodes_utilization, may_help
                );

                // Send metrics to CA
                self.ctx.emit(
                    EventPostCAMetrics {
                        pending_pod_count,
                        used_nodes_utilization,
                        may_help,
                    },
                    self.ca_sim_id,
                    self.init_config.borrow().network_delays.api2ca,
                );
            }
        });
    }
}
