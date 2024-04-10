use std::collections::BTreeMap;
use crate::my_imports::*;

pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub cluster_state: Rc<RefCell<ClusterState>>,
    pub api_sim_id: dsc::Id,
    pub node: Node,

    pub pods: HashMap<u64, Pod>,
    pub running_loads: BTreeMap<u64, (u64, u64, LoadType)>,
    pub self_update_enabled: bool,
}

impl Kubelet {
    pub fn new(ctx: dsc::SimulationContext, node: Node, cluster_state: Rc<RefCell<ClusterState>>) -> Self {
        Self {
            ctx,
            cluster_state,
            api_sim_id: dsc::Id::MAX,
            node,
            pods: HashMap::new(),
            running_loads: BTreeMap::new(),
            self_update_enabled: false,
        }
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn self_update_on(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(APIKubeletSelfUpdate {}, self.cluster_state.borrow().constants.kubelet_self_update_period);
        }
    }

    pub fn place_new_pod(&mut self, pod: Pod) -> bool {
        let pod_uid = pod.metadata.uid;

        println!("Pods: {0}", self.pods.len());
        println!("MEM: Available/Installed: {0}/{1}", self.node.spec.available_memory, self.node.spec.installed_memory);
        println!("CPU: Available/Installed: {0}/{1}", self.node.spec.available_cpu, self.node.spec.installed_cpu);


        // Store pod
        assert!(!self.pods.contains_key(&pod_uid));
        self.pods.insert(pod_uid, pod.clone());

        // Run pod's load
        let mut load = pod.spec.load.clone();
        let (cpu, memory, is_finished) = load.start(self.ctx.time());
        assert!(!is_finished);

        if !self.node.is_consumable(cpu, memory) {
            self.pods.remove(&pod_uid).unwrap();
            return false;
        }
        self.node.consume(cpu, memory);
        self.running_loads.insert(pod_uid, (cpu, memory, load));

        return true;
    }

    pub fn update_load(&mut self) {
        // Restore current resources, find finished pods
        let mut finished_pods: Vec<u64> = Vec::new();
        for (pod_uid, (prev_cpu, prev_memory, load)) in self.running_loads.iter_mut() {
            self.node.restore(prev_cpu.clone(), prev_memory.clone());
            let (_, _, is_finished) = load.update(self.ctx.time());

            if is_finished {
                finished_pods.push(pod_uid.clone());
            }
        }
        // println!("MEM: Available/Installed: {0}/{1}", self.node.spec.available_memory, self.node.spec.installed_memory);
        // println!("CPU: Available/Installed: {0}/{1}", self.node.spec.available_cpu, self.node.spec.installed_cpu);
        assert_eq!(self.node.spec.installed_memory, self.node.spec.available_memory);
        assert_eq!(self.node.spec.installed_cpu, self.node.spec.available_cpu);

        // println!("##########################################");
        // let mut buffer = String::new();
        // io::stdin().read_line(&mut buffer).unwrap();

        // Delete finished pods
        for pod_uid in finished_pods.iter() {
            self.running_loads.remove(pod_uid).unwrap();
            self.pods.remove(pod_uid).unwrap();

            let data = APIUpdatePodFromKubelet {
                pod_uid: pod_uid.clone(),
                new_phase: PodPhase::Succeeded,
                node_uid: self.node.metadata.uid,
            };
            self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
        }

        // Consume resources. Find pods to evict
        let mut evicted_pods: Vec<u64> = Vec::new();
        for (pod_uid, (prev_cpu, prev_memory, load)) in self.running_loads.iter_mut() {
            let (tmp_cpu, tmp_memory, is_finished) = load.update(self.ctx.time());
            assert!(!is_finished);

            if self.node.is_consumable(tmp_cpu, tmp_memory) {
                self.node.consume(tmp_cpu, tmp_memory);
                *prev_cpu = tmp_cpu;
                *prev_memory = tmp_memory;
            } else {
                evicted_pods.push(pod_uid.clone());
            }
        }

        // Evict pods
        for pod_uid in evicted_pods.iter() {
            self.running_loads.remove(pod_uid).unwrap();
            self.pods.remove(pod_uid).unwrap();

            let data = APIUpdatePodFromKubelet {
                pod_uid: pod_uid.clone(),
                new_phase: PodPhase::Pending,
                node_uid: self.node.metadata.uid,
            };
            self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
        }
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        println!("Kubelet Node_{0} EventHandler ------>", self.node.metadata.uid);
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                println!("New pod");

                assert_eq!(node_uid, self.node.metadata.uid);
                assert_eq!(new_phase, PodPhase::Running);
                assert_eq!(self.running_loads.len(), self.pods.len());

                if !self.pods.contains_key(&pod.metadata.uid) {
                    self.update_load();

                    if self.place_new_pod(pod.clone()) {
                        self.self_update_on();
                    } else {
                        let data = APIUpdatePodFromKubelet {
                            pod_uid: pod.metadata.uid,
                            new_phase: PodPhase::Pending,
                            node_uid: self.node.metadata.uid,
                        };
                        self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
                    }
                    assert_eq!(self.running_loads.len(), self.pods.len());
                }
            }
            APIKubeletSelfUpdate{} => {
                println!("Self update");
                assert_eq!(self.running_loads.len(), self.pods.len());
                self.update_load();
                assert_eq!(self.running_loads.len(), self.pods.len());

                if !self.pods.is_empty() {
                    self.self_update_enabled = true;
                    self.ctx.emit_self(APIKubeletSelfUpdate{}, self.cluster_state.borrow().constants.kubelet_self_update_period);
                } else {
                    self.self_update_enabled = false;
                }
            }
        });
        println!("Kubelet Node_{0} EventHandler <------", self.node.metadata.uid);
    }
}
