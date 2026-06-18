use std::time::Duration;

use super::CoverOp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverFrame {
    pub ops: Vec<CoverOp>,
    pub delay: Duration,
}

impl CoverFrame {
    pub fn new(ops: Vec<CoverOp>, delay: Duration) -> Self {
        Self { ops, delay }
    }
}
