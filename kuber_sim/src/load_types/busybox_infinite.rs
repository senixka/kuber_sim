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

impl FromStr for BusyBoxInfinite {
    type Err = ();

    /// Expects "i64;i64;64;i64;f64"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cpu_down_str, other) = s.split_once(';').unwrap();
        let (memory_down_str, other) = other.split_once(';').unwrap();
        let (cpu_up_str, other) = other.split_once(';').unwrap();
        let (memory_up_str, shift_time_str) = other.split_once(';').unwrap();

        Ok(Self {
            cpu_down: sim_ok!(str::parse(cpu_down_str), "BusyBoxInfinite. Invalid value for cpu_down."),
            memory_down: sim_ok!(
                str::parse(memory_down_str),
                "BusyBoxInfinite. Invalid value for memory_down."
            ),
            cpu_up: sim_ok!(str::parse(cpu_up_str), "BusyBoxInfinite. Invalid value for cpu_up."),
            memory_up: sim_ok!(
                str::parse(memory_up_str),
                "BusyBoxInfinite. Invalid value for memory_up."
            ),
            shift_time: sim_ok!(
                str::parse(shift_time_str),
                "BusyBoxInfinite. Invalid value for shift_time."
            ),
            start_time: 0.0,
        })
    }
}

impl Eq for BusyBoxInfinite {}
