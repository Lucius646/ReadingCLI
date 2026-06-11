use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookMetadata {
    pub book_path: PathBuf,
    #[serde(default)]
    pub current_offset: u64,
}

impl BookMetadata {
    pub fn new(book_path: PathBuf) -> Self {
        Self {
            book_path,
            current_offset: 0,
        }
    }
}
