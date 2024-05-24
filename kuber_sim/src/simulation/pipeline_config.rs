use crate::scheduler::pipeline::filter::IFilterPlugin;
use crate::scheduler::pipeline::score::IScorePlugin;
use crate::scheduler::pipeline::score_normalize::IScoreNormalizePlugin;
use crate::scheduler::queues::active_queue::IActiveQ;
use crate::scheduler::queues::backoff_queue::IBackOffQ;

pub struct PipelineConfig {
    pub active_queue: Box<dyn IActiveQ + Send>,
    pub backoff_queue: Box<dyn IBackOffQ + Send>,

    pub filters: Vec<Box<dyn IFilterPlugin + Send>>,
    pub post_filters: Vec<Box<dyn IFilterPlugin + Send>>,
    pub scorers: Vec<Box<dyn IScorePlugin + Send>>,
    pub score_normalizers: Vec<Box<dyn IScoreNormalizePlugin + Send>>,
    pub scorer_weights: Vec<i64>,
}

impl PipelineConfig {
    pub fn new(
        active_queue: Box<dyn IActiveQ + Send>,
        backoff_queue: Box<dyn IBackOffQ + Send>,

        filters: Vec<Box<dyn IFilterPlugin + Send>>,
        post_filters: Vec<Box<dyn IFilterPlugin + Send>>,
        scorers: Vec<Box<dyn IScorePlugin + Send>>,
        score_normalizers: Vec<Box<dyn IScoreNormalizePlugin + Send>>,
        scorer_weights: Vec<i64>,
    ) -> Self {
        Self {
            active_queue,
            backoff_queue,
            filters,
            post_filters,
            scorers,
            score_normalizers,
            scorer_weights,
        }
    }
}

impl Clone for PipelineConfig {
    fn clone(&self) -> Self {
        Self {
            active_queue: (*self.active_queue).clone(),
            backoff_queue: (*self.backoff_queue).clone(),
            filters: self.filters.iter().map(|x| (*x).clone()).collect(),
            post_filters: self.post_filters.iter().map(|x| (*x).clone()).collect(),
            scorers: self.scorers.iter().map(|x| (*x).clone()).collect(),
            score_normalizers: self.score_normalizers.iter().map(|x| (*x).clone()).collect(),
            scorer_weights: self.scorer_weights.clone(),
        }
    }
}
