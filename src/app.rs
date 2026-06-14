use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::metadata::BookMetadata;
use crate::session::ReadingSession;
use crate::text_source::TextSource;
use crate::tui;

const DEFAULT_BOOK_PATH: &str = "default.txt";

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Open { path }) => {
            let (mut session, mut text_source) = open_book(path)?;

            tui::run_reader(&mut session, &mut text_source)?;

            save_metadata(&session.metadata)?;
            println!("see u again!");
        }
        None => {
            let metadata = load_existing_metadata()?
                .unwrap_or_else(|| BookMetadata::new(PathBuf::from(DEFAULT_BOOK_PATH)));

            let mut session = ReadingSession::new(metadata);
            let mut text_source = TextSource::new(session.metadata.book_path.clone())?;

            tui::run_reader(&mut session, &mut text_source)?;

            save_metadata(&session.metadata)?;
            println!("see u again!");
        }
    }

    Ok(())
}

fn open_book(path: PathBuf) -> Result<(ReadingSession, TextSource)> {
    let metadata = load_or_create_metadata(path)?;
    let session = ReadingSession::new(metadata);
    let text_source = TextSource::new(session.metadata.book_path.clone())?;

    Ok((session, text_source))
}

fn load_existing_metadata() -> Result<Option<BookMetadata>> {
    let metadata_path = ".reading/current-book.json";

    if !fs::exists(metadata_path)? {
        return Ok(None);
    }

    let json = fs::read_to_string(metadata_path)?;
    let metadata = serde_json::from_str(&json)?;

    Ok(Some(metadata))
}

fn save_metadata(metadata: &BookMetadata) -> Result<()> {
    fs::create_dir_all(".reading")?;

    let json = serde_json::to_string_pretty(metadata)?;
    fs::write(".reading/current-book.json", json)?;

    Ok(())
}

fn load_or_create_metadata(path: PathBuf) -> Result<BookMetadata> {
    let book_path = normalize_book_path(path);

    if let Some(metadata) = load_existing_metadata()? {
        if normalize_book_path(metadata.book_path.clone()) == book_path {
            return Ok(metadata);
        }
    }

    Ok(BookMetadata::new(book_path))
}

fn normalize_book_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}
