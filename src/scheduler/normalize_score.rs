use std::ops::Neg;
use crate::my_imports::*;

pub type NormalizeScorePluginT = fn (&HashMap<u64, Pod>,
                                     &HashMap<u64, Pod>,
                                     &HashMap<u64, Node>,
                                     &Pod,
                                     &Vec<Node>,
                                     &mut Vec<i64>);

pub fn skip(_: &HashMap<u64, Pod>,
            _: &HashMap<u64, Pod>,
            _: &HashMap<u64, Node>,
            _: &Pod,
            _: &Vec<Node>,
            _: &mut Vec<i64>) {
}

pub fn neg(_: &HashMap<u64, Pod>,
           _: &HashMap<u64, Pod>,
           _: &HashMap<u64, Node>,
           _: &Pod,
           _: &Vec<Node>,
           scores: &mut Vec<i64>) {
    for score in scores {
        *score = score.neg();
    }
}
