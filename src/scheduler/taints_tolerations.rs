use crate::my_imports::*;


// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationEffect {
    #[default]
    Empty = 0,
    NoSchedule = 1,
    PreferNoSchedule = 2,
}


// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationOperator {
    #[default]
    Equal = 0,
    Exists = 1,
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaintValue {
    pub value: String,
    pub effect: TaintTolerationEffect,
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TolerationValue {
    #[serde(default)]
    pub value: String,
    pub operator: TaintTolerationOperator,
    pub effect: TaintTolerationEffect,
}
