// use yaml_rust::yaml::Hash;
// use crate::my_imports::*;
//
//
// pub struct CA {
//     ctx: dsc::SimulationContext,
//     cluster_state: Rc<RefCell<ClusterState>>,
//     api_sim_id: dsc::Id,
//     monitoring: Rc<RefCell<Monitoring>>,
//
//     // kubelet_pool: Vec<(dsc::Id, Rc<RefCell<Kubelet>>)>,
//     // free_nodes: Vec<NodeGroup>,
//     // in_use: Vec<NodeGroup>,
// }
//
//
// impl CA {
//     pub fn new(ctx: dsc::SimulationContext,
//                cluster_state: Rc<RefCell<ClusterState>>,
//                monitoring: Rc<RefCell<Monitoring>>) -> Self {
//         Self {
//             ctx,
//             cluster_state,
//             api_sim_id: dsc::Id::MAX,
//             monitoring,
//         }
//     }
//
//     pub fn presimulation_init(&mut self, sim: &mut dsc::Simulation, api_sim_id: dsc::Id, node_pool: &Vec<NodeGroup>) {
//         self.api_sim_id = api_sim_id;
//
//
//     }
//
//     pub fn presimulation_check(&self) {
//         assert_ne!(self.api_sim_id, dsc::Id::MAX);
//     }
//
// }