use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::library::{BookLibrary, current_timestamp, load_or_migrate_library, save_library};
use crate::metadata::BookMetadata;
use crate::session::ReadingSession;
use crate::text_source::TextSource;
use crate::tui;

const DEFAULT_BOOK_PATH: &str = "default.txt";
const LIBRARY_PATH: &str = ".reading/library.json";
const LEGACY_METADATA_PATH: &str = ".reading/current-book.json";

// 应用主流程：读取书架状态，打开当前书，进入 TUI，退出后保存书架。
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut library = load_or_migrate_library(
        &PathBuf::from(LIBRARY_PATH),
        &PathBuf::from(LEGACY_METADATA_PATH),
        PathBuf::from(DEFAULT_BOOK_PATH),
    )?;

    let metadata = match cli.command {
        Some(Command::Open { path }) => open_book_metadata(path, &mut library),
        None => Ok(library
            .current_book()
            .unwrap_or_else(|| BookMetadata::new(PathBuf::from(DEFAULT_BOOK_PATH)))),
    }?;

    let mut session = ReadingSession::new(metadata);
    let mut text_source = TextSource::new(session.metadata.book_path.clone())?;
    session.metadata.file_len = text_source.file_len();

    tui::run_reader(&mut session, &mut text_source, &mut library)?;

    library.current_book_path = session.metadata.book_path.clone();
    library.upsert_book(session.metadata.clone());
    save_library(&PathBuf::from(LIBRARY_PATH), &library)?;

    println!("see u again!");

    Ok(())
}

// 通过命令行路径激活一本书，并复用书架里已有的阅读进度。
fn open_book_metadata(path: PathBuf, library: &mut BookLibrary) -> Result<BookMetadata> {
    let book_path = normalize_book_path(path);

    Ok(library.activate_book(book_path, current_timestamp()))
}

// 尽量把路径标准化为绝对路径；失败时保留原路径。
fn normalize_book_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}
