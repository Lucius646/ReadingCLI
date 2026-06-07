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

            let mut text_source = TextSource::new(
                session.metadata.book_path.clone(),
                session.metadata.block_size,
                10,
            )?;

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
    
    if fs::exists(metadata_path)? {
        let json = fs::read_to_string(metadata_path)?;
        let metadata = serde_json::from_str(&json)?;
        Ok(metadata) 
    } else {
        Ok(BookMetadata { 
            book_path: path, 
            current_block: 0, 
            current_offset: 0,
            block_size: 1200,
        })
    }
}
