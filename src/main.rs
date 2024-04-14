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
    pub use crate::scheduler::filter::*;
    pub use crate::scheduler::score::*;
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Data {
    pub cpu: u32,
    pub memory: u32,
    pub uid: u32,

    pub value: u32,
}

impl Data {
    pub fn new() -> Self {
        Self {
            cpu: 1,
            memory: 1,
            uid: 1,
            value: 1,
        }
    }
}

impl dsc::EventHandler for Data {
    fn on(&mut self, event: Event) {
        dsc::cast!(match event.data {
            Data { cpu, memory, uid, value } => {
                println!("Consume");
            }
        });
    }
}

impl RTreeObject for Data {
    type Envelope = AABB<(i64, i64, i64)>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point((self.cpu as i64, self.memory as i64, self.uid as i64))
    }
}

fn main() {
    println!("Node: {0}", size_of::<Node>());
    println!("NodeSpec: {0}", size_of::<NodeSpec>());
    println!("NodeStatus: {0}", size_of::<NodeStatus>());
    println!("ObjectMeta: {0}", size_of::<ObjectMeta>());
    println!("Pod: {0}", size_of::<Pod>());

    {
        let mut backoff = BackOffQExponential::new(1.0, 10.0);
        backoff.push(1, 0, 0.0);
        assert_eq!(backoff.try_pop(0.5), None);
        assert_eq!(backoff.try_pop(1.1), Some(1));
    }

    {
        let mut index = RTree::new();
        index.insert(Data{ cpu: 0, memory: 0, uid: 0, value: 23 });
        index.insert(Data{ cpu: 0, memory: 2, uid: 1, value: 23 });
        index.insert(Data{ cpu: 2, memory: 0, uid: 2, value: 42 });
        index.insert(Data{ cpu: 2, memory: 2, uid: 3, value: 42 });

        let half_unit_square = AABB::from_corners((0, 0, 0), (1, 2, i64::MAX));
        let unit_square = AABB::from_corners((0, 0, 0), (2, 2, i64::MAX));

        let elements_in_half_unit_square = index.locate_in_envelope(&half_unit_square);
        let elements_in_unit_square = index.locate_in_envelope(&unit_square);

        assert_eq!(elements_in_half_unit_square.count(), 2);
        assert_eq!(elements_in_unit_square.count(), 4);
    }

    {
        // let mut sim = dsc::Simulation::new(179);
        //
        // let ctx = sim.create_context("kek");
        // let data = Rc::new(RefCell::new(Data::new()));
        // let _ = sim.add_handler("kek", data.clone());
        //
        // ctx.emit_self(Data {cpu: 1, memory: 1, uid: 1, value: 1}, 10.0);
        // println!("{0} == 1", sim.event_count());
        //
        // ctx.emit_self(Data {cpu: 1, memory: 1, uid: 1, value: 1}, 10.0);
        // println!("{0} == 2", sim.event_count());
        //
        // // sim.step_for_duration(20.0);
        // sim.step_until_no_events();
        //
        // println!("{0} == 0", sim.event_count());
    }

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
