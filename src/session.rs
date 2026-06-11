use crate::metadata::BookMetadata;

#[derive(Debug)]
pub struct ReadingSession {
    pub metadata: BookMetadata,
    pub running: bool,
}

impl ReadingSession {
    pub fn new(metadata: BookMetadata) -> Self {
        Self {
            metadata,
            running: true,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
