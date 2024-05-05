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

impl Default for LoadType {
    fn default() -> Self { Self::PanicStub(PanicStub::default()) }
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PanicStub;
impl PanicStub {
    pub fn start(&mut self, _: f64) -> (i64, i64, f64, bool) { panic!("PanicStub."); }
    pub fn update(&mut self, _: f64) -> (i64, i64, f64, bool) { panic!("PanicStub."); }
}
