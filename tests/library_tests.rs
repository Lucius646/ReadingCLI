use std::path::PathBuf;

use anyhow::Result;
use reading_cli::library::{BookLibrary, load_library, load_or_migrate_library, save_library};
use reading_cli::metadata::BookMetadata;
use tempfile::tempdir;

#[test]
fn library_starts_with_default_book() {
    let default_path = PathBuf::from("default.txt");
    let library = BookLibrary::new(default_path.clone());

    assert_eq!(library.current_book_path, default_path);
    assert_eq!(library.books.len(), 1);
    assert_eq!(library.current_book().unwrap().book_path, default_path);
}

#[test]
fn library_upsert_replaces_existing_book() {
    let default_path = PathBuf::from("default.txt");
    let book_path = PathBuf::from("novel.txt");
    let mut library = BookLibrary::new(default_path);

    library.upsert_book(BookMetadata::new(book_path.clone()));

    let mut updated_book = BookMetadata::new(book_path.clone());
    updated_book.current_offset = 120;
    updated_book.last_opened_at = 10;
    library.upsert_book(updated_book);

    let saved_book = library
        .books
        .iter()
        .find(|book| book.book_path == book_path)
        .unwrap();

    assert_eq!(library.books.len(), 2);
    assert_eq!(saved_book.current_offset, 120);
    assert_eq!(saved_book.last_opened_at, 10);
}

#[test]
fn visible_books_filters_default_and_sorts_by_last_opened() {
    let default_path = PathBuf::from("default.txt");
    let mut library = BookLibrary::new(default_path.clone());

    let mut older_book = BookMetadata::new(PathBuf::from("older.txt"));
    older_book.last_opened_at = 1;

    let mut newer_book = BookMetadata::new(PathBuf::from("newer.txt"));
    newer_book.last_opened_at = 2;

    library.upsert_book(older_book);
    library.upsert_book(newer_book);

    let visible_books = library.visible_books(&default_path);

    assert_eq!(visible_books.len(), 2);
    assert_eq!(visible_books[0].book_path, PathBuf::from("newer.txt"));
    assert_eq!(visible_books[1].book_path, PathBuf::from("older.txt"));
}

#[test]
fn activate_book_reuses_existing_offset() {
    let default_path = PathBuf::from("default.txt");
    let book_path = PathBuf::from("novel.txt");
    let mut library = BookLibrary::new(default_path);
    let mut book = BookMetadata::new(book_path.clone());
    book.current_offset = 256;
    book.last_opened_at = 1;
    library.upsert_book(book);

    let activated_book = library.activate_book(book_path.clone(), 2);

    assert_eq!(activated_book.book_path, book_path);
    assert_eq!(activated_book.current_offset, 256);
    assert_eq!(activated_book.last_opened_at, 2);
    assert_eq!(library.current_book_path, book_path);
}

#[test]
fn activate_book_adds_new_book() {
    let default_path = PathBuf::from("default.txt");
    let book_path = PathBuf::from("novel.txt");
    let mut library = BookLibrary::new(default_path);

    let activated_book = library.activate_book(book_path.clone(), 2);

    assert_eq!(activated_book.book_path, book_path);
    assert_eq!(activated_book.current_offset, 0);
    assert_eq!(activated_book.last_opened_at, 2);
    assert_eq!(library.current_book_path, book_path);
    assert_eq!(library.books.len(), 2);
}

#[test]
fn library_can_be_saved_and_loaded() -> Result<()> {
    let temp_dir = tempdir()?;
    let library_path = temp_dir.path().join(".reading").join("library.json");
    let mut library = BookLibrary::new(PathBuf::from("default.txt"));

    library.upsert_book(BookMetadata::new(PathBuf::from("novel.txt")));

    save_library(&library_path, &library)?;

    let loaded_library = load_library(&library_path)?.unwrap();

    assert_eq!(
        loaded_library.current_book_path,
        PathBuf::from("default.txt")
    );
    assert_eq!(loaded_library.books.len(), 2);

    Ok(())
}

#[test]
fn loading_missing_library_returns_none() -> Result<()> {
    let temp_dir = tempdir()?;
    let library_path = temp_dir.path().join(".reading").join("library.json");

    let library = load_library(&library_path)?;

    assert!(library.is_none());

    Ok(())
}

#[test]
fn load_or_migrate_prefers_existing_library() -> Result<()> {
    let temp_dir = tempdir()?;
    let library_path = temp_dir.path().join(".reading").join("library.json");
    let legacy_path = temp_dir.path().join(".reading").join("current-book.json");
    let mut library = BookLibrary::new(PathBuf::from("default.txt"));
    let library_book = BookMetadata::new(PathBuf::from("library-book.txt"));
    let legacy_book = BookMetadata::new(PathBuf::from("legacy-book.txt"));

    library.current_book_path = library_book.book_path.clone();
    library.upsert_book(library_book);
    save_library(&library_path, &library)?;

    let legacy_json = serde_json::to_string_pretty(&legacy_book)?;
    std::fs::write(&legacy_path, legacy_json)?;

    let loaded_library =
        load_or_migrate_library(&library_path, &legacy_path, PathBuf::from("default.txt"))?;

    assert_eq!(
        loaded_library.current_book_path,
        PathBuf::from("library-book.txt")
    );

    Ok(())
}

#[test]
fn load_or_migrate_uses_legacy_metadata_when_library_is_missing() -> Result<()> {
    let temp_dir = tempdir()?;
    let library_path = temp_dir.path().join(".reading").join("library.json");
    let legacy_path = temp_dir.path().join(".reading").join("current-book.json");
    let mut legacy_book = BookMetadata::new(PathBuf::from("legacy-book.txt"));
    legacy_book.current_offset = 42;

    std::fs::create_dir_all(legacy_path.parent().unwrap())?;
    let legacy_json = serde_json::to_string_pretty(&legacy_book)?;
    std::fs::write(&legacy_path, legacy_json)?;

    let loaded_library =
        load_or_migrate_library(&library_path, &legacy_path, PathBuf::from("default.txt"))?;

    assert_eq!(
        loaded_library.current_book_path,
        PathBuf::from("legacy-book.txt")
    );
    assert_eq!(loaded_library.current_book().unwrap().current_offset, 42);

    Ok(())
}

#[test]
fn load_or_migrate_uses_default_when_no_storage_exists() -> Result<()> {
    let temp_dir = tempdir()?;
    let library_path = temp_dir.path().join(".reading").join("library.json");
    let legacy_path = temp_dir.path().join(".reading").join("current-book.json");

    let loaded_library =
        load_or_migrate_library(&library_path, &legacy_path, PathBuf::from("default.txt"))?;

    assert_eq!(
        loaded_library.current_book_path,
        PathBuf::from("default.txt")
    );
    assert_eq!(
        loaded_library.current_book().unwrap().book_path,
        PathBuf::from("default.txt")
    );

    Ok(())
}
