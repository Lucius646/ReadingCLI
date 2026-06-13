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
    Reading,
    Cover,
}

pub fn run_reader(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let mut app_mode = AppMode::Reading;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let (mut columns, rows) = terminal::size()?;
    let mut body_rows = rows.saturating_sub(2);
    let mut page_index = PageIndex::build(text_source, columns, body_rows)?;
    let mut current_page_index = page_index.find_page_by_offset(session.metadata.current_offset);

    if let Some(page_start) = page_index.page_start(current_page_index) {
        session.metadata.current_offset = page_start;
    }

    loop {
        let (latest_columns, latest_rows) = terminal::size()?;
        let latest_body_rows = latest_rows.saturating_sub(2);

        if latest_columns != columns || latest_body_rows != body_rows {
            columns = latest_columns;
            body_rows = latest_body_rows;

            page_index = PageIndex::build(text_source, columns, body_rows)?;
            current_page_index = page_index.find_page_by_offset(session.metadata.current_offset);

            if let Some(page_start) = page_index.page_start(current_page_index) {
                session.metadata.current_offset = page_start;
            }
        }

        draw(
            &mut terminal,
            app_mode,
            session,
            text_source,
            current_page_index,
            page_index.page_count(),
        )?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            match app_mode {
                AppMode::Reading => {
                    handle_reading_key(
                        key_event.code,
                        session,
                        &page_index,
                        &mut current_page_index,
                        &mut app_mode,
                    );
                }
                AppMode::Cover => {
                    handle_cover_key(key_event.code, &mut app_mode, session);
                }
            }
        }

        if !session.running {
            break;
        }
    }
    Ok(())
}

fn handle_reading_key(
    key_code: KeyCode,
    session: &mut ReadingSession,
    page_index: &PageIndex,
    current_page_index: &mut usize,
    app_mode: &mut AppMode,
) {
    match key_code {
        KeyCode::Char('n') => {
            if *current_page_index + 1 < page_index.page_count() {
                *current_page_index += 1;

                if let Some(page_start) = page_index.page_start(*current_page_index) {
                    session.metadata.current_offset = page_start;
                }
            }
        }
        KeyCode::Char('p') => {
            if *current_page_index > 0 {
                *current_page_index -= 1;

                if let Some(page_start) = page_index.page_start(*current_page_index) {
                    session.metadata.current_offset = page_start;
                }
            }
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        KeyCode::Char('c') => {
            *app_mode = AppMode::Cover;
        }
        _ => {}
    }
}

fn handle_cover_key(key_code: KeyCode, app_mode: &mut AppMode, session: &mut ReadingSession) {
    match key_code {
        KeyCode::Char('c') => {
            *app_mode = AppMode::Reading;
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        _ => {}
    }
}

fn draw_reading_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    current_page_index: usize,
    page_count: usize,
) -> Result<()> {
    let (columns, rows) = terminal::size()?;
    let body_rows = rows.saturating_sub(2); // Reserve 2 rows for title and status

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

    let status_line = format!(
        "[page {}/{} offset {}/{} {:.2}%] n: next | p: previous | q: quit | c: cover",
        current_page_index + 1,
        page_count,
        session.metadata.current_offset,
        file_len,
        progress
    );

    terminal.draw(|frame| draw_reading(frame, &page, &status_line))?;

    Ok(())
}

fn draw_cover_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| draw_cover(frame))?;

    Ok(())
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app_mode: AppMode,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    current_page_index: usize,
    page_count: usize,
) -> Result<()> {
    match app_mode {
        AppMode::Reading => draw_reading_screen(
            terminal,
            session,
            text_source,
            current_page_index,
            page_count,
        ),
        AppMode::Cover => draw_cover_screen(terminal),
    }
}

fn draw_reading(frame: &mut Frame, page: &Page, status_line: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let body = Paragraph::new(page.text.as_str());
    let status = Paragraph::new(status_line);

    frame.render_widget(body, chunks[0]);
    frame.render_widget(status, chunks[1]);
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
