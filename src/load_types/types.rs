use crate::Constant;
use crate::BusyBox;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum LoadType {
    #[default]
    None,
    Constant(Constant),
    BusyBox(BusyBox),
}

impl LoadType {
    pub fn get_duration(&self) -> f64 {
        return match self {
            LoadType::Constant(load) => { load.duration }
            LoadType::BusyBox(load) => { load.duration }
            _ => { 0.0 }
        }
    }

    pub fn start(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        return match self {
            LoadType::Constant(load) => { load.start(current_time) }
            LoadType::BusyBox(load) => { load.start(current_time) }
            _ => { (0, 0, 0.0, false) }
        }
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        return match self {
            LoadType::Constant(load) => { load.update(current_time) }
            LoadType::BusyBox(load) => { load.update(current_time) }
            _ => { (0, 0, 0.0, false) }
        }
    }
}
