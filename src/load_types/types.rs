use crate::Constant;
use crate::BusyBox;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum LoadType {
    Constant(Constant),
    BusyBox(BusyBox),
}

impl LoadType {
    pub fn get_duration(&self) -> f64 {
        return match self {
            LoadType::Constant(load) => { load.duration }
            LoadType::BusyBox(load) => { load.duration }
        }
    }

    pub fn start(&mut self, current_time: f64) -> (u64, u64, bool) {
        return match self {
            LoadType::Constant(load) => { load.start(current_time) }
            LoadType::BusyBox(load) => { load.start(current_time) }
        }
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, bool) {
        return match self {
            LoadType::Constant(load) => { load.update(current_time) }
            LoadType::BusyBox(load) => { load.update(current_time) }
        }
    }
}
