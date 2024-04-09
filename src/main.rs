mod pod;
mod node;
mod api;
mod scheduler;
mod kubelet;
mod init;
mod sim_config;
mod load_types;
mod object_meta;
mod active_queue;
mod backoff_queue;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;
    pub use serde::Serialize;

    pub use std::collections::HashMap;

    pub mod dsc {
        pub use dslab_core::{cast, Simulation, SimulationContext, Id, EventHandler, Event, EPSILON};
    }

    pub use crate::pod::Pod;
    pub use crate::load_types::LoadType;
    pub use crate::object_meta::ObjectMeta;
    pub use crate::active_queue::ActiveQCmpUid;
    pub use crate::backoff_queue::BackOffQExponential;
    pub use crate::backoff_queue::TraitBackOffQ;

    pub use crate::pod::PodPhase;
    pub use crate::node::Node;
    pub use crate::scheduler::Scheduler;
    pub use crate::kubelet::Kubelet;
    pub use crate::sim_config::*;

    pub use crate::api::*;
}
use my_imports::*;
use crate::init::Init;

fn main() {
    let mut sim = dsc::Simulation::new(179);

    // Create components

    let api = Rc::new(RefCell::new(APIServer::new(sim.create_context("api"))));
    let api_id = sim.add_handler("api", api.clone());

    let scheduler = Rc::new(RefCell::new(Scheduler::<ActiveQCmpUid, BackOffQExponential>::new(sim.create_context("scheduler"))));
    let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

    let init = Rc::new(RefCell::new(Init::new(sim.create_context("init"))));

    // Init components

    sim_config::SimConfig::from_yaml("./data/sim_config.yaml");
    sim_config::NetworkDelays::from_yaml("./data/sim_config.yaml");
    api.borrow_mut().presimulation_init(scheduler_id);
    scheduler.borrow_mut().presimulation_init(api_id);
    init.borrow_mut().presimulation_init(api_id);

    // Final check

    api.borrow().presimulation_check();
    scheduler.borrow().presimulation_check();
    init.borrow().presimulation_check();

    // Simulation

    init.borrow().submit_nodes(&mut sim);

    while sim.step() == true {
        println!("-------------------------------");
    }

    init.borrow().submit_pods();

    while sim.step() == true {
        println!("-------------------------------");
    }
}
