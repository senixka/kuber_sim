use serde::{Deserialize, Serialize};
use crate::my_imports::dsc;


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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    #[serde(skip_deserializing)]
    pub start_time: f64,

    pub duration: f64,
    pub cpu: u64,
    pub memory: u64,
}

impl Constant {
    pub fn start(&mut self, current_time: f64) -> (u64, u64, bool) {
        self.start_time = current_time;
        return (self.cpu, self.memory, self.duration < dsc::EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, bool) {
        return (self.cpu, self.memory, current_time - self.start_time + dsc::EPSILON > self.duration);
    }
}

impl Eq for Constant {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusyBox {
    #[serde(skip_deserializing)]
    pub start_time: f64,

    pub duration: f64,
    pub shift_time: f64,
    pub cpu_down: u64,
    pub memory_down: u64,
    pub cpu_up: u64,
    pub memory_up: u64,
}

impl BusyBox {
    pub fn start(&mut self, current_time: f64) -> (u64, u64, bool) {
        self.start_time = current_time;
        return (self.cpu_down, self.memory_down, self.duration < dsc::EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, bool) {
        let epoch: u64 = ((current_time - self.start_time) / self.shift_time) as u64;

        if epoch % 2 == 0 {
            return (self.cpu_down, self.memory_down, current_time - self.start_time + dsc::EPSILON > self.duration);
        }
        return (self.cpu_up, self.memory_up, current_time - self.start_time + dsc::EPSILON > self.duration);
    }
}

impl Eq for BusyBox {}

////////////////////////////////////////////////////////////////////////////////////////////////////
