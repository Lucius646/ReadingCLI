#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverOp {
    Write(String),
    NewLine,
    EraseLine,
    CursorUp(usize),
}
