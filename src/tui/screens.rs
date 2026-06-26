use std::io;
use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::terminal::size;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use time::{Month, OffsetDateTime};
use tui_input::Input;

use crate::cover::CoverEngine;
use crate::highlight::render::render_highlighted_page;
use crate::highlight::store::AnnotationCache;
use crate::library::BookLibrary;
use crate::page_layout::layout_page;
use crate::session::ReadingSession;
use crate::text_source::TextSource;

use super::state::{AppMode, TuiState};

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
    annotation_cache: &mut AnnotationCache,
) -> Result<()> {
    let (columns, rows) = size()?;
    let body_rows = rows.saturating_sub(2);

    let candidate = text_source.read_from_offset(session.metadata.current_offset, 64 * 1024)?;

    let page = layout_page(
        &candidate,
        session.metadata.current_offset,
        columns,
        body_rows,
    );
    let annotations = annotation_cache.query(session.metadata.current_offset, page.end_offset)?;
    let highlighted_page = render_highlighted_page(
        &candidate,
        session.metadata.current_offset,
        columns,
        body_rows,
        &annotations,
    );

    let file_len = text_source.file_len();
    let progress = if file_len == 0 {
        0.0
    } else {
        session.metadata.current_offset as f64 / file_len as f64 * 100.0
    };

    let progress_line = format!(
        "page {}/{} | offset {}/{} | {:.2}% | analyzer: {}",
        state.current_page_index + 1,
        state.page_index.page_count(),
        session.metadata.current_offset,
        file_len,
        progress,
        state.analyzer_kind.analyzer_id()
    );

    let shortcut_line = "n: next | p: previous | h/Esc: home | c: cover | q: quit";

    terminal
        .draw(|frame| draw_reading(frame, highlighted_page.lines, &progress_line, shortcut_line))?;

    Ok(())
}

fn draw_cover_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    engine: &CoverEngine,
) -> Result<()> {
    terminal.draw(|frame| draw_cover(frame, engine))?;

    Ok(())
}

fn draw_open_input_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_open_input(frame, state))?;

    Ok(())
}

fn draw_select_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    library: &BookLibrary,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| draw_select(frame, library, state))?;

    Ok(())
}

pub(super) fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &BookLibrary,
    state: &TuiState,
    annotation_cache: &mut AnnotationCache,
    cover_engine: Option<&CoverEngine>,
) -> Result<()> {
    match state.app_mode {
        AppMode::Home => draw_home_screen(terminal, state),
        AppMode::Reading => {
            draw_reading_screen(terminal, session, text_source, state, annotation_cache)
        }
        AppMode::Cover => draw_cover_screen(
            terminal,
            cover_engine.expect("Cover mode requires an active CoverEngine"),
        ),
        AppMode::OpenInput => draw_open_input_screen(terminal, state),
        AppMode::Select => draw_select_screen(terminal, library, state),
    }
}

fn draw_home(frame: &mut Frame, state: &TuiState) {
    let analyzer_item = format!("Analyzer: {}", state.analyzer_kind.analyzer_id());
    let items = [
        "Continue".to_string(),
        "Open New Book".to_string(),
        "Select".to_string(),
        analyzer_item,
        "Quit".to_string(),
    ];
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

    if let Some(error) = &state.home_error {
        text.push_str("\n");
        text.push_str(error);
        text.push('\n');
    }

    text.push_str("\nUp/Down: move | Left/Right: switch analyzer | Enter: select | q: quit");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
}

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

fn book_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn progress_percent(current_offset: u64, file_len: u64) -> f64 {
    if file_len == 0 {
        0.0
    } else {
        current_offset as f64 / file_len as f64 * 100.0
    }
}

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

fn draw_reading(
    frame: &mut Frame,
    body_lines: Vec<Line<'static>>,
    progress_line: &str,
    shortcut_line: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let body = Paragraph::new(body_lines);
    let progress = Paragraph::new(progress_line);
    let shortcuts = Paragraph::new(shortcut_line);

    frame.render_widget(body, chunks[0]);
    frame.render_widget(progress, chunks[1]);
    frame.render_widget(shortcuts, chunks[2]);
}

fn draw_cover(frame: &mut Frame, engine: &CoverEngine) {
    let area = frame.area();
    let lines = engine
        .terminal()
        .visible_rows(area.width as usize, area.height as usize)
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    let output = Paragraph::new(lines);

    frame.render_widget(output, area);
}
