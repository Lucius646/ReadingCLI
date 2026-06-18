use super::{CoverContext, CoverFrame};

pub trait CoverModule {
    fn name(&self) -> &'static str;

    fn signature(&self) -> String;

    fn next_frame(&mut self, context: &CoverContext) -> Option<CoverFrame>;
}
