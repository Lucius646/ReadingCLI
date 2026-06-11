use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::metadata::BookMetadata;
use crate::session::ReadingSession;
use crate::text_source::TextSource;
use crate::tui;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Open { path } => {
            let metadata = load_or_create_metadata(path)?;

            let mut session = ReadingSession::new(metadata);

            let mut text_source = TextSource::new(session.metadata.book_path.clone())?;

            tui::run_reader(&mut session, &mut text_source)?;

            fs::create_dir_all(".reading")?;

            let json = serde_json::to_string_pretty(&session.metadata)?;
            fs::write(".reading/current-book.json", json)?;
            println!("see u again!");
        }
    }

    Ok(())
}

fn load_or_create_metadata(path: PathBuf) -> Result<BookMetadata> {
    let metadata_path = ".reading/current-book.json";
    let book_path = normalize_book_path(path);

    if fs::exists(metadata_path)? {
        let json = fs::read_to_string(metadata_path)?;
        let metadata: BookMetadata = serde_json::from_str(&json)?;

        if normalize_book_path(metadata.book_path.clone()) == book_path {
            return Ok(metadata);
        }
    }

    Ok(BookMetadata::new(book_path))
}

fn normalize_book_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}
