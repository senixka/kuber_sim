use crate::my_imports::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

impl FromStr for LoadType {
    type Err = ();

    /// Expects "<enum_index: u8>;<enum_inner>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (enum_index, enum_inner) = s.split_once(';').unwrap();
        let enum_inner = enum_inner.trim();

        match enum_index {
            "0" => Ok(LoadType::Constant(str::parse(enum_inner).unwrap())),
            "1" => Ok(LoadType::ConstantInfinite(str::parse(enum_inner).unwrap())),
            "2" => Ok(LoadType::BusyBox(str::parse(enum_inner).unwrap())),
            "3" => Ok(LoadType::BusyBoxInfinite(str::parse(enum_inner).unwrap())),
            _ => Err(()),
        }
    }
}

impl Default for LoadType {
    fn default() -> Self {
        Self::PanicStub(PanicStub::default())
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PanicStub;
impl PanicStub {
    pub fn start(&mut self, _: f64) -> (i64, i64, f64, bool) {
        panic!("PanicStub.");
    }
    pub fn update(&mut self, _: f64) -> (i64, i64, f64, bool) {
        panic!("PanicStub.");
    }
}
