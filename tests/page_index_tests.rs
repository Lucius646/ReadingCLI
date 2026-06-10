use reading_cli::{page_index::PageIndex};
use anyhow::Result;
use reading_cli::text_source::TextSource;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn page_index_can_report_page_count() {
    let index = PageIndex {
        columns: 100,
        rows: 28,
        page_starts: vec![0, 1200, 2400],
    } ;
    assert_eq!(index.page_count(), 3);
}

#[test]
fn page_index_finds_page_by_offset() {
    let index = PageIndex {
        columns: 100,
        rows: 28,
        page_starts: vec![0, 100, 200],
    };

    assert_eq!(index.find_page_by_offset(0), 0);
    assert_eq!(index.find_page_by_offset(50), 0);
    assert_eq!(index.find_page_by_offset(100), 1);
    assert_eq!(index.find_page_by_offset(199), 1);
    assert_eq!(index.find_page_by_offset(200), 2);
    assert_eq!(index.find_page_by_offset(999), 2);
}

#[test]
fn page_index_returns_page_start_by_page_number() {
    let index = PageIndex {
        columns: 100,
        rows: 28,
        page_starts: vec![0, 100, 200],
    };
    assert_eq!(index.page_start(0), Some(0));
    assert_eq!(index.page_start(1), Some(100));
    assert_eq!(index.page_start(2), Some(200));
    assert_eq!(index.page_start(3), None);
}

#[test]
fn page_index_builds_from_text_source() -> Result<()> {
    let file = NamedTempFile::new()?;
    fs::write(file.path(), "一二三四五六七八九十")?;
    
    let text_source = TextSource::new(file.path().to_path_buf(), 1200, 10)?;

    let index = PageIndex::build(&text_source, 4, 2)?;

    assert_eq!(index.page_starts, vec![0, 12, 24]);

    Ok(())
}