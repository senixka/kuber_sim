use crate::my_imports::*;


#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    #[serde(skip)]
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
                self.duration < dsc::EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (u64, u64, f64, bool) {
        let next_change = self.duration - (current_time - self.start_time);
        if next_change < dsc::EPSILON {
            return (0, 0, 0.0, true);
        }

        return (self.cpu,
                self.memory,
                next_change,
                current_time - self.start_time + dsc::EPSILON > self.duration);
    }
}

impl Eq for Constant {}
