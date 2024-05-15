use crate::my_imports::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConstantInfinite {
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

impl FromStr for ConstantInfinite {
    type Err = ();

    /// Expects "i64;i64"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cpu_str, memory_str) = s.split_once(';').unwrap();

        Ok(Self {
            cpu: sim_ok!(str::parse(cpu_str), "ConstantInfinite. Invalid value for cpu."),
            memory: sim_ok!(str::parse(memory_str), "ConstantInfinite. Invalid value for memory."),
        })
    }
}

impl Eq for ConstantInfinite {}
