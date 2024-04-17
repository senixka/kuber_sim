mod kubelet;
mod scheduler;
mod load_types;
mod objects;
mod api_server;
mod simulation;
mod test;
mod my_macro;

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
    pub use crate::scheduler::filter::*;
    pub use crate::scheduler::score::*;
    pub use crate::scheduler::normalize_score::*;
}

use std::collections::{BTreeMap, BTreeSet, HashSet, LinkedList};
use std::io::stdin;
use std::mem::size_of;
use dslab_core::{Event, EventHandler};
use my_imports::*;
use crate::scheduler::backoff_queue::{BackOffQExponential, TraitBackOffQ};
use crate::simulation::experiment::*;

use rstar::{RTree, AABB, RTreeObject};
use serde::{Deserialize, Serialize};
use crate::my_imports::Node;
use crate::objects::node::{NodeSpec, NodeStatus};
use crate::simulation::monitoring::Monitoring;
use crate::test::Test;

#[macro_use] use crate::my_macro::*;


fn main() {
    // println!("Node: {0}", size_of::<Node>());
    // println!("NodeSpec: {0}", size_of::<NodeSpec>());
    // println!("NodeStatus: {0}", size_of::<NodeStatus>());
    // println!("ObjectMeta: {0}", size_of::<ObjectMeta>());
    // println!("Pod: {0}", size_of::<Pod>());
    // println!("BTreeMap: {0}", size_of::<BTreeMap<String, String>>());
    // println!("HashMap: {0}", size_of::<HashMap<String, String>>());
    // println!("Vec: {0}", size_of::<Vec<(String, String)>>());
    // println!("List: {0}", size_of::<LinkedList<(String, String)>>());

    debug_print!("Debug print Enabled");

    Test::test_all();

    let mut value = String::new();
    stdin().read_line(&mut value).unwrap();
    value = value.trim().to_string();

    if value == "1" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_1.yaml",
            "./data/workload/test_1.yaml",
            179
        );
        test.run();
    }
    if value == "2" {
        let mut test = Experiment::new(
            "./data/cluster_state/state.yaml",
            "./data/workload/pods.yaml",
            179
        );
        test.run();
    }
}
