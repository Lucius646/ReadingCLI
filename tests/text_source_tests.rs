use std::fs;

use encoding_rs::GBK;
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

#[test]
fn text_source_reads_text_from_byte_offsets() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join("novel.txt");

    fs::write(&file_path, "一二三四五六")?;

    let source = TextSource::new(file_path, 4, 1)?;

    let offset = "一二".len() as u64;
    let text = source.read_from_offset(offset, 1024)?;

    assert_eq!(text, "三四五六");
    
    Ok(())
}

#[test]
fn text_source_reads_gbk_text_as_utf8() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join("gbk-novel.txt");

    let text = "\u{4e00}\u{4e8c}\u{4e09}\u{56db}\u{4e94}\u{516d}";
    let (gbk_bytes, _, _) = GBK.encode(text);
    fs::write(&file_path, gbk_bytes.as_ref())?;

    let mut source = TextSource::new(file_path, 4, 1)?;

    assert_eq!(
        source.read_block(0)?,
        "\u{4e00}\u{4e8c}\u{4e09}\u{56db}"
    );
    assert_eq!(source.read_block(1)?, "\u{4e94}\u{516d}");

    Ok(())
}
