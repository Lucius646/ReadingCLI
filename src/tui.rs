use std::io;

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
    widgets::Paragraph,
};

use crate::page_index::PageIndex;
use crate::page_layout::{Page, layout_page};
use crate::session::ReadingSession;
use crate::text_source::TextSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Home,
    Reading,
    Cover,
}

const HOME_ITEM_COUNT: usize = 4;

struct TuiState {
    app_mode: AppMode,
    columns: u16,
    body_rows: u16,
    page_index: PageIndex,
    current_page_index: usize,
    selected_home_item: usize,
}

impl TuiState {
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
        })
    }

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

        // 终端尺寸变化后，页索引必须按新尺寸重建。
        self.columns = columns;
        self.body_rows = body_rows;
        self.page_index = PageIndex::build(text_source, self.columns, self.body_rows)?;
        self.current_page_index = self.page_index.find_page_by_offset(current_offset);

        Ok(self.page_index.page_start(self.current_page_index))
    }

    fn next_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index + 1 < self.page_index.page_count() {
            self.current_page_index += 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }

    fn previous_page(&mut self, session: &mut ReadingSession) {
        if self.current_page_index > 0 {
            self.current_page_index -= 1;

            if let Some(page_start) = self.page_index.page_start(self.current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }
    }
}

pub fn run_reader(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
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

        draw(&mut terminal, session, text_source, &state)?;

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
            }
        }

        if !session.running {
            break;
        }
    }
    Ok(())
}

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
            1 => {}
            2 => {}
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

fn draw_home_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_home(frame, state))?;

    Ok(())
}

fn draw_reading_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    state: &TuiState,
) -> Result<()> {
    let (columns, rows) = terminal::size()?;
    let body_rows = rows.saturating_sub(2); // Reserve 2 rows for progress and shortcuts

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

fn draw_cover_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| draw_cover(frame))?;

    Ok(())
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    state: &TuiState,
) -> Result<()> {
    match state.app_mode {
        AppMode::Home => draw_home_screen(terminal, state),
        AppMode::Reading => draw_reading_screen(terminal, session, text_source, state),
        AppMode::Cover => draw_cover_screen(terminal),
    }
}

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

fn draw_cover(frame: &mut Frame) {
    let text = Paragraph::new(
        "Compiling reading_cli v0.1.0\nFinished dev profile\n\npress c to return | q to quit",
    );
    frame.render_widget(text, frame.area());
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
