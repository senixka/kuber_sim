use crate::my_imports::*;

// https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/#operators
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeAffinityOperator {
    /// In = The label value is present in the supplied set of strings.
    #[default]
    In = 0,
    /// NotIn = The label value is not contained in the supplied set of strings.
    NotIn = 1,
    /// Exists = A label with this key exists on the object.
    Exists = 2,
    /// DoNotExist = No label with this key exists on the object.
    DoesNotExist = 3,
    /// Gt = True if the specified value is LESS than the value on the node.
    /// The supplied value will be parsed as an integer.
    Gt = 4,
    /// Lt = True if the specified value is GRATER than the value on the node.
    /// The supplied value will be parsed as an integer.
    Lt = 5,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinityMatchExpression {
    /// The label key that the selector applies to.
    pub key: String,
    /// Represents a key's relationship to a set of values.
    /// Valid operators are In, NotIn, Exists, DoesNotExist. Gt, and Lt.
    pub operator: NodeAffinityOperator,
    /// An array of string values.
    /// If the operator is In or NotIn, the values array must be non-empty.
    /// If the operator is Exists or DoesNotExist, the values array must be empty.
    /// If the operator is Gt or Lt, the values array must have a single element, which will be interpreted as an integer.
    #[serde(default)]
    pub values: Vec<String>,
}

impl NodeAffinityMatchExpression {
    pub fn matches(&self, node: &Node) -> bool {
        let value = node.metadata.labels.get(&self.key);

        return match value {
            Some(node_value) => match self.operator {
                NodeAffinityOperator::Exists => true,
                NodeAffinityOperator::DoesNotExist => false,
                NodeAffinityOperator::In => self.values.contains(node_value),
                NodeAffinityOperator::NotIn => !self.values.contains(node_value),
                NodeAffinityOperator::Gt => {
                    let pod_int = self.values[0].parse::<i64>().unwrap();
                    match node_value.parse::<i64>() {
                        Ok(node_int) => pod_int < node_int,
                        Err(_) => false,
                    }
                }
                NodeAffinityOperator::Lt => {
                    let pod_int = self.values[0].parse::<i64>().unwrap();
                    match node_value.parse::<i64>() {
                        Ok(node_int) => pod_int > node_int,
                        Err(_) => false,
                    }
                }
            },
            None => match self.operator {
                NodeAffinityOperator::In
                | NodeAffinityOperator::Exists
                | NodeAffinityOperator::Gt
                | NodeAffinityOperator::Lt => false,
                NodeAffinityOperator::NotIn | NodeAffinityOperator::DoesNotExist => true,
            },
        };
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinityPreferredTerm {
    /// A list of node selector preferences by node's labels.
    /// The terms are ANDed.
    pub node_selector_term: Vec<NodeAffinityMatchExpression>,
    /// Weight associated with matching the corresponding node_selector_term, in the range 1-100.
    pub weight: i64,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinitySelectorTerm {
    /// A list of node selector requirements by node's labels.
    /// The terms are ANDed.
    pub node_selector_term: Vec<NodeAffinityMatchExpression>,
}

// https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinity {
    /// Analog of preferredDuringSchedulingIgnoredDuringExecution.
    pub preferred_terms: Vec<NodeAffinityPreferredTerm>,
    /// Analog of requiredDuringSchedulingIgnoredDuringExecution.
    /// The terms are ORed.
    pub required_terms: Vec<NodeAffinitySelectorTerm>,
}

impl NodeAffinity {
    /// Is required NodeAffinitySelectorTerms matches node.
    pub fn is_required_matches(&self, node: &Node) -> bool {
        for match_expression in &self.required_terms {
            let mut flag = true;
            for expression in &match_expression.node_selector_term {
                if !expression.matches(node) {
                    flag = false;
                    break;
                }
            }

            if flag {
                return true;
            }
        }

        return self.required_terms.len() == 0;
    }

    /// Counts weighted sum of preferred NodeAffinitySelectorTerms that matches node.
    pub fn preferred_sum(&self, node: &Node) -> i64 {
        let mut match_sum = 0;
        for match_expression in &self.preferred_terms {
            let mut flag = 1;
            for expression in &match_expression.node_selector_term {
                if !expression.matches(node) {
                    flag = 0;
                    break;
                }
            }
            match_sum += flag * match_expression.weight;
        }

        return match_sum;
    }
}
