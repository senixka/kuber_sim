use crate::api_server::events::*;
use crate::common_imports::dsc;
use crate::dp_api_server;
use crate::objects::pod::PodPhase;
use crate::simulation::init_config::InitConfig;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The component of the Kubernetes responsible for the interactions between the other components.
pub struct APIServer {
    /// DSLab-Core simulation context of API-Server.
    pub ctx: dsc::SimulationContext,

    /// Scheduler simulation DSlab-Core Id.
    scheduler_sim_id: dsc::Id,
    /// Cluster-Autoscaler simulation DSlab-Core Id.
    ca_sim_id: Option<dsc::Id>,
    /// Horizontal-Pod-Autoscaler simulation DSlab-Core Id.
    hpa_sim_id: Option<dsc::Id>,
    /// Vertical-Pod-Autoscaler simulation DSlab-Core Id.
    vpa_sim_id: Option<dsc::Id>,

    /// Configuration constants of components in simulation.
    init_config: Rc<RefCell<InitConfig>>,

    /// Which node is managed by which kubelet.
    kubelets: HashMap<u64, dsc::Id>, // HashMap<node_uid, kubelet_sim_id>
    /// Which pod belongs to which pod group.
    pod2group: HashMap<u64, u64>, // HashMap<pod_uid, group_uid>
}

impl APIServer {
    pub fn new(ctx: dsc::SimulationContext, init_config: Rc<RefCell<InitConfig>>) -> Self {
        Self {
            ctx,
            scheduler_sim_id: dsc::Id::MAX,
            ca_sim_id: None,
            hpa_sim_id: None,
            vpa_sim_id: None,
            init_config,
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
        self.ca_sim_id = ca_sim_id;
        self.vpa_sim_id = vpa_sim_id;
        self.hpa_sim_id = hpa_sim_id;
    }

    fn notify_hpa_and_vpa<T: dsc::EventData + Clone>(&self, event: T) {
        // Notify HPA
        if self.hpa_sim_id.is_some() {
            self.ctx.emit(
                event.clone(),
                self.hpa_sim_id.unwrap(),
                self.init_config.borrow().network_delays.api2hpa,
            );
        }

        // Notify VPA
        if self.vpa_sim_id.is_some() {
            self.ctx.emit(
                event,
                self.vpa_sim_id.unwrap(),
                self.init_config.borrow().network_delays.api2vpa,
            );
        }
    }

    fn notify_scheduler<T: dsc::EventData>(&self, event: T) {
        self.ctx.emit(
            event,
            self.scheduler_sim_id,
            self.init_config.borrow().network_delays.api2scheduler,
        );
    }

    fn notify_kubelet<T: dsc::EventData>(&self, event: T, kubelet_sim_id: dsc::Id) {
        self.ctx.emit(
            event,
            kubelet_sim_id,
            self.init_config.borrow().network_delays.api2kubelet,
        );
    }

    fn notify_ca<T: dsc::EventData>(&self, event: T) {
        if self.ca_sim_id.is_some() {
            self.ctx.emit(
                event,
                self.ca_sim_id.unwrap(),
                self.init_config.borrow().network_delays.api2ca,
            );
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
                    "{:.3} api_server EventUpdatePodFromScheduler pod_uid:{:?} preempt_uids:{:?} node_uid:{:?} new_phase:{:?}",
                    self.ctx.time(), pod_uid, preempt_uids, node_uid, new_phase
                );

                // Get kubelet sim_id
                let to = self.kubelets.get(&node_uid);
                match to {
                    Some(&kubelet_id) => {
                        // If kubelet turned on (routing exists) -> Notify kubelet
                        self.notify_kubelet(
                            EventUpdatePodFromScheduler {
                                pod_uid,
                                pod,
                                preempt_uids,
                                new_phase,
                                node_uid,
                            },
                            kubelet_id,
                        );
                    }
                    None => {
                        // If kubelet turned off (not in routing) -> Notify scheduler returning this pod
                        self.notify_scheduler(EventPodUpdateToScheduler {
                            pod_uid,
                            current_phase: PodPhase::Pending,
                        });

                        dp_api_server!(
                            "{:.3} api_server INNER EventUpdatePodFromScheduler pod_uid:{:?} node_uid:{:?} new_phase:{:?} NOT IN ROUTE",
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
                    "{:.3} api_server EventUpdatePodFromKubelet pod_uid:{:?} current_phase:{:?} current_cpu:{:?} current_memory:{:?}",
                    self.ctx.time(), pod_uid, current_phase, current_cpu, current_memory
                );

                // Notify scheduler if pod not in Running phase
                if current_phase != PodPhase::Running {
                    self.notify_scheduler(EventPodUpdateToScheduler {
                        pod_uid,
                        current_phase: current_phase.clone(),
                    });
                }

                // Post pod metrics to HPA and VPA
                self.notify_hpa_and_vpa(EventPodMetricsPost {
                    group_uid: *self.pod2group.get(&pod_uid).unwrap(),
                    pod_uid,
                    current_phase: current_phase.clone(),
                    current_cpu,
                    current_memory,
                });
            }

            EventAddPod { pod } => {
                dp_api_server!("{:.3} api_server EventAddPod pod:{:?}", self.ctx.time(), pod);

                // Check that pod was properly prepared
                assert_ne!(pod.metadata.uid, 0);
                assert_ne!(pod.metadata.group_uid, 0);

                // Create mapping pod_uid to group_uid
                self.pod2group.insert(pod.metadata.uid, pod.metadata.group_uid);

                // Notify HPA and VPA
                self.notify_hpa_and_vpa(EventAddPod { pod: pod.clone() });
                // Notify scheduler
                self.notify_scheduler(EventAddPod { pod });
            }

            EvenAddPodGroup { pod_group } => {
                dp_api_server!(
                    "{:.3} api_server EvenAddPodGroup pod_group:{:?}",
                    self.ctx.time(),
                    pod_group
                );

                // Notify HPA and VPA
                self.notify_hpa_and_vpa(EvenAddPodGroup {
                    pod_group: pod_group.clone(),
                });

                // Emit AddPod events
                for _ in 0..pod_group.pod_count {
                    // Get pod template
                    let mut pod = pod_group.pod.clone();
                    // Prepare pod template
                    pod.prepare(pod_group.group_uid);

                    // Emit new pod
                    self.ctx.emit_self_now(EventAddPod { pod: pod.clone() });
                }
            }

            EventRemovePod { pod_uid } => {
                dp_api_server!("{:.3} api_server EventRemovePod pod_uid:{:?}", self.ctx.time(), pod_uid);

                // Notify scheduler
                self.notify_scheduler(EventRemovePod { pod_uid });
                // Here, it is enough to notify the scheduler. Scheduler will notify kubelets.
            }

            EventRemovePodGroup { group_uid } => {
                dp_api_server!(
                    "{:.3} api_server EventRemovePodGroup group_uid:{:?}",
                    self.ctx.time(),
                    group_uid
                );

                // Notify HPA and VPA
                self.notify_hpa_and_vpa(EventRemovePodGroup { group_uid });
                // Notify Scheduler
                self.notify_scheduler(EventRemovePodGroup { group_uid });
                self.ctx.emit(
                    EventRemovePodGroup { group_uid },
                    self.scheduler_sim_id,
                    self.init_config.borrow().network_delays.max_delay * 5.0,
                );
                // Here, it is enough to notify the scheduler. Scheduler will notify kubelets.
            }

            EventAddNode { kubelet_sim_id, node } => {
                dp_api_server!("{:.3} api_server EventAddNode node:{:?}", self.ctx.time(), node);

                // Check that node was properly prepared
                assert_ne!(node.metadata.uid, 0);
                assert_ne!(node.metadata.group_uid, 0);

                // Add routing [node_uid] -> [kubelet_sim_id]
                self.kubelets.insert(node.metadata.uid, kubelet_sim_id);

                // Notify scheduler
                self.notify_scheduler(EventAddNode { kubelet_sim_id, node });
            }

            EventRemoveNode { node_uid } => {
                dp_api_server!("{:.3} api_server EventRemoveNode node:{:?}", self.ctx.time(), node_uid);

                // Remove node_uid from routing
                match self.kubelets.remove(&node_uid) {
                    Some(kubelet_sim_id) => {
                        // Notify scheduler
                        self.notify_scheduler(EventRemoveNode { node_uid });
                        // Notify kubelet
                        self.notify_kubelet(EventRemoveNode { node_uid }, kubelet_sim_id)
                    }
                    None => {
                        dp_api_server!(
                            "{:.3} api_server INNER EventRemoveNode node:{:?} NOT IN ROUTE",
                            self.ctx.time(),
                            node_uid
                        );
                    }
                }
            }

            EventRemoveNodeAck { node_uid } => {
                dp_api_server!(
                    "{:.3} api_server EventRemoveNodeAck node_uid:{:?}",
                    self.ctx.time(),
                    node_uid
                );

                // Notify CA
                self.notify_ca(EventRemoveNodeAck { node_uid });
            }
        });
    }
}
