use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

use anyhow::Result;

pub struct TextSource {
    path: PathBuf,
    block_size: usize,
    cache_radius: usize,
    block_offsets: Vec<u64>,
    cache: HashMap<usize, String>,
}

impl TextSource {
    pub fn new(path: PathBuf, block_size: usize, cache_radius: usize) -> Result<Self> {
        let content = fs::read_to_string(&path)?;

        let mut block_offsets = vec![0];

        for (char_count, (byte_index, _ch)) in content.char_indices().enumerate() {
            if char_count > 0 && char_count % block_size == 0 {
                block_offsets.push(byte_index as u64);
            }
        }

        let file_len = content.len() as u64;
        if block_offsets.last().copied() != Some(file_len) {
            block_offsets.push(file_len);
        }

        Ok(Self {
            path,
            block_size,
            cache_radius,
            block_offsets,
            cache: HashMap::new(),
        })
    }

    pub fn read_block(&mut self, block_index: usize) -> Result<String> {
        if let Some(block) = self.cache.get(&block_index) {
            return Ok(block.clone());
        }

        self.load_cache_around(block_index)?;

        Ok(self.cache.get(&block_index).cloned().unwrap_or_default())
    }

    fn block_count(&self) -> usize {
        self.block_offsets.len().saturating_sub(1)
    }

    fn load_cache_around(&mut self, block_index: usize) -> Result<()> {
        self.cache.clear();

        let block_count = self.block_count();
        if block_count == 0 || block_index >= block_count {
            return Ok(());
        }

        let start = block_index.saturating_sub(self.cache_radius);
        let end = block_index.saturating_add(self.cache_radius).min(block_count - 1);

        for index in start..=end {
            let block = self.read_block_from_file(index)?;
            self.cache.insert(index, block);
        }

        Ok(())
    }
    
    fn read_block_from_file(&self, block_index: usize) -> Result<String> {
        let start = self.block_offsets[block_index];
        let end = self.block_offsets[block_index + 1];

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(start))?;

        let mut buffer = vec![0; (end - start) as usize];
        file.read_exact(&mut buffer)?;

        let text = String::from_utf8(buffer)?;
        Ok(text)
    }

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

    pub fn read_before_offset(&self, offset: u64, max_bytes: usize) ->Result<(u64, String)> {
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
    
    pub fn file_len(&self) -> u64 {
        self.block_offsets.last().copied().unwrap_or(0)
    }
}
