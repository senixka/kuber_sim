use crate::my_imports::*;

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PanicStub;
impl PanicStub {
    pub fn get_duration(&self) -> f64 { panic!("PanicStub."); }
    pub fn start(&mut self, _: f64) -> (i64, i64, f64, bool) { panic!("PanicStub."); }
    pub fn update(&mut self, _: f64) -> (i64, i64, f64, bool) { panic!("PanicStub."); }
}

impl Default for LoadType {
    fn default() -> Self { LoadType::PanicStub(PanicStub::default()) }
}


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

    PanicStub(PanicStub),
}

//
// impl LoadType {
//     pub fn get_duration(&self) -> f64 {
//         return match self {
//             LoadType::Constant(load) => { load.duration }
//             LoadType::BusyBox(load) => { load.duration }
//             LoadType::BusyBoxInfinite(_) => { f64::MAX / 8.0 }
//             LoadType::ConstantInfinite(_) => { f64::MAX / 8.0 }
//             _ => { panic!("Unexpected load type.") }
//         }
//     }
//
//     pub fn start(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
//         return match self {
//             LoadType::Constant(load) => { load.start(current_time) }
//             LoadType::BusyBox(load) => { load.start(current_time) }
//             LoadType::BusyBoxInfinite(load) => { load.start(current_time) }
//             LoadType::ConstantInfinite(load) => { load.start(current_time) }
//             _ => { panic!("Unexpected load type.") }
//         }
//     }
//
//     pub fn update(&mut self, current_time: f64) -> (i64, i64, f64, bool) {
//         return match self {
//             LoadType::Constant(load) => { load.update(current_time) }
//             LoadType::BusyBox(load) => { load.update(current_time) }
//             LoadType::BusyBoxInfinite(load) => { load.update(current_time) }
//             LoadType::ConstantInfinite(load) => { load.update(current_time) }
//             _ => { panic!("Unexpected load type.") }
//         }
//     }
// }
