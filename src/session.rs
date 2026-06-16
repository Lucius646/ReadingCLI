use crate::metadata::BookMetadata;

#[derive(Debug)]
pub struct ReadingSession {
    pub metadata: BookMetadata,
    pub running: bool,
}

impl ReadingSession {
    // 根据书籍元数据创建一次运行中的阅读会话。
    pub fn new(metadata: BookMetadata) -> Self {
        Self {
            metadata,
            running: true,
        }
    }

    // 标记会话退出，让 TUI 主循环结束。
    pub fn quit(&mut self) {
        self.running = false;
    }
}
