use crate::my_imports::*;


pub type ScoreNormalizePluginF = fn (&HashMap<u64, Pod>,
                                     &HashMap<u64, Pod>,
                                     &HashMap<u64, Node>,
                                     &Pod,
                                     &Vec<Node>,
                                     &mut Vec<i64>);


pub trait IScoreNormalizePlugin {
    fn name(&self) -> String;

    fn normalize(&self, 
                 running_pods: &HashMap<u64, Pod>, 
                 pending_pods: &HashMap<u64, Pod>, 
                 all_nodes: &HashMap<u64, Node>, 
                 pod: &Pod, 
                 score_nodes: &Vec<Node>,
                 scores: &mut Vec<i64>);
}


////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreNormalizeSkip;

impl IScoreNormalizePlugin for ScoreNormalizeSkip {
    fn name(&self) -> String {
        return "ScoreNormalizeSkip".to_string();
    }

    fn normalize(&self,
                 _: &HashMap<u64, Pod>,
                 _: &HashMap<u64, Pod>,
                 _: &HashMap<u64, Node>,
                 _: &Pod,
                 _: &Vec<Node>,
                 _: &mut Vec<i64>) {
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreNormalizeNeg;

impl IScoreNormalizePlugin for ScoreNormalizeNeg {
    fn name(&self) -> String {
        return "ScoreNormalizeNeg".to_string();
    }

    fn normalize(&self,
                 _: &HashMap<u64, Pod>,
                 _: &HashMap<u64, Pod>,
                 _: &HashMap<u64, Node>,
                 _: &Pod,
                 _: &Vec<Node>,
                 scores: &mut Vec<i64>) {
        for score in scores {
            *score = -(*score);
        }
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
