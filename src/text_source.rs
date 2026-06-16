use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::Result;
use encoding_rs::GBK;

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub struct TextSource {
    path: PathBuf,
    file_len: u64,
}

impl TextSource {
    // 打开文本文件，并统一转换成后续可按 UTF-8 offset 读取的文本源。
    pub fn new(path: PathBuf) -> Result<Self> {
        let raw_bytes = fs::read(&path)?;
        let decoded = decode_text(&raw_bytes);
        let path = if decoded.needs_utf8_cache {
            write_utf8_cache(&decoded.content)?
        } else {
            path
        };

        let content = decoded.content;
        let file_len = content.len() as u64;

        Ok(Self { path, file_len })
    }

    // 从指定 UTF-8 字节偏移读取一段合法字符串。
    pub fn read_from_offset(&self, offset: u64, max_bytes: usize) -> Result<String> {
        let file_len = self.file_len();

        if offset >= file_len {
            return Ok(String::new());
        }

        let read_len = (file_len - offset).min(max_bytes as u64) as usize;

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0; read_len];
        file.read_exact(&mut buffer)?;

        while String::from_utf8(buffer.clone()).is_err() {
            buffer.pop();

            if buffer.is_empty() {
                return Ok(String::new());
            }
        }

        let text = String::from_utf8(buffer)?;
        Ok(text)
    }

    // 从指定 offset 往前读取一段合法 UTF-8 字符串。
    pub fn read_before_offset(&self, offset: u64, max_bytes: usize) -> Result<(u64, String)> {
        let file_len = self.file_len();
        let end = offset.min(file_len);

        if end == 0 {
            return Ok((0, String::new()));
        }

        let start = end.saturating_sub(max_bytes as u64);
        let read_len = (end - start) as usize;

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(start))?;

        let mut buffer = vec![0; read_len];
        file.read_exact(&mut buffer)?;

        let mut actual_start = start;

        while String::from_utf8(buffer.clone()).is_err() {
            if buffer.is_empty() {
                return Ok((end, String::new()));
            }

            buffer.remove(0);
            actual_start += 1;
        }

        let text = String::from_utf8(buffer)?;
        Ok((actual_start, text))
    }

    // 返回当前 UTF-8 文本源的字节长度。
    pub fn file_len(&self) -> u64 {
        self.file_len
    }
}

struct DecodedText {
    content: String,
    needs_utf8_cache: bool,
}

// 将原始字节解码成 UTF-8 字符串，并标记是否需要写缓存文件。
fn decode_text(bytes: &[u8]) -> DecodedText {
    if bytes.starts_with(UTF8_BOM) {
        let content = String::from_utf8_lossy(&bytes[UTF8_BOM.len()..]).into_owned();
        return DecodedText {
            content,
            needs_utf8_cache: true,
        };
    }

    match String::from_utf8(bytes.to_vec()) {
        Ok(content) => DecodedText {
            content,
            needs_utf8_cache: false,
        },
        Err(_) => {
            let (content, _, _) = GBK.decode(bytes);
            DecodedText {
                content: content.into_owned(),
                needs_utf8_cache: true,
            }
        }
    }
}

// 把非普通 UTF-8 文本写成 UTF-8 缓存文件，供 offset 读取使用。
fn write_utf8_cache(content: &str) -> Result<PathBuf> {
    fs::create_dir_all(".reading")?;

    let path = PathBuf::from(".reading/current-text.utf8.txt");
    fs::write(&path, content)?;

    Ok(path)
}
