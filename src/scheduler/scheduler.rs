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
    running_pods: HashMap<u64, Pod>,
    pending_pods: HashMap<u64, Pod>, // TODO: remove it
    nodes: HashMap<u64, Node>,

    // Queues
    active_queue: BinaryHeap<ActiveQCmp>,
    backoff_queue: BackOffQ,
    failed_attempts: HashMap<u64, u64>,
}

impl<ActiveQCmp: TraitActiveQCmp, BackOffQ: TraitBackOffQ> Scheduler<ActiveQCmp, BackOffQ> {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>, monitoring: Rc<RefCell<Monitoring>>) -> Scheduler<ActiveQCmp, BackOffQ> {
        Self {
            ctx,
            cluster_state,
            api_sim_id: dsc::Id::MAX,
            monitoring,
            running_pods: HashMap::new(),
            pending_pods: HashMap::new(),
            nodes: HashMap::new(),
            self_update_enabled: false,
            active_queue: BinaryHeap::new(),
            failed_attempts: HashMap::new(),
            backoff_queue: BackOffQ::new(1.0, 10.0),
        }
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
        while let Some(wrapper) = self.active_queue.pop() {
            let mut pod = wrapper.inner();
            let pod_uid = pod.metadata.uid;
            let cpu = pod.spec.request_cpu;
            let memory = pod.spec.request_memory;

            let mut assigned_node_uid: Option<u64> = None;
            for (node_uid, node) in self.nodes.iter_mut() {
                if !Scheduler::<ActiveQCmp, BackOffQ>::is_node_consumable(&node, cpu, memory) {
                    continue;
                }
                assigned_node_uid = Some(*node_uid);
                break;
            }

            match assigned_node_uid {
                None => {
                    // Increase failed attempts
                    let attempts = self.failed_attempts.entry(pod_uid).or_default();
                    *attempts += 1;

                    // Place pod to BackOffQ
                    self.backoff_queue.push(pod_uid, *attempts - 1, self.ctx.time());
                }
                Some(node_uid) => {
                    // Move cached pod from pending to running
                    let mut cached = self.pending_pods.remove(&pod_uid).unwrap();
                    cached.status.phase = PodPhase::Running;
                    cached.status.node_uid = Some(node_uid);
                    self.running_pods.insert(pod_uid, cached);

                    // Update pod status
                    pod.status.node_uid = Some(node_uid);
                    pod.status.phase = PodPhase::Running;

                    // Consume node resources
                    self.consume_node_resources(node_uid, cpu, memory);

                    let data = APIUpdatePodFromScheduler {
                        pod,
                        new_phase: PodPhase::Running,
                        node_uid: node_uid,
                    };

                    // println!("{:?} Scheduler Pod_{:?} placed to Node_{:?} artime: {:?}", self.ctx.time(), pod.metadata.uid, node.metadata.uid, pod.spec.arrival_time);
                    self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.scheduler2api);
                }
            }
        }

        // TODO: Implement API event with time period
        while let Some(pod_uid) = self.backoff_queue.try_pop(self.ctx.time()) {
            self.active_queue.push(ActiveQCmp::wrap(self.pending_pods.get(&pod_uid).unwrap().clone()));
        }
    }


    pub fn is_node_consumable(node: &Node, cpu: u64, memory: u64) -> bool {
        return cpu <= node.spec.available_cpu && memory <= node.spec.available_memory;
    }

    pub fn consume_node_resources(&mut self, node_uid: u64, cpu: u64, memory: u64) {
        self.nodes.get_mut(&node_uid).unwrap().consume(cpu, memory);
        self.monitoring.borrow_mut().scheduler_on_node_consume(cpu, memory);
    }

    pub fn restore_node_resources(&mut self, node_uid: u64, cpu: u64, memory: u64) {
        self.nodes.get_mut(&node_uid).unwrap().restore(cpu, memory);
        self.monitoring.borrow_mut().scheduler_on_node_restore(cpu, memory);
    }


    pub fn process_new_pod(&mut self, pod: Pod) {
        let pod_uid = pod.metadata.uid;

        assert_eq!(self.running_pods.contains_key(&pod_uid), false);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);
        assert_eq!(pod.status.phase, PodPhase::Pending);

        self.pending_pods.insert(pod_uid, pod.clone());
        self.active_queue.push(ActiveQCmp::wrap(pod));
    }

    pub fn process_evicted_pod(&mut self, pod_uid: u64) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running set
        let mut pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources
        let node_uid = pod.status.node_uid.unwrap();
        self.restore_node_resources(node_uid, pod.spec.request_cpu, pod.spec.request_memory);
        pod.status.node_uid = None;

        // Place pod to pending set
        pod.status.phase = PodPhase::Pending;
        self.pending_pods.insert(pod_uid, pod.clone());

        // Place pod to ActiveQ
        self.active_queue.push(ActiveQCmp::wrap(pod));

        // Place pod to BackOffQ
        // self.backoff_queue.push(pod_uid, *attempts - 1, self.ctx.time());
    }

    pub fn process_finished_pod(&mut self, pod_uid: u64) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running
        let pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources
        let node_uid = pod.status.node_uid.unwrap();
        self.restore_node_resources(node_uid, pod.spec.request_cpu, pod.spec.request_memory);

        // Remove pod's filed attempts
        self.failed_attempts.remove(&pod_uid);
    }
}

impl<ActiveQCmp: TraitActiveQCmp, BackOffQ: TraitBackOffQ> dsc::EventHandler for Scheduler<ActiveQCmp, BackOffQ> {
    fn on(&mut self, event: dsc::Event) {
        // println!("Scheduler EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid } => {
                // println!("{:?} Scheduler <Update Pod From Kubelet> pod_{:?} new_phase: {:?}", self.ctx.time(), pod_uid, new_phase);

                match new_phase {
                    PodPhase::Pending => {
                        self.process_evicted_pod(pod_uid);
                    }
                    PodPhase::Succeeded => {
                        self.process_finished_pod(pod_uid);
                    }
                    PodPhase::Running => {
                    }
                    PodPhase::Failed => {
                        panic!("Bad PodPhase Failed");
                    }
                    PodPhase::Unknown => {
                        panic!("Bad PodPhase Unknown");
                    }
                }

                self.schedule();
                self.self_update_on();
            }
            APIAddPod { pod } => {
                // println!("Scheduler <Add Pod>");
                self.process_new_pod(pod);

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

                if self.pending_pods.len() > 0 {
                    self.ctx.emit_self(APISchedulerSelfUpdate{}, self.cluster_state.borrow().constants.scheduler_self_update_period);
                }
            }
        });
        // println!("Scheduler EventHandler <------");
    }
}
