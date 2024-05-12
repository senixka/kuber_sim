use crate::my_imports::*;

// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationEffect {
    /// No Pods will be scheduled on the tainted node unless they have a matching toleration.
    #[default]
    NoSchedule = 0,
    /// PreferNoSchedule is a "preference" or "soft" version of NoSchedule.
    /// The control plane will try to avoid placing a Pod that does not tolerate the taint on the node, but it is not guaranteed.
    PreferNoSchedule = 1,
}

// https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintTolerationOperator {
    /// Equal = A value of the label with this key on the object is equal to the supplied value.
    #[default]
    Equal = 0,
    /// Exists = A label with this key exists on the object.
    Exists = 1,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Taint {
    /// The taint key to be applied to a node.
    pub key: String,
    /// The taint value corresponding to the taint key.
    pub value: String,
    /// The effect of the taint on pods that do not tolerate the taint.
    pub effect: TaintTolerationEffect,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Toleration {
    /// Key is the taint key that the toleration applies to. Empty means match all taint keys.
    /// If the key is empty, operator must be Exists; this combination means to match all values and all keys.
    pub key: String,
    /// Value is the taint value the toleration matches to.
    #[serde(default)]
    pub value: String,
    /// Operator represents a key's relationship to the value.
    pub operator: TaintTolerationOperator,
    /// Effect indicates the taint effect to match.
    pub effect: TaintTolerationEffect,
}

impl Taint {
    ///
    /// A toleration "matches" a taint if the keys are the same and the effects are the same, and:
    ///   - the operator is Exists (in which case no value should be specified), or
    ///   - the operator is Equal and the values should be equal.
    ///
    /// There are one special case:
    ///   - An empty key with operator Exists matches all keys, values and effects which means this will tolerate everything.
    ///
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
            || (tol.operator == TaintTolerationOperator::Equal && self.value == tol.value)
        {
            return true;
        }
        return false;
    }
}
