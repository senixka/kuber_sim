use crate::my_imports::*;


pub trait IActiveQ {
    fn push(&mut self, pod: Pod);
    fn try_pop(&mut self) -> Option<Pod>;
    fn try_remove(&mut self, pod: Pod) -> bool;
    fn len(&self) -> usize;
}

pub type ActiveQDefault = ActiveMinQ<ActiveQCmpDefault>;


////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ActiveMinQ<PodWrapper: TraitActiveQCmp> (pub BTreeSet<PodWrapper>);

impl<PodWrapper: TraitActiveQCmp> ActiveMinQ<PodWrapper> {
    pub fn new() -> Self {
        Self {
            0: BTreeSet::new()
        }
    }

    pub fn default() -> Self {
        Self {
            0: BTreeSet::default()
        }
    }
}

impl<PodWrapper: TraitActiveQCmp> IActiveQ for ActiveMinQ<PodWrapper> {
    fn push(&mut self, pod: Pod) {
        self.0.insert(PodWrapper::wrap(pod));
    }

    fn try_pop(&mut self) -> Option<Pod> {
        return match self.0.pop_first() {
            Some(wrapper) => {
                Some(wrapper.inner())
            }
            None => {
                None
            }
        }
    }

    fn try_remove(&mut self, pod: Pod) -> bool {
        return self.0.remove(&PodWrapper::wrap(pod));
    }

    fn len(&self) -> usize {
        return self.0.len();
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ActiveMaxQ<PodWrapper: TraitActiveQCmp> (pub BTreeSet<PodWrapper>);

impl<PodWrapper: TraitActiveQCmp> ActiveMaxQ<PodWrapper> {
    pub fn new() -> Self {
        Self {
            0: BTreeSet::new()
        }
    }

    pub fn default() -> Self {
        Self {
            0: BTreeSet::default()
        }
    }
}

impl<PodWrapper: TraitActiveQCmp> IActiveQ for ActiveMaxQ<PodWrapper> {
    fn push(&mut self, pod: Pod) {
        self.0.insert(PodWrapper::wrap(pod));
    }

    fn try_pop(&mut self) -> Option<Pod> {
        return match self.0.pop_last() {
            Some(wrapper) => {
                Some(wrapper.inner())
            }
            None => {
                None
            }
        }
    }

    fn try_remove(&mut self, pod: Pod) -> bool {
        return self.0.remove(&PodWrapper::wrap(pod));
    }

    fn len(&self) -> usize {
        return self.0.len();
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
