use std::cmp::Ordering;
use crate::my_imports::*;

#[derive(Debug, Clone)]
pub struct ActiveQCmpUid(pub Pod);

impl PartialOrd for ActiveQCmpUid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActiveQCmpUid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.metadata.uid.cmp(&other.0.metadata.uid)
    }
}

impl PartialEq for ActiveQCmpUid {
    fn eq(&self, other: &Self) -> bool {
        self.0.metadata.uid == other.0.metadata.uid
    }
}

impl Eq for ActiveQCmpUid {}
