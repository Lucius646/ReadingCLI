use std::fs;

use reading_cli::text_source::TextSource;

#[test]
fn read_block_returns_text_by_character_block() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join("novel.txt");

    fs::write(&file_path, "一二三四五六七八九十")?;

    let mut source = TextSource::new(file_path, 4, 1)?;

    assert_eq!(source.read_block(0)?, "一二三四");
    assert_eq!(source.read_block(1)?, "五六七八");
    assert_eq!(source.read_block(2)?, "九十");
    
    Ok(())
}
