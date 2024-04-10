mod kubelet;
mod scheduler;
mod load_types;
mod objects;
mod api_server;
mod simulation;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;

    pub use std::collections::HashMap;

    pub mod dsc {
        pub use dslab_core::{cast, Event, EventHandler, Id, Simulation, SimulationContext};
    }

    pub use crate::objects::pod::Pod;
    pub use crate::load_types::types::LoadType;
    pub use crate::load_types::busybox::BusyBox;
    pub use crate::load_types::constant::Constant;
    pub use crate::objects::object_meta::ObjectMeta;

    pub use crate::objects::pod::PodPhase;
    pub use crate::objects::node::Node;
    pub use crate::scheduler::scheduler::*;
    pub use crate::api_server::api::*;
    pub use crate::api_server::events::*;
    pub use crate::kubelet::Kubelet;
    pub use crate::simulation::config::*;
    pub use crate::api_server::*;
}
use my_imports::*;
use crate::simulation::experiment::*;

fn main() {
    let mut test_1 = Experiment::new(
        "./data/cluster_state/test_1.yaml",
        "./data/workload/test_1.yaml",
        179
    );
    test_1.run();
}
