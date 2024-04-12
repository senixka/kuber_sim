use std::collections::BinaryHeap;
use crate::simulation::config::ClusterState;
use crate::simulation::monitoring::Monitoring;
use super::super::my_imports::*;
use super::active_queue::*;
use super::backoff_queue::*;


pub struct Scheduler<ActiveQCmp, BackOffQ> {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    self_update_enabled: bool,

    // Cache
    pods: HashMap<u64, Pod>,
    nodes: HashMap<u64, Node>,

    // Queues
    active_queue: BinaryHeap<ActiveQCmp>,
    backoff_queue: BackOffQ,
    failed_attempts: HashMap<u64, u64>,
}

pub fn place_pod(pod: &Pod, node: &mut Node) {
    node.spec.available_cpu -= pod.spec.request_cpu;
    node.spec.available_memory -= pod.spec.request_memory;
}

pub fn unplace_pod(pod: &Pod, node: &mut Node) {
    node.spec.available_cpu += pod.spec.request_cpu;
    node.spec.available_memory += pod.spec.request_memory;
}

pub fn is_pod_placeable(pod: &Pod, node: &Node) -> bool {
    return pod.spec.request_cpu <= node.spec.available_cpu
        && pod.spec.request_memory <= node.spec.available_memory;
}

impl<ActiveQCmp: TraitActiveQCmp, BackOffQ: TraitBackOffQ> Scheduler<ActiveQCmp, BackOffQ> {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>, monitoring: Rc<RefCell<Monitoring>>) -> Scheduler<ActiveQCmp, BackOffQ> {
        Self {
            ctx,
            cluster_state,
            api_sim_id: dsc::Id::MAX,
            monitoring,
            pods: HashMap::new(),
            nodes: HashMap::new(),
            self_update_enabled: false,
            active_queue: BinaryHeap::new(),
            failed_attempts: HashMap::new(),
            backoff_queue: BackOffQ::new(1.0, 10.0),
        }
    }

    pub fn test(&mut self, pod: Pod) -> Pod{
        self.active_queue.push(ActiveQCmp::wrap(pod));
        return self.active_queue.pop().unwrap().inner();
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.api_sim_id, dsc::Id::MAX);
    }

    pub fn self_update_on(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(APISchedulerSelfUpdate {}, self.cluster_state.borrow().constants.scheduler_self_update_period);
        }
    }

    pub fn schedule(&mut self) {
        for (_, pod) in self.pods.iter_mut() {
            if pod.status.phase != PodPhase::Pending {
                continue;
            }
            // println!("Scheduler Pod_{0} is Pending", pod_uid);

            for (node_uid, node) in self.nodes.iter_mut() {
                // println!("Try node_{0}", node.metadata.uid);
                if is_pod_placeable(&pod, &node) {
                    place_pod(&pod, node);
                    pod.status.node_uid = Some(node.metadata.uid);
                    pod.status.phase = PodPhase::Running;

                    let data = APIUpdatePodFromScheduler {
                        pod: pod.clone(),
                        new_phase: PodPhase::Running,
                        node_uid: node_uid.clone(),
                    };

                    // println!("{:?} Scheduler Pod_{:?} placed to Node_{:?} artime: {:?}", self.ctx.time(), pod.metadata.uid, node.metadata.uid, pod.spec.arrival_time);
                    self.monitoring.borrow_mut().scheduler_on_pod_placed(&pod);
                    self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.scheduler2api);
                    break;
                }
            }
        }
    }
}

impl<ActiveQCmp: TraitActiveQCmp, BackOffQ: TraitBackOffQ> dsc::EventHandler for Scheduler<ActiveQCmp, BackOffQ> {
    fn on(&mut self, event: dsc::Event) {
        // println!("Scheduler EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid } => {
                // println!("{:?} Scheduler <Update Pod From Kubelet> pod_{:?} new_phase: {:?}", self.ctx.time(), pod_uid, new_phase);

                if new_phase == PodPhase::Succeeded || new_phase == PodPhase::Failed {
                    let pod = self.pods.get_mut(&pod_uid).unwrap();

                    unplace_pod(pod, self.nodes.get_mut(&node_uid).unwrap());
                    self.monitoring.borrow_mut().scheduler_on_pod_unplaced(&pod);

                    self.pods.remove(&pod_uid);
                }
                if new_phase == PodPhase::Pending {
                    let pod = self.pods.get_mut(&pod_uid).unwrap();
                    pod.status.phase = new_phase;

                    self.monitoring.borrow_mut().scheduler_on_pod_unplaced(&pod);
                    unplace_pod(self.pods.get_mut(&pod_uid).unwrap(), self.nodes.get_mut(&node_uid).unwrap());
                }
                self.schedule();
                self.self_update_on();
            }
            APIAddPod { pod } => {
                // println!("Scheduler <Add Pod>");

                self.pods.insert(pod.metadata.uid, pod);
                self.schedule();
                self.self_update_on();
            }
            APIAddNode { kubelet_sim_id: _ , node } => {
                // println!("Scheduler <Add Kubelet>");
                self.monitoring.borrow_mut().scheduler_on_node_added(&node);
                self.nodes.insert(node.metadata.uid, node);
            }
            APISchedulerSelfUpdate { } => {
                // println!("Scheduler <Self Update>");
                self.schedule();

                if self.pods.len() > 0 {
                    self.ctx.emit_self(APISchedulerSelfUpdate{}, self.cluster_state.borrow().constants.scheduler_self_update_period);
                }
            }
        });
        // println!("Scheduler EventHandler <------");
    }
}
