use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookMetadata {
    pub book_path: PathBuf,
    pub current_block: usize,
    pub block_size: usize,
}
