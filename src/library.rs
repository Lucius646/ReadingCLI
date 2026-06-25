use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::metadata::BookMetadata;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookLibrary {
    pub current_book_path: PathBuf,
    pub books: Vec<BookMetadata>,
}

impl BookLibrary {
    // 创建默认书架，用 default.txt 作为无书目时的当前书。
    pub fn new(default_book_path: PathBuf) -> Self {
        let default_book = BookMetadata::new(default_book_path.clone());

        Self {
            current_book_path: default_book_path,
            books: vec![default_book],
        }
    }

    // 返回当前书的元数据。
    pub fn current_book(&self) -> Option<BookMetadata> {
        let current_book_key = book_path_key(&self.current_book_path);

        self.books
            .iter()
            .find(|book| book_path_key(&book.book_path) == current_book_key)
            .cloned()
    }

    // 插入新书，或用同一路径身份的新元数据覆盖旧记录。
    pub fn upsert_book(&mut self, book: BookMetadata) {
        let book_key = book_path_key(&book.book_path);

        if let Some(existing_book) = self
            .books
            .iter_mut()
            .find(|existing_book| book_path_key(&existing_book.book_path) == book_key)
        {
            *existing_book = book;
            return;
        }

        self.books.push(book);
    }

    // 激活一本书：已有记录则复用进度，新书则创建默认元数据。
    pub fn activate_book(&mut self, book_path: PathBuf, opened_at: u64) -> BookMetadata {
        let book_key = book_path_key(&book_path);
        let mut book = self
            .books
            .iter()
            .find(|book| book_path_key(&book.book_path) == book_key)
            .cloned()
            .unwrap_or_else(|| BookMetadata::new(book_path.clone()));

        book.last_opened_at = opened_at;
        self.current_book_path = book.book_path.clone();
        self.upsert_book(book.clone());

        book
    }

    // 返回 Select 页面可展示的书目，过滤 default.txt 并按最近打开时间排序。
    pub fn visible_books(&self, default_book_path: &PathBuf) -> Vec<BookMetadata> {
        let mut books = self
            .books
            .iter()
            .filter(|book| book.book_path != *default_book_path)
            .cloned()
            .collect::<Vec<_>>();

        books.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
        books
    }

    // 合并历史书架里的重复路径记录，例如同一本书的相对路径和绝对路径。
    pub fn deduplicate_books(&mut self) {
        let mut merged_books: Vec<BookMetadata> = Vec::new();

        for book in self.books.drain(..) {
            let book_key = book_path_key(&book.book_path);

            if let Some(existing_book) = merged_books
                .iter_mut()
                .find(|existing_book| book_path_key(&existing_book.book_path) == book_key)
            {
                merge_book_metadata(existing_book, book);
            } else {
                merged_books.push(book);
            }
        }

        self.books = merged_books;
    }
}

// 从 library.json 读取书架；文件不存在时返回 None。
pub fn load_library(path: &PathBuf) -> Result<Option<BookLibrary>> {
    if !fs::exists(path)? {
        return Ok(None);
    }

    let json = fs::read_to_string(path)?;
    let mut library: BookLibrary = serde_json::from_str(&json)?;
    library.deduplicate_books();

    Ok(Some(library))
}

// 读取 library.json；如果不存在，则从旧 current-book.json 迁移或创建默认书架。
pub fn load_or_migrate_library(
    library_path: &PathBuf,
    legacy_metadata_path: &PathBuf,
    default_book_path: PathBuf,
) -> Result<BookLibrary> {
    if let Some(library) = load_library(library_path)? {
        return Ok(library);
    }

    if fs::exists(legacy_metadata_path)? {
        let json = fs::read_to_string(legacy_metadata_path)?;
        let metadata: BookMetadata = serde_json::from_str(&json)?;

        let mut library = BookLibrary::new(default_book_path);
        library.current_book_path = metadata.book_path.clone();
        library.upsert_book(metadata);

        return Ok(library);
    }

    Ok(BookLibrary::new(default_book_path))
}

// 把书架保存为格式化 JSON。
pub fn save_library(path: &PathBuf, library: &BookLibrary) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(library)?;
    fs::write(path, json)?;

    Ok(())
}

// 返回当前 Unix 秒时间戳，用于记录最近打开时间。
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn book_path_key(path: &Path) -> String {
    let normalized_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let key = normalized_path.to_string_lossy().replace('\\', "/");

    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    }
}

fn merge_book_metadata(existing_book: &mut BookMetadata, incoming_book: BookMetadata) {
    if incoming_book.last_opened_at >= existing_book.last_opened_at {
        *existing_book = incoming_book;
        return;
    }

    if existing_book.current_offset == 0 && incoming_book.current_offset != 0 {
        existing_book.current_offset = incoming_book.current_offset;
    }

    if existing_book.file_len == 0 && incoming_book.file_len != 0 {
        existing_book.file_len = incoming_book.file_len;
    }
}
