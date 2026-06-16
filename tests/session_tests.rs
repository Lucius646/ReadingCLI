use std::path::PathBuf;

use reading_cli::metadata::BookMetadata;
use reading_cli::session::ReadingSession;

#[test]
fn session_starts_running() {
    let metadata = BookMetadata {
        book_path: PathBuf::from("novel.txt"),
        current_offset: 0,
        last_opened_at: 0,
        file_len: 0,
    };

    let session = ReadingSession::new(metadata);

    assert!(session.running);
    assert_eq!(session.metadata.current_offset, 0);
}

#[test]
fn session_can_quit() {
    let metadata = BookMetadata {
        book_path: PathBuf::from("novel.txt"),
        current_offset: 0,
        last_opened_at: 0,
        file_len: 0,
    };

    let mut session = ReadingSession::new(metadata);

    assert!(session.running);

    session.quit();

    assert!(!session.running);
}
