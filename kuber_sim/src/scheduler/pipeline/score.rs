use crate::my_imports::*;

pub type ScorePluginF = fn(&HashMap<u64, Pod>, &HashMap<u64, Pod>, &HashMap<u64, Node>, &Pod, &Node) -> i64;

pub trait IScorePlugin {
    fn name(&self) -> String;

    fn score(
        &self,
        running_pods: &HashMap<u64, Pod>,
        pending_pods: &HashMap<u64, Pod>,
        nodes: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> i64;

    fn clone(&self) -> Box<dyn IScorePlugin + Send>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreIsNodeEmpty;

impl IScorePlugin for ScoreIsNodeEmpty {
    fn name(&self) -> String {
        return "ScoreIsNodeEmpty".to_string();
    }

    fn score(&self, _: &HashMap<u64, Pod>, _: &HashMap<u64, Pod>, _: &HashMap<u64, Node>, _: &Pod, node: &Node) -> i64 {
        return (node.status.pods.len() == 0) as i64;
    }

    fn clone(&self) -> Box<dyn IScorePlugin + Send> {
        return Box::new(ScoreIsNodeEmpty);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreCountRunningPods;

impl IScorePlugin for ScoreCountRunningPods {
    fn name(&self) -> String {
        return "ScoreCountRunningPods".to_string();
    }

    fn score(&self, _: &HashMap<u64, Pod>, _: &HashMap<u64, Pod>, _: &HashMap<u64, Node>, _: &Pod, node: &Node) -> i64 {
        return node.status.pods.len() as i64;
    }

    fn clone(&self) -> Box<dyn IScorePlugin + Send> {
        return Box::new(ScoreCountRunningPods);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreTetris;

impl IScorePlugin for ScoreTetris {
    fn name(&self) -> String {
        return "ScoreTetris".to_string();
    }

    fn score(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> i64 {
        let n_cpu = node.spec.available_cpu;
        let n_mem = node.spec.available_memory;
        let p_cpu = pod.spec.request_cpu;
        let p_mem = pod.spec.request_memory;
        let scale = 10000.0;

        return if n_cpu * p_mem >= n_mem * p_cpu {
            let y = (n_cpu * p_mem - n_mem * p_cpu) as f64;
            let x = (n_cpu * p_cpu + n_mem * p_mem) as f64;

            let angle: f64 = y.atan2(x);
            assert!(angle >= 0.0);

            let reversed: f64 = std::f64::consts::PI / 2.0 - angle;
            assert!(reversed >= 0.0);

            (reversed * scale) as i64
        } else {
            let y = (n_mem * p_cpu - n_cpu * p_mem) as f64;
            let x = (n_cpu * p_cpu + n_mem * p_mem) as f64;

            let angle: f64 = y.atan2(x);
            assert!(angle >= 0.0);

            let reversed: f64 = std::f64::consts::PI / 2.0 - angle;
            assert!(reversed >= 0.0);

            (reversed * scale) as i64
        };
    }

    fn clone(&self) -> Box<dyn IScorePlugin + Send> {
        return Box::new(ScoreTetris);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreTaintsTolerations;

impl IScorePlugin for ScoreTaintsTolerations {
    fn name(&self) -> String {
        return "ScoreTaintsTolerations".to_string();
    }

    fn score(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> i64 {
        let (mut no_schedule, mut prefer_no_schedule) = (false, false);
        for taint in node.spec.taints.iter() {
            let mut matches = false;
            for tol in pod.spec.tolerations.iter() {
                matches |= taint.matches(tol);
                if matches {
                    break;
                }
            }

            if !matches {
                match taint.effect {
                    TaintTolerationEffect::NoSchedule => {
                        no_schedule = true;
                    }
                    TaintTolerationEffect::PreferNoSchedule => {
                        prefer_no_schedule = true;
                    }
                }
            }
        }

        if no_schedule {
            return -1;
        }
        if prefer_no_schedule {
            return 0;
        }
        return 1;
    }

    fn clone(&self) -> Box<dyn IScorePlugin + Send> {
        return Box::new(ScoreTaintsTolerations);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ScoreNodeAffinity;

impl IScorePlugin for ScoreNodeAffinity {
    fn name(&self) -> String {
        return "ScoreNodeAffinity".to_string();
    }

    fn score(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> i64 {
        return pod.spec.node_affinity.preferred_sum(&node);
    }

    fn clone(&self) -> Box<dyn IScorePlugin + Send> {
        return Box::new(ScoreNodeAffinity);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
