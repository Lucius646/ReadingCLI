use crate::metadata::BookMetadata;

#[derive(Debug)]
pub struct ReadingSession {
    pub metadata: BookMetadata,
    pub running: bool,
    page_history: Vec<u64>,
}

impl ReadingSession {
    pub fn new(metadata: BookMetadata) -> Self {
        Self {
            metadata,
            running: true,
            page_history: Vec::new(),
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

    pub fn move_to_offset(&mut self, next_offset: u64) {
        if next_offset != self.metadata.current_offset {
            self.page_history.push(self.metadata.current_offset);
            self.metadata.current_offset = next_offset;
        }
    }

    pub fn previous_offset(&mut self) {
        if let Some(previous_offset) = self.page_history.pop() {
            self.metadata.current_offset = previous_offset;
        }
    }
}
