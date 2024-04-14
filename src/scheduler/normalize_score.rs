use std::ops::Neg;
use crate::my_imports::*;


pub enum PluginNormalizeScore {
    Skip,
    Neg,
}

impl PluginNormalizeScore {
    #[inline]
    pub fn normalize_score(&self,
                 running_pods: &HashMap<u64, Pod>,
                 pending_pods: &HashMap<u64, Pod>,
                 all_nodes: &HashMap<u64, Node>,
                 pod: &Pod,
                 nodes: &Vec<Node>,
                 scores: &mut Vec<i64>) {
        match self {
            PluginNormalizeScore::Skip => {
            }
            PluginNormalizeScore::Neg => {
                for score in scores {
                    score.neg();
                }
            }
        }
    }
}
