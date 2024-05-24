pub use crate::common_imports::dsc;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BusyBox {
    #[serde(skip)]
    pub start_time: f64,

    pub cpu_down: i64,
    pub memory_down: i64,
    pub cpu_up: i64,
    pub memory_up: i64,

    pub shift_time: f64,
    pub duration: f64,
}

impl BusyBox {
    pub fn start(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        self.start_time = current_time;
        return (
            self.cpu_down,
            self.memory_down,
            self.shift_time,
            self.duration < dsc::EPSILON,
        );
    }

    pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
        if current_time - self.start_time + dsc::EPSILON > self.duration {
            return (0, 0, 0.0, true);
        }

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

impl std::str::FromStr for BusyBox {
    type Err = ();

    /// Expects "i64;i64;64;i64;f64;f64"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cpu_down_str, other) = s.split_once(';').unwrap();
        let (memory_down_str, other) = other.split_once(';').unwrap();
        let (cpu_up_str, other) = other.split_once(';').unwrap();
        let (memory_up_str, other) = other.split_once(';').unwrap();
        let (shift_time_str, duration_str) = other.split_once(';').unwrap();

        Ok(Self {
            cpu_down: sim_ok!(str::parse(cpu_down_str), "BusyBox. Invalid value for cpu_down."),
            memory_down: sim_ok!(str::parse(memory_down_str), "BusyBox. Invalid value for memory_down."),
            cpu_up: sim_ok!(str::parse(cpu_up_str), "BusyBox. Invalid value for cpu_up."),
            memory_up: sim_ok!(str::parse(memory_up_str), "BusyBox. Invalid value for memory_up."),
            shift_time: sim_ok!(str::parse(shift_time_str), "BusyBox. Invalid value for shift_time."),
            duration: sim_ok!(str::parse(duration_str), "BusyBox. Invalid value for duration."),
            start_time: 0.0,
        })
    }
}

impl Eq for BusyBox {}
