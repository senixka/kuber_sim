use crate::my_imports::*;


// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationEffect {
    #[default]
    NoSchedule = 0,
    PreferNoSchedule = 1,
}


// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationOperator {
    #[default]
    Equal = 0,
    Exists = 1,
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Taint {
    pub key: String,
    pub value: String,
    pub effect: TaintTolerationEffect,
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    #[serde(default)]
    pub value: String,
    pub operator: TaintTolerationOperator,
    pub effect: TaintTolerationEffect,
}


impl Taint {
    //
    // A toleration "matches" a taint if the keys are the same and the effects are the same, and:
    //   - the operator is Exists (in which case no value should be specified), or
    //   - the operator is Equal and the values should be equal.
    //
    // There are one special case:
    //   - An empty key with operator Exists matches all keys, values and effects which means this will tolerate everything.
    //
    pub fn matches(&self, tol: &Toleration) -> bool {
        // Special case with empty key
        if tol.key.is_empty() && tol.operator == TaintTolerationOperator::Exists {
            return true;
        }

        // Keys and effects should be equal
        if self.key != tol.key || self.effect != tol.effect {
            return false;
        }

        if tol.operator == TaintTolerationOperator::Exists
            || (tol.operator == TaintTolerationOperator::Equal && self.value == tol.value) {
            return true;
        }
        return false;
    }
}
