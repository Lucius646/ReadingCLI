use anyhow::Result;
use tui_input::Input;

use crate::highlight::analyzer::AnalyzerKind;
use crate::page_index::PageIndex;
use crate::session::ReadingSession;
use crate::text_source::TextSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AppMode {
    Home,
    Reading,
    Cover,
    OpenInput,
    Select,
}

pub(super) struct TuiState {
    pub(super) app_mode: AppMode,
    pub(super) columns: u16,
    pub(super) body_rows: u16,
    pub(super) page_index: PageIndex,
    pub(super) current_page_index: usize,
    pub(super) selected_home_item: usize,
    pub(super) open_input: Input,
    pub(super) open_error: Option<String>,
    pub(super) home_error: Option<String>,
    pub(super) selected_book_index: usize,
    pub(super) analyzer_kind: AnalyzerKind,
}

impl TuiState {
    /// 创建 TUI 运行状态，并按当前终端尺寸建立页索引。
    pub(super) fn new(
        text_source: &TextSource,
        current_offset: u64,
        columns: u16,
        rows: u16,
    ) -> Result<Self> {
        let body_rows = rows.saturating_sub(2);
        let page_index = PageIndex::build(text_source, columns, body_rows)?;
        let current_page_index = page_index.find_page_by_offset(current_offset);

        Ok(Self {
            app_mode: AppMode::Home,
            columns,
            body_rows,
            page_index,
            current_page_index,
            selected_home_item: 0,
            open_input: Input::default(),
            open_error: None,
            home_error: None,
            selected_book_index: 0,
            analyzer_kind: AnalyzerKind::Jieba,
        })
    }

    /// 终端尺寸变化时重建页索引，并返回对齐后的页起点。
    pub(super) fn resize_if_needed(
        &mut self,
        text_source: &TextSource,
        current_offset: u64,
        columns: u16,
        rows: u16,
    ) -> Result<Option<u64>> {
        let body_rows = rows.saturating_sub(2);

        if columns == self.columns && body_rows == self.body_rows {
            return Ok(None);
        }

        self.columns = columns;
        self.body_rows = body_rows;
        self.page_index = PageIndex::build(text_source, self.columns, self.body_rows)?;
        self.current_page_index = self.page_index.find_page_by_offset(current_offset);

        Ok(self.page_index.page_start(self.current_page_index))
    }

    /// 翻到下一页，并同步更新 session 中可持久化的 offset。
    pub(super) fn next_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index + 1 < self.page_index.page_count() {
            self.current_page_index += 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }

    /// 翻到上一页，并同步更新 session 中可持久化的 offset。
    pub(super) fn previous_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index > 0 {
            self.current_page_index -= 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }

    pub(super) fn cycle_analyzer(&mut self) {
        self.analyzer_kind = self.analyzer_kind.next();
        self.home_error = None;
    }
}
