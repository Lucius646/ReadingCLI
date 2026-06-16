use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookMetadata {
    pub book_path: PathBuf,
    #[serde(default)]
    pub current_offset: u64,
    #[serde(default)]
    pub last_opened_at: u64,
    #[serde(default)]
    pub file_len: u64,
}

impl BookMetadata {
    // 创建一本新书的默认元数据。
    pub fn new(book_path: PathBuf) -> Self {
        Self {
            book_path,
            current_offset: 0,
            last_opened_at: 0,
            file_len: 0,
        }
    }
}
