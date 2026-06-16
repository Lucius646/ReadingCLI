use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, terminal,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use time::{Month, OffsetDateTime};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::library::{BookLibrary, current_timestamp};
use crate::page_index::PageIndex;
use crate::page_layout::{Page, layout_page};
use crate::session::ReadingSession;
use crate::text_source::TextSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Home,
    Reading,
    Cover,
    OpenInput,
    Select,
}

const HOME_ITEM_COUNT: usize = 4;

struct TuiState {
    app_mode: AppMode,
    columns: u16,
    body_rows: u16,
    page_index: PageIndex,
    current_page_index: usize,
    selected_home_item: usize,
    open_input: Input,
    open_error: Option<String>,
    selected_book_index: usize,
}

impl TuiState {
    // 创建 TUI 运行状态，并按当前终端尺寸建立页索引。
    fn new(text_source: &TextSource, current_offset: u64, columns: u16, rows: u16) -> Result<Self> {
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
            selected_book_index: 0,
        })
    }

    // 终端尺寸变化时重建页索引，并返回对齐后的页起点。
    fn resize_if_needed(
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

    // 翻到下一页，并同步更新 session 的持久化 offset。
    fn next_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index + 1 < self.page_index.page_count() {
            self.current_page_index += 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }

    // 翻到上一页，并同步更新 session 的持久化 offset。
    fn previous_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index > 0 {
            self.current_page_index -= 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }
}

// 进入全屏 TUI 主循环，处理绘制、按键、切书和退出。
pub fn run_reader(
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
) -> Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let (columns, rows) = terminal::size()?;
    let mut state = TuiState::new(text_source, session.metadata.current_offset, columns, rows)?;

    if let Some(page_start) = state.page_index.page_start(state.current_page_index) {
        session.metadata.current_offset = page_start;
    }

    loop {
        let (latest_columns, latest_rows) = terminal::size()?;
        if let Some(page_start) = state.resize_if_needed(
            text_source,
            session.metadata.current_offset,
            latest_columns,
            latest_rows,
        )? {
            session.metadata.current_offset = page_start;
        }

        draw(&mut terminal, session, text_source, library, &state)?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            match state.app_mode {
                AppMode::Home => {
                    handle_home_key(key_event.code, session, &mut state);
                }
                AppMode::Reading => {
                    handle_reading_key(key_event.code, session, &mut state);
                }
                AppMode::Cover => {
                    handle_cover_key(key_event.code, session, &mut state);
                }
                AppMode::OpenInput => {
                    handle_open_input_key(key_event, session, text_source, library, &mut state)?;
                }
                AppMode::Select => {
                    handle_select_key(key_event.code, session, text_source, library, &mut state)?;
                }
            }
        }

        if !session.running {
            break;
        }
    }
    Ok(())
}

// 处理首页菜单的方向键、确认键和退出键。
fn handle_home_key(key_code: KeyCode, session: &mut ReadingSession, state: &mut TuiState) {
    match key_code {
        KeyCode::Up => {
            state.selected_home_item =
                (state.selected_home_item + HOME_ITEM_COUNT - 1) % HOME_ITEM_COUNT;
        }
        KeyCode::Down => {
            state.selected_home_item = (state.selected_home_item + 1) % HOME_ITEM_COUNT;
        }
        KeyCode::Enter => match state.selected_home_item {
            0 => {
                state.app_mode = AppMode::Reading;
            }
            1 => {
                state.open_input = Input::default();
                state.open_error = None;
                state.app_mode = AppMode::OpenInput;
            }
            2 => {
                state.selected_book_index = 0;
                state.app_mode = AppMode::Select;
            }
            3 => {
                session.quit();
            }
            _ => {}
        },
        KeyCode::Char('q') => {
            session.quit();
        }
        _ => {}
    }
}

// 处理书架选择页的移动、打开和返回。
fn handle_select_key(
    key_code: KeyCode,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    state: &mut TuiState,
) -> Result<()> {
    let default_book_path = PathBuf::from("default.txt");
    let books = library.visible_books(&default_book_path);

    match key_code {
        KeyCode::Esc => {
            state.app_mode = AppMode::Home;
        }
        KeyCode::Up => {
            if !books.is_empty() {
                state.selected_book_index =
                    (state.selected_book_index + books.len() - 1) % books.len();
            }
        }
        KeyCode::Down => {
            if !books.is_empty() {
                state.selected_book_index = (state.selected_book_index + 1) % books.len();
            }
        }
        KeyCode::Enter => {
            if let Some(book) = books.get(state.selected_book_index) {
                switch_book(session, text_source, library, state, book.book_path.clone())?;
            }
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        _ => {}
    }

    Ok(())
}

// 处理新书路径输入页的文本编辑、路径校验和打开动作。
fn handle_open_input_key(
    key_event: event::KeyEvent,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    state: &mut TuiState,
) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            state.app_mode = AppMode::Home;
            state.open_error = None;
        }
        KeyCode::Enter => {
            let input = state.open_input.value().trim().trim_matches('"');

            if input.is_empty() {
                state.open_error = Some("path is empty".to_string());
                return Ok(());
            }

            let path = PathBuf::from(input);
            let Ok(path) = fs::canonicalize(path) else {
                state.open_error = Some("file not found".to_string());
                return Ok(());
            };

            let is_txt = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"));

            if !is_txt {
                state.open_error = Some("only .txt is supported".to_string());
                return Ok(());
            }

            switch_book(session, text_source, library, state, path)?;
        }
        _ => {
            state.open_input.handle_event(&Event::Key(key_event));
            state.open_error = None;
        }
    }

    Ok(())
}

// 处理阅读页的翻页、回首页、Cover Mode 和退出。
fn handle_reading_key(key_code: KeyCode, session: &mut ReadingSession, state: &mut TuiState) {
    match key_code {
        KeyCode::Char('n') => {
            state.next_page(session);
        }
        KeyCode::Char('p') => {
            state.previous_page(session);
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        KeyCode::Esc | KeyCode::Char('h') => {
            state.app_mode = AppMode::Home;
        }
        KeyCode::Char('c') => {
            state.app_mode = AppMode::Cover;
        }
        _ => {}
    }
}

// 处理 Cover Mode 的返回和退出。
fn handle_cover_key(key_code: KeyCode, session: &mut ReadingSession, state: &mut TuiState) {
    match key_code {
        KeyCode::Char('c') => {
            state.app_mode = AppMode::Reading;
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        _ => {}
    }
}

// 切换当前书：保存旧进度，激活新书，重建 TextSource 和 PageIndex。
fn switch_book(
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    state: &mut TuiState,
    path: PathBuf,
) -> Result<()> {
    library.upsert_book(session.metadata.clone());

    let metadata = library.activate_book(path, current_timestamp());
    session.metadata = metadata;
    *text_source = TextSource::new(session.metadata.book_path.clone())?;
    session.metadata.file_len = text_source.file_len();

    state.page_index = PageIndex::build(text_source, state.columns, state.body_rows)?;
    state.current_page_index = state
        .page_index
        .find_page_by_offset(session.metadata.current_offset);
    state.open_input = Input::default();
    state.open_error = None;
    state.app_mode = AppMode::Reading;

    Ok(())
}

// 绘制首页模式。
fn draw_home_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_home(frame, state))?;

    Ok(())
}

// 准备阅读页数据并绘制正文、进度和快捷键。
fn draw_reading_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    state: &TuiState,
) -> Result<()> {
    let (columns, rows) = terminal::size()?;
    let body_rows = rows.saturating_sub(2);

    let candidate = text_source.read_from_offset(session.metadata.current_offset, 64 * 1024)?;

    let page = layout_page(
        &candidate,
        session.metadata.current_offset,
        columns,
        body_rows,
    );

    let file_len = text_source.file_len();
    let progress = if file_len == 0 {
        0.0
    } else {
        session.metadata.current_offset as f64 / file_len as f64 * 100.0
    };

    let progress_line = format!(
        "page {}/{} | offset {}/{} | {:.2}%",
        state.current_page_index + 1,
        state.page_index.page_count(),
        session.metadata.current_offset,
        file_len,
        progress
    );

    let shortcut_line = "n: next | p: previous | h/Esc: home | c: cover | q: quit";

    terminal.draw(|frame| draw_reading(frame, &page, &progress_line, shortcut_line))?;

    Ok(())
}

// 绘制 Cover Mode 页面。
fn draw_cover_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| draw_cover(frame))?;

    Ok(())
}

// 绘制新书路径输入页面。
fn draw_open_input_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_open_input(frame, state))?;

    Ok(())
}

// 绘制书架选择页面。
fn draw_select_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    library: &BookLibrary,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_select(frame, library, state))?;

    Ok(())
}

// 根据当前 AppMode 分发到对应页面绘制函数。
fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &BookLibrary,
    state: &TuiState,
) -> Result<()> {
    match state.app_mode {
        AppMode::Home => draw_home_screen(terminal, state),
        AppMode::Reading => draw_reading_screen(terminal, session, text_source, state),
        AppMode::Cover => draw_cover_screen(terminal),
        AppMode::OpenInput => draw_open_input_screen(terminal, state),
        AppMode::Select => draw_select_screen(terminal, library, state),
    }
}

// 在 ratatui frame 上渲染首页菜单。
fn draw_home(frame: &mut Frame, state: &TuiState) {
    let items = ["Continue", "Open New Book", "Select", "Quit"];
    let mut text = String::from("ReadingCLI\n\n");

    for (index, item) in items.iter().enumerate() {
        if index == state.selected_home_item {
            text.push_str("> ");
        } else {
            text.push_str("  ");
        }
        text.push_str(item);
        text.push('\n');
    }

    text.push_str("\nUp/Down: move | Enter: select | q: quit");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
}

// 在 ratatui frame 上渲染路径输入框和错误信息。
fn draw_open_input(frame: &mut Frame, state: &TuiState) {
    let mut lines = vec![
        Line::from("Open New Book"),
        Line::from(""),
        Line::from("Input txt path:"),
        render_input_line(&state.open_input),
    ];

    if let Some(error) = &state.open_error {
        lines.push(Line::from(""));
        lines.push(Line::from(error.as_str()));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Enter: open | Esc: home | Backspace: delete"));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, frame.area());
}

// 在 ratatui frame 上渲染可选择书目列表。
fn draw_select(frame: &mut Frame, library: &BookLibrary, state: &TuiState) {
    let default_book_path = PathBuf::from("default.txt");
    let books = library.visible_books(&default_book_path);
    let mut text = String::from("Select Book\n\n");

    if books.is_empty() {
        text.push_str("No books in library.\n\nEsc: home | q: quit");
    } else {
        for (index, book) in books.iter().enumerate() {
            if index == state.selected_book_index {
                text.push_str("> ");
            } else {
                text.push_str("  ");
            }
            text.push_str(&book_title(&book.book_path));
            let progress = progress_percent(book.current_offset, book.file_len);
            text.push_str(&format!(
                "\n    {:.2}% | offset {}/{} | last opened {}",
                progress,
                book.current_offset,
                book.file_len,
                format_timestamp(book.last_opened_at)
            ));
            text.push('\n');
        }

        text.push_str("\nUp/Down: move | Enter: open | Esc: home | q: quit");
    }

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
}

// 把 tui-input 的文本和光标位置渲染成高亮 Span。
fn render_input_line(input: &Input) -> Line<'static> {
    let value = input.value();
    let cursor = input.cursor();
    let chars = value.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();

    for (index, ch) in chars.iter().enumerate() {
        if index == cursor {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::raw(ch.to_string()));
        }
    }

    if cursor >= chars.len() {
        spans.push(Span::styled(
            " ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(spans)
}

// 从路径中提取展示用书名。
fn book_title(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

// 根据 offset 和文件长度计算阅读百分比。
fn progress_percent(current_offset: u64, file_len: u64) -> f64 {
    if file_len == 0 {
        0.0
    } else {
        current_offset as f64 / file_len as f64 * 100.0
    }
}

// 把 Unix 秒时间戳格式化成 Select 页面展示用字符串。
fn format_timestamp(timestamp: u64) -> String {
    if timestamp == 0 {
        return "never".to_string();
    }

    let Ok(datetime) = OffsetDateTime::from_unix_timestamp(timestamp as i64) else {
        return "invalid".to_string();
    };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} UTC",
        datetime.year(),
        month_number(datetime.month()),
        datetime.day(),
        datetime.hour(),
        datetime.minute()
    )
}

// 把 time::Month 转成数字月份。
fn month_number(month: Month) -> u8 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}

// 在 ratatui frame 上渲染阅读正文和底部两行状态。
fn draw_reading(frame: &mut Frame, page: &Page, progress_line: &str, shortcut_line: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let body = Paragraph::new(page.text.as_str());
    let progress = Paragraph::new(progress_line);
    let shortcuts = Paragraph::new(shortcut_line);

    frame.render_widget(body, chunks[0]);
    frame.render_widget(progress, chunks[1]);
    frame.render_widget(shortcuts, chunks[2]);
}

// 在 ratatui frame 上渲染伪装输出。
fn draw_cover(frame: &mut Frame) {
    let text = Paragraph::new(
        "Compiling reading_cli v0.1.0\nFinished dev profile\n\npress c to return | q to quit",
    );
    frame.render_widget(text, frame.area());
}

struct TerminalGuard;

impl TerminalGuard {
    // 进入 raw mode、alternate screen，并隐藏光标。
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    // 离开 TUI 时尽量恢复终端状态。
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
