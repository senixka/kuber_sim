use serde::{Deserialize, Serialize};
use dslab_core::EPSILON;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    #[serde(skip_deserializing)]
    pub start_time: f64,

    pub duration: f64,
    pub cpu: u64,
    pub memory: u64,
}

impl Constant {
    pub fn start(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        self.start_time = current_time;
        return (self.cpu,
                self.memory,
                self.duration,
                self.duration < EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        let mut next_spike = self.duration - (current_time - self.start_time);
        if next_spike < EPSILON {
            next_spike = 4.0 * EPSILON;
        }

        return (self.cpu,
                self.memory,
                next_spike,
                current_time - self.start_time + EPSILON > self.duration);
    }
}

impl Eq for Constant {}
