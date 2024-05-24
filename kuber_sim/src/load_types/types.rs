use crate::load_types::busybox::*;
use crate::load_types::busybox_infinite::*;
use crate::load_types::constant::*;
use crate::load_types::constant_infinite::*;

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[impl_enum::with_methods {
pub fn start(&mut self, current_time: f64) -> (i64, i64, f64, bool)
pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool)
}]
pub enum LoadType {
    Constant(Constant),
    ConstantInfinite(ConstantInfinite),
    BusyBox(BusyBox),
    BusyBoxInfinite(BusyBoxInfinite),

    // Default
    PanicStub(PanicStub),
}

impl std::str::FromStr for LoadType {
    type Err = ();

    /// Expects "<enum_index: u8>;<enum_inner>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (enum_index, enum_inner) = s.split_once(';').unwrap();
        let enum_inner = enum_inner.trim();

        match enum_index {
            "0" => Ok(LoadType::Constant(sim_ok!(
                str::parse(enum_inner),
                "LoadType. Cannot parse Constant workload model."
            ))),
            "1" => Ok(LoadType::ConstantInfinite(sim_ok!(
                str::parse(enum_inner),
                "LoadType. Cannot parse ConstantInfinite workload model."
            ))),
            "2" => Ok(LoadType::BusyBox(sim_ok!(
                str::parse(enum_inner),
                "LoadType. Cannot parse BusyBox workload model."
            ))),
            "3" => Ok(LoadType::BusyBoxInfinite(sim_ok!(
                str::parse(enum_inner),
                "LoadType. Cannot parse BusyBoxInfinite workload model."
            ))),
            _ => Err(()),
        }
    }
}

impl Default for LoadType {
    fn default() -> Self {
        Self::PanicStub(PanicStub::default())
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanicStub;
impl PanicStub {
    pub fn start(&mut self, _: f64) -> (i64, i64, f64, bool) {
        panic!("PanicStub.");
    }
    pub fn update(&mut self, _: f64) -> (i64, i64, f64, bool) {
        panic!("PanicStub.");
    }
}
