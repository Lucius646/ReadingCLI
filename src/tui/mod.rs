use std::io;
use std::time::{Duration, Instant};
mod events;
mod highlight_cache;
mod screens;
mod state;
mod terminal;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::size,
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::cover::{CoverContext, CoverEngine};
use crate::library::BookLibrary;
use crate::session::ReadingSession;
use crate::text_source::TextSource;

use events::{
    handle_cover_key, handle_cover_mouse, handle_home_key, handle_open_input_key,
    handle_reading_key, handle_select_key,
};
use highlight_cache::load_or_build_annotation_cache;
use screens::draw;
use state::{AppMode, TuiState};
use terminal::TerminalGuard;

const HOME_ITEM_COUNT: usize = 5;
/// 进入全屏 TUI 主循环，处理绘制、按键、切书和退出。
pub fn run_reader(
    session: &mut ReadingSession,
    text_source: &mut TextSource,
    library: &mut BookLibrary,
) -> Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let (columns, rows) = size()?;
    let mut state = TuiState::new(text_source, session.metadata.current_offset, columns, rows)?;
    let mut cover_engine: Option<CoverEngine> = None;
    let mut annotation_cache = load_or_build_annotation_cache(
        &session.metadata.book_path,
        text_source,
        state.analyzer_kind,
    )?;

    if let Some(page_start) = state.page_index.page_start(state.current_page_index) {
        session.metadata.current_offset = page_start;
    }

    loop {
        let (latest_columns, latest_rows) = size()?;
        if let Some(page_start) = state.resize_if_needed(
            text_source,
            session.metadata.current_offset,
            latest_columns,
            latest_rows,
        )? {
            session.metadata.current_offset = page_start;
        }

        if state.app_mode == AppMode::Cover
            && let Some(engine) = cover_engine.as_mut()
        {
            let context = CoverContext {
                output_width: latest_columns as usize,
            };
            engine.tick(Instant::now(), &context);
        }

        draw(
            &mut terminal,
            session,
            text_source,
            library,
            &state,
            &mut annotation_cache,
            cover_engine.as_ref(),
        )?;

        let next_event = if state.app_mode == AppMode::Cover {
            if event::poll(Duration::from_millis(50))? {
                Some(event::read()?)
            } else {
                None
            }
        } else {
            Some(event::read()?)
        };

        if let Some(next_event) = next_event {
            match next_event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match state.app_mode {
                        AppMode::Home => {
                            handle_home_key(
                                key_event.code,
                                session,
                                text_source,
                                &mut annotation_cache,
                                &mut state,
                            )?;
                        }
                        AppMode::Reading => {
                            handle_reading_key(
                                key_event.code,
                                session,
                                &mut state,
                                &mut cover_engine,
                            );
                        }
                        AppMode::Cover => {
                            handle_cover_key(
                                key_event.code,
                                &mut state,
                                &mut cover_engine,
                                latest_columns as usize,
                                latest_rows as usize,
                            );
                        }
                        AppMode::OpenInput => {
                            handle_open_input_key(
                                key_event,
                                session,
                                text_source,
                                library,
                                &mut annotation_cache,
                                &mut state,
                            )?;
                        }
                        AppMode::Select => {
                            handle_select_key(
                                key_event.code,
                                session,
                                text_source,
                                library,
                                &mut annotation_cache,
                                &mut state,
                            )?;
                        }
                    }
                }
                Event::Mouse(mouse_event) if state.app_mode == AppMode::Cover => {
                    if let Some(engine) = cover_engine.as_mut() {
                        handle_cover_mouse(
                            mouse_event.kind,
                            engine,
                            latest_columns as usize,
                            latest_rows as usize,
                        );
                    }
                }
                _ => {}
            }
        }

        if !session.running {
            break;
        }
    }
    Ok(())
}
