use std::path::PathBuf;

use reading_cli::metadata::BookMetadata;
use reading_cli::session::ReadingSession;

#[test]
fn session_can_move_next_and_previous() {
    let metadata = BookMetadata {
        book_path: PathBuf::from("novel.txt"),
        current_block: 0,
        block_size: 1200,
    };

    let mut session = ReadingSession::new(metadata);

    session.next_block();
    assert_eq!(session.metadata.current_block, 1);

    session.previous_block();
    assert_eq!(session.metadata.current_block, 0);

    session.previous_block();
    assert_eq!(session.metadata.current_block, 0);
}

#[test]
fn session_can_quit() {
    let metadata = BookMetadata {
        book_path: PathBuf::from("novel.txt"),
        current_block: 0,
        block_size: 1200,
    };

    let mut session = ReadingSession::new(metadata);

    assert_eq!(session.running, true);

    session.quit();

    assert_eq!(session.running, false);
}
