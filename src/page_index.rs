use anyhow::Result;

use crate::page_layout::layout_page;
use crate::text_source::TextSource;

pub struct PageIndex {
    pub columns: u16,
    pub rows: u16,
    pub page_starts: Vec<u64>,
}

impl PageIndex {
    // 按当前终端尺寸从头扫描文本，建立每一页的起始 offset。
    pub fn build(text_source: &TextSource, columns: u16, rows: u16) -> Result<Self> {
        let mut page_starts = Vec::new();
        let mut current_offset = 0;

        let file_len = text_source.file_len();
        let read_size = 64 * 1024;

        while current_offset < file_len {
            page_starts.push(current_offset);

            let candidate = text_source.read_from_offset(current_offset, read_size)?;
            let page = layout_page(&candidate, current_offset, columns, rows);

            if page.end_offset <= current_offset {
                break;
            }
            current_offset = page.end_offset;
        }

        Ok(Self {
            columns,
            rows,
            page_starts,
        })
    }

    // 返回当前页索引里的总页数。
    pub fn page_count(&self) -> usize {
        self.page_starts.len()
    }

    // 根据持久化 offset 找到它在当前终端尺寸下属于哪一页。
    pub fn find_page_by_offset(&self, offset: u64) -> usize {
        let mut page_index = 0;
        for (index, page_start) in self.page_starts.iter().enumerate() {
            if *page_start <= offset {
                page_index = index;
            } else {
                break;
            }
        }
        page_index
    }

    // 返回指定页的起始 offset；页号越界时返回 None。
    pub fn page_start(&self, page_index: usize) -> Option<u64> {
        self.page_starts.get(page_index).copied()
    }
}
