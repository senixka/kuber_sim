use crate::my_imports::*;


#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BusyBoxInfinite {
    #[serde(skip)]
    pub start_time: f64,

    pub cpu_down: i64,
    pub memory_down: i64,
    pub cpu_up: i64,
    pub memory_up: i64,

    pub shift_time: f64,
}


impl BusyBoxInfinite {
    pub fn start(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        self.start_time = current_time;
        (self.cpu_down, self.memory_down, self.shift_time, false)
    }

    pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        let epoch = ((current_time - self.start_time) / self.shift_time) as i64;
        let mut next_change = (epoch + 1) as f64 * self.shift_time - (current_time - self.start_time);
        if next_change < dsc::EPSILON {
            next_change += 10.0 * dsc::EPSILON;
        }

        if epoch % 2 == 0 {
            return (self.cpu_down, self.memory_down, next_change, false);
        }
        return (self.cpu_up, self.memory_up, next_change, false);
    }
}

impl Eq for BusyBoxInfinite {}
