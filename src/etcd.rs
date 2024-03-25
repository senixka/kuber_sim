use crate::my_imports::*;

pub struct Etcd {
    ctx: dsc::SimulationContext,

    pods: HashMap<u64, Pod>, // Pod uid -> Pod
    kubelets: HashMap<u64, Rc<RefCell<Kubelet>>>, // Node uid -> Kubelet ptr
}

impl Etcd {
    pub fn new(ctx: dsc::SimulationContext) -> Self {
        Self {
            ctx,
            pods: HashMap::new(),
            kubelets: HashMap::new(),
        }
    }
}

impl dsc::EventHandler for Etcd {
    fn on(&mut self, event: dsc::Event) {
        println!("Etcd EventHandler ------>");
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid } => {
                assert_eq!(self.pods.contains_key(&pod_uid), true);
                // self.pods.get_mut(&pod_uid).unwrap().status.phase = new_phase;
            }
            APIUpdatePodFromScheduler { pod, new_phase, kubelet_sim_id } => {
                assert_eq!(self.pods.contains_key(&pod.metadata.uid), true);
                // self.pods.get_mut(&pod_uid).unwrap().status.phase = new_phase;
            }
            APIAddPod { pod } => {
                assert_eq!(self.pods.contains_key(&pod.metadata.uid), false);
                // self.pods.insert(pod.metadata.uid, pod);
            }
            APIRemovePod { pod_uid } => {
                assert_eq!(self.pods.contains_key(&pod_uid), true);
                // self.pods.remove(&pod_uid);
            }
            APIAddNode { node } => {
                let node_uid = node.metadata.uid;
                assert_eq!(self.kubelets.contains_key(&node_uid), false);
                // self.kubelets.insert(node_uid, kubelet);
            }
            APIRemoveKubelet { node_uid } => {
                assert_eq!(self.kubelets.contains_key(&node_uid), true);
                // self.kubelets.remove(&node_uid);
            }
        });
        println!("Etcd EventHandler <------");
    }
}
