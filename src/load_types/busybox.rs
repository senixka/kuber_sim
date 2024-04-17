use serde::{Deserialize, Serialize};
use dslab_core::EPSILON;


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
    pub fn start(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        self.start_time = current_time;
        return (self.cpu_down,
                self.memory_down,
                self.shift_time,
                self.duration < EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        let epoch: u64 = ((current_time - self.start_time) / self.shift_time) as u64;
        let mut next_change = (epoch + 1) as f64 * self.shift_time - (current_time - self.start_time);
        if next_change < EPSILON {
            return (0, 0, 0.0, true);
        }

        if epoch % 2 == 0 {
            return (self.cpu_down,
                    self.memory_down,
                    next_change,
                    current_time - self.start_time + EPSILON > self.duration);
        }
        return (self.cpu_up,
                self.memory_up,
                next_change,
                current_time - self.start_time + EPSILON > self.duration);
    }
}

impl Eq for BusyBox {}
