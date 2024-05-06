use crate::my_imports::*;


#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConstantInfinite {
    #[serde(skip)]
    pub cpu: i64,
    pub memory: i64,
}


impl ConstantInfinite {
    pub fn start(&mut self, _: f64) -> (i64, i64, f64, bool) {
        (self.cpu, self.memory, f64::MAX / 8.0, false)
    }

    pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        (self.cpu, self.memory, f64::MAX / 8.0 - current_time, false)
    }
}

impl Eq for ConstantInfinite {}
