use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEventKind};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::cover::{CoverEngine, default_registry};
use crate::highlight::store::AnnotationCache;
use crate::library::{BookLibrary, current_timestamp};
use crate::page_index::PageIndex;
use crate::session::ReadingSession;
use crate::text_source::TextSource;

use super::state::{AppMode, TuiState};
use super::{HOME_ITEM_COUNT, load_or_build_annotation_cache};

pub(super) fn handle_home_key(
    key_code: KeyCode,
    session: &mut ReadingSession,
    text_source: &TextSource,
    annotation_cache: &mut AnnotationCache,
    state: &mut TuiState,
) -> Result<()> {
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
                match load_or_build_annotation_cache(
                    &session.metadata.book_path,
                    text_source,
                    state.analyzer_kind,
                ) {
                    Ok(cache) => {
                        *annotation_cache = cache;
                        state.home_error = None;
                        state.app_mode = AppMode::Reading;
                    }
                    Err(error) => {
                        state.home_error = Some(error.to_string());
                    }
                }
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
                state.cycle_analyzer();
            }
            4 => {
                session.quit();
            }
            _ => {}
        },
        KeyCode::Char('q') => {
            session.quit();
        }
        KeyCode::Left | KeyCode::Right => {
            if state.selected_home_item == 3 {
                state.cycle_analyzer();
            }
        }
        _ => {}
    }

    Ok(())
}

pub(super) fn handle_select_key(
    key_code: KeyCode,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    annotation_cache: &mut AnnotationCache,
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
                switch_book(
                    session,
                    text_source,
                    library,
                    annotation_cache,
                    state,
                    book.book_path.clone(),
                )?;
            }
        }
        KeyCode::Char('q') => {
            session.quit();
        }
        _ => {}
    }

    Ok(())
}

pub(super) fn handle_open_input_key(
    key_event: KeyEvent,
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    annotation_cache: &mut AnnotationCache,
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

            switch_book(session, text_source, library, annotation_cache, state, path)?;
        }
        _ => {
            state.open_input.handle_event(&Event::Key(key_event));
            state.open_error = None;
        }
    }

    Ok(())
}

pub(super) fn handle_reading_key(
    key_code: KeyCode,
    session: &mut ReadingSession,
    state: &mut TuiState,
    cover_engine: &mut Option<CoverEngine>,
) {
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
            *cover_engine = Some(CoverEngine::new(default_registry(), 10_000, Instant::now()));
            state.app_mode = AppMode::Cover;
        }
        _ => {}
    }
}

pub(super) fn handle_cover_key(
    key_code: KeyCode,
    state: &mut TuiState,
    cover_engine: &mut Option<CoverEngine>,
    body_width: usize,
    body_height: usize,
) {
    if key_code == KeyCode::Char('c') {
        *cover_engine = None;
        state.app_mode = AppMode::Reading;
        return;
    }

    let Some(engine) = cover_engine.as_mut() else {
        state.app_mode = AppMode::Reading;
        return;
    };

    match key_code {
        KeyCode::Up => {
            engine.terminal_mut().scroll_up(1, body_width, body_height);
        }
        KeyCode::Down => {
            engine
                .terminal_mut()
                .scroll_down(1, body_width, body_height);
        }
        KeyCode::PageUp => {
            engine
                .terminal_mut()
                .scroll_up(body_height, body_width, body_height);
        }
        KeyCode::PageDown => {
            engine
                .terminal_mut()
                .scroll_down(body_height, body_width, body_height);
        }
        KeyCode::End => {
            engine.terminal_mut().scroll_to_bottom();
        }
        _ => {}
    }
}

pub(super) fn handle_cover_mouse(
    mouse_kind: MouseEventKind,
    engine: &mut CoverEngine,
    body_width: usize,
    body_height: usize,
) {
    match mouse_kind {
        MouseEventKind::ScrollUp => {
            engine.terminal_mut().scroll_up(3, body_width, body_height);
        }
        MouseEventKind::ScrollDown => {
            engine
                .terminal_mut()
                .scroll_down(3, body_width, body_height);
        }
        _ => {}
    }
}

fn switch_book(
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
    annotation_cache: &mut AnnotationCache,
    state: &mut TuiState,
    path: PathBuf,
) -> Result<()> {
    library.upsert_book(session.metadata.clone());

    let metadata = library.activate_book(path, current_timestamp());
    session.metadata = metadata;
    *text_source = TextSource::new(session.metadata.book_path.clone())?;
    session.metadata.file_len = text_source.file_len();
    *annotation_cache = load_or_build_annotation_cache(
        &session.metadata.book_path,
        text_source,
        state.analyzer_kind,
    )?;

    state.page_index = PageIndex::build(text_source, state.columns, state.body_rows)?;
    state.current_page_index = state
        .page_index
        .find_page_by_offset(session.metadata.current_offset);
    state.open_input = Input::default();
    state.open_error = None;
    state.app_mode = AppMode::Reading;

    Ok(())
}
