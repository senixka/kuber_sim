use crate::my_imports::*;


// https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeAffinityType {
    #[default]
    Required = 0,   // Analog of requiredDuringSchedulingIgnoredDuringExecution
    Preferred = 1,  // Analog of preferredDuringSchedulingIgnoredDuringExecution
}


// https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/#operators
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeAffinityOperator {
    #[default]
    In = 0,             // The label value is present in the supplied set of strings
    NotIn = 1,          // The label value is not contained in the supplied set of strings
    Exists = 2,         // A label with this key exists on the object
    DoesNotExist = 3,   // No label with this key exists on the object
    Gt = 4,             // The supplied value will be parsed as an integer. True if the specified value is LESS than the value on the node
    Lt = 5,             // The supplied value will be parsed as an integer. True if the specified value is GRATER than the value on the node
}


// https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeAffinityExpressionType {
    #[default]
    MatchExpression = 0,    // Scheduled only if ALL rules are satisfied
    NodeSelectorTerms = 1,  // Scheduled to a node if ANY defined conditions match
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinityExpression {
    pub key: String,
    pub operator: NodeAffinityOperator,

    #[serde(default)]
    pub values: Vec<String>,
}


impl NodeAffinityExpression {
    pub fn matches(&self, node: &Node) -> bool {
        let value = node.metadata.labels.get(&self.key);

        return match value {
            Some(node_value) => {
                match self.operator {
                    NodeAffinityOperator::Exists => {
                        true
                    }
                    NodeAffinityOperator::DoesNotExist => {
                        false
                    }
                    NodeAffinityOperator::In => {
                        self.values.contains(node_value)
                    }
                    NodeAffinityOperator::NotIn => {
                        !self.values.contains(node_value)
                    }
                    NodeAffinityOperator::Gt => {
                        let pod_int = self.values[0].parse::<i64>().unwrap();
                        match node_value.parse::<i64>() {
                            Ok(node_int) => {
                                pod_int < node_int
                            }
                            Err(_) => {
                                false
                            }
                        }
                    }
                    NodeAffinityOperator::Lt => {
                        let pod_int = self.values[0].parse::<i64>().unwrap();
                        match node_value.parse::<i64>() {
                            Ok(node_int) => {
                                pod_int > node_int
                            }
                            Err(_) => {
                                false
                            }
                        }
                    }
                }
            }
            None => {
                match self.operator {
                    NodeAffinityOperator::In | NodeAffinityOperator::Exists
                    | NodeAffinityOperator::Gt | NodeAffinityOperator::Lt => {
                        false
                    }
                    NodeAffinityOperator::NotIn | NodeAffinityOperator::DoesNotExist => {
                        true
                    }
                }
            }
        }
    }
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MatchExpression {
    pub match_expression: Vec<NodeAffinityExpression>,

    // TODO: weight
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinity {
    // NodeSelectorTerms contains multiple MatchExpression, while each MatchExpression contains multiple Expressions.
    // Entries in NodeSelectorTerms use OR (scheduled to a node if ANY defined MatchExpression match).
    // Entries in MatchExpression use AND (MatchExpression matches if ALL inner expressions are match).
    pub node_selector_terms: Vec<MatchExpression>, // NodeSelectorTerms<MatchExpression<NodeAffinityExpression>>

    // Required or Preferred
    pub schedule_type: NodeAffinityType,
}


impl NodeAffinity {
    pub fn matches(&self, node: &Node) -> usize {
        let mut match_count = 0;
        for match_expression in &self.node_selector_terms {
            match_count += 1;
            for expression in &match_expression.match_expression {
                if !expression.matches(node) {
                    match_count -= 1;
                    break;
                }
            }
        }

        return match_count;
    }
}
