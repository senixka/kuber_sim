use crate::my_imports::*;
use std::f64::consts::PI;


pub enum PluginScore {
    EmptyFirst,
    ByPodCount,
    Tetris,
}

impl PluginScore {
    #[inline]
    pub fn score(&self,
                          running_pods: &HashMap<u64, Pod>,
                          pending_pods: &HashMap<u64, Pod>,
                          all_nodes: &HashMap<u64, Node>,
                          pod: &Pod,
                          node: &Node) -> i64 {
        match self {
            PluginScore::EmptyFirst => { // higher values mean better
                if node.status.pods.len() == 0 { 1 } else { 0 }
            }
            PluginScore::ByPodCount => { // lower values mean better
                node.status.pods.len() as i64
            }
            PluginScore::Tetris => { // higher values mean better
                let n_cpu = node.spec.available_cpu;
                let n_mem = node.spec.available_memory;
                let p_cpu = pod.spec.request_cpu;
                let p_mem = pod.spec.request_memory;
                let scale = 10000;

                return if (n_cpu * p_mem >= n_mem * p_cpu) {
                    let y = (n_cpu * p_mem - n_mem * p_cpu) as f64;
                    let x = (n_cpu * p_cpu + n_mem * p_mem) as f64;

                    let angle: f64 = y.atan2(x);
                    assert!(angle >= 0.0);

                    let reversed: f64 = std::f64::consts::PI / 2.0 - angle;
                    assert!(reversed >= 0.0);

                    reversed as i64 * scale
                } else {
                    let y = (n_mem * p_cpu - n_cpu * p_mem) as f64;
                    let x = (n_cpu * p_cpu + n_mem * p_mem) as f64;

                    let angle: f64 = y.atan2(x);
                    assert!(angle >= 0.0);

                    let reversed: f64 = std::f64::consts::PI / 2.0 - angle;
                    assert!(reversed >= 0.0);

                    reversed as i64 * scale
                }
            }
        }
    }
}
