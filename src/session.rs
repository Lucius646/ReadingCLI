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

    pub fn next_block(&mut self) {
        self.metadata.current_block += 1;
    }

    pub fn previous_block(&mut self) {
        if self.metadata.current_block > 0 {
            self.metadata.current_block -= 1;
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
