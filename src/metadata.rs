use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookMetadata {
    pub book_path: PathBuf,
    pub current_block: usize,

    #[serde(default)]
    pub current_offset: u64,
    
    pub block_size: usize,
}
