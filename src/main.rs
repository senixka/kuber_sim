mod pod;
mod node;
mod etcd;
mod types;
mod api;
mod scheduler;
mod kubelet;
mod workload;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;
    pub use serde::Serialize;

    pub use std::collections::HashMap;

    pub mod dsc {
        pub use dslab_core::{cast, Simulation, SimulationContext, Id, EventHandler, Event};
    }

    pub use crate::pod::Pod;
    pub use crate::pod::PodPhase;
    pub use crate::node::Node;
    pub use crate::etcd::Etcd;
    pub use crate::scheduler::Scheduler;
    pub use crate::kubelet::Kubelet;

    pub use crate::api::*;
    pub use crate::types::*;
}
use my_imports::*;
use crate::workload::Workload;

fn main() {
    let mut sim = dsc::Simulation::new(179);

    let etcd = Rc::new(RefCell::new(Etcd::new(sim.create_context("etcd"))));
    let etcd_id = sim.add_handler("etcd", etcd.clone());

    let api = Rc::new(RefCell::new(APIServer::new(sim.create_context("api"))));
    let api_id = sim.add_handler("api", api.clone());

    let scheduler = Rc::new(RefCell::new(Scheduler::new(sim.create_context("scheduler"))));
    let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

    let workload = Rc::new(RefCell::new(Workload::new(sim.create_context("workload"))));
    let _ = sim.add_handler("workload", workload.clone());

    let kubelet_1 = Rc::new(RefCell::new(Kubelet::new(
        sim.create_context("kubelet_1"),
        Node::from_yaml("./data/node_1.yaml")
    )));
    let kubelet_1_id = sim.add_handler("kubelet_1", kubelet_1.clone());

    scheduler.borrow_mut().presimulation_init(api_id);
    api.borrow_mut().presimulation_init(etcd_id, scheduler_id);
    workload.borrow_mut().presimulation_init(api_id);
    kubelet_1.borrow_mut().presimulation_init(kubelet_1_id, api_id);

    api.borrow_mut().presimulation_check();
    scheduler.borrow_mut().presimulation_check();
    workload.borrow_mut().presimulation_check();

    let init = sim.create_context("init");
    init.emit_now(APIAddNode{node: kubelet_1.borrow().node.clone()}, api_id);

    while sim.step() == true {
        println!("-------------------------------");
    }

    workload.borrow().submit_pods();

    while sim.step() == true {
        println!("-------------------------------");
    }
}
