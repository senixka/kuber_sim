use crate::my_imports::*;

pub struct Scheduler {
    ctx: dsc::SimulationContext,
    api_sim_id: dsc::Id,

    pods: HashMap<u64, Pod>,
    nodes: HashMap<u64, Node>,
    self_update_enabled: bool,
}

impl Scheduler {
    pub fn new(ctx: dsc::SimulationContext) -> Self {
        Self {
            ctx,
            api_sim_id: dsc::Id::MAX,
            pods: HashMap::new(),
            nodes: HashMap::new(),
            self_update_enabled: false,
        }
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.api_sim_id, dsc::Id::MAX);
    }

    pub fn is_pod_placeable(pod: &Pod, node: &Node) -> bool {
        return pod.spec.request_cpu <= node.spec.available_cpu
            && pod.spec.request_memory <= node.spec.available_memory;
    }

    pub fn place_pod(pod: &Pod, node: &mut Node) {
        node.spec.available_cpu -= pod.spec.request_cpu;
        node.spec.available_memory -= pod.spec.request_memory;
    }

    pub fn unplace_pod(pod: &Pod, node: &mut Node) {
        node.spec.available_cpu += pod.spec.request_cpu;
        node.spec.available_memory += pod.spec.request_memory;
    }

    pub fn self_update_on(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(APISchedulerSelfUpdate {}, 10.0);
        }
    }

    pub fn schedule(&mut self) {
        for (pod_uid, pod) in self.pods.iter_mut() {
            if pod.status.phase != PodPhase::Pending {
                continue;
            }
            println!("Scheduler Pod_{0} is Pending", pod_uid);

            for (node_uid, node) in self.nodes.iter_mut() {
                println!("Try node_{0}", node.metadata.uid);
                if Scheduler::is_pod_placeable(&pod, &node) {
                    Scheduler::place_pod(&pod, node);
                    pod.status.node_uid = Some(node.metadata.uid);
                    pod.status.phase = PodPhase::Running;

                    let data = APIUpdatePodFromScheduler {
                        pod: pod.clone(),
                        new_phase: PodPhase::Running,
                        node_uid: node_uid.clone(),
                    };

                    println!("Scheduler Pod_{0} placed to Node_{1}", pod_uid, node.metadata.uid);
                    self.ctx.emit(data, self.api_sim_id, NetworkDelays::scheduler2api());
                    break;
                }
            }
        }
    }
}

impl dsc::EventHandler for Scheduler {
    fn on(&mut self, event: dsc::Event) {
        println!("Scheduler EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid } => {
                println!("Scheduler <Update Pod From Kubelet> pod_{0}", pod_uid);

                if new_phase == PodPhase::Succeeded || new_phase == PodPhase::Failed {
                    Scheduler::unplace_pod(self.pods.get_mut(&pod_uid).unwrap(), self.nodes.get_mut(&node_uid).unwrap());
                    self.pods.remove(&pod_uid);
                }
                if new_phase == PodPhase::Pending {
                    self.pods.get_mut(&pod_uid).unwrap().status.phase = new_phase;
                    Scheduler::unplace_pod(self.pods.get_mut(&pod_uid).unwrap(), self.nodes.get_mut(&node_uid).unwrap());
                }
                self.schedule();
                self.self_update_on();
            }
            APIAddPod { pod } => {
                println!("Scheduler <Add Pod>");

                self.pods.insert(pod.metadata.uid, pod);
                self.schedule();
            }
            APIAddNode { kubelet_sim_id: _ , node } => {
                println!("Scheduler <Add Kubelet>");

                self.nodes.insert(node.metadata.uid, node);
            }
            APISchedulerSelfUpdate { } => {
                println!("Scheduler <Self Update>");
                self.schedule();

                if self.pods.len() > 0 {
                    self.ctx.emit_self(APISchedulerSelfUpdate{}, 10.0);
                }
            }
        });
        println!("Scheduler EventHandler <------");
    }
}
