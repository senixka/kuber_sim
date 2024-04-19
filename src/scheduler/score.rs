use crate::my_imports::*;


pub type ScorePluginT = fn(&HashMap<u64, Pod>,
                           &HashMap<u64, Pod>,
                           &HashMap<u64, Node>,
                           &Pod,
                           &Node) -> i64;

pub fn score_is_empty(_: &HashMap<u64, Pod>,
                      _: &HashMap<u64, Pod>,
                      _: &HashMap<u64, Node>,
                      _: &Pod,
                      node: &Node) -> i64 {
    return (node.status.pods.len() == 0) as i64;
}

pub fn score_running_pods(_: &HashMap<u64, Pod>,
                          _: &HashMap<u64, Pod>,
                          _: &HashMap<u64, Node>,
                          _: &Pod,
                          node: &Node) -> i64 {
    return node.status.pods.len() as i64;
}

pub fn score_tetris(_: &HashMap<u64, Pod>,
                    _: &HashMap<u64, Pod>,
                    _: &HashMap<u64, Node>,
                    pod: &Pod,
                    node: &Node) -> i64 {
    let n_cpu = node.spec.available_cpu;
    let n_mem = node.spec.available_memory;
    let p_cpu = pod.spec.request_cpu;
    let p_mem = pod.spec.request_memory;
    let scale = 10000;

    return if n_cpu * p_mem >= n_mem * p_cpu {
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
