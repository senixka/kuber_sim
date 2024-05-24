pub use crate::common_imports::dsc;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Constant {
    #[serde(skip)]
    pub start_time: f64,

    pub cpu: i64,
    pub memory: i64,
    pub duration: f64,
}

impl Constant {
    pub fn start(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        self.start_time = current_time;
        return (self.cpu, self.memory, self.duration, self.duration < dsc::EPSILON);
    }

    pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        let next_change = self.duration - (current_time - self.start_time);
        if next_change < dsc::EPSILON {
            return (0, 0, 0.0, true);
        }

        return (
            self.cpu,
            self.memory,
            next_change,
            current_time - self.start_time + dsc::EPSILON > self.duration,
        );
    }
}

impl std::str::FromStr for Constant {
    type Err = ();

    /// Expects "i64;i64;f64"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cpu_str, other) = s.split_once(';').unwrap();
        let (memory_str, duration_str) = other.split_once(';').unwrap();

        Ok(Self {
            cpu: sim_ok!(str::parse(cpu_str), "Constant. Invalid value for cpu."),
            memory: sim_ok!(str::parse(memory_str), "Constant. Invalid value for memory."),
            duration: sim_ok!(str::parse(duration_str), "Constant. Invalid value for duration."),
            start_time: 0.0,
        })
    }
}

impl Eq for Constant {}
