use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, 
    terminal::{self, ClearType},
};

use crate::page_layout::{layout_page, Page};
use crate::page_index::PageIndex;
use crate::session::ReadingSession;
use crate::text_source::TextSource;

pub fn run_reader(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

    let (columns, rows) = terminal::size()?;
    let body_rows = rows.saturating_sub(2);
    let page_index = PageIndex::build(text_source, columns, body_rows)?;
    let mut current_page_index = page_index.find_page_by_offset(session.metadata.current_offset);

    if let Some(page_start) = page_index.page_start(current_page_index) {
        session.metadata.current_offset = page_start;
    }

    loop {
        draw(session, text_source)?;

        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }

            match key_event.code {
                KeyCode::Char('n') => {
                    if current_page_index + 1 < page_index.page_count() {
                        current_page_index += 1;

                        if let Some(page_start) = page_index.page_start(current_page_index) {
                            session.metadata.current_offset = page_start;
                        }
                    }
                }
                KeyCode::Char('p') => {
                    if current_page_index > 0 {
                        current_page_index -= 1;

                        if let Some(page_start) = page_index.page_start(current_page_index) {
                            session.metadata.current_offset = page_start;
                        }
                    }
                }
                KeyCode::Char('q') => {
                    session.quit();
                    break;
                }
                _ => {}
            }
        }

        if !session.running {
            break;
        }
    }
    execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn draw(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<Page> {
    let mut stdout = io::stdout();

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All)
    )?;

    let (columns, rows) = terminal::size()?;
    let body_rows = rows.saturating_sub(2); // Reserve 2 rows for title and status

    let candidate = text_source.read_from_offset(
        session.metadata.current_offset,
        64 * 1024,
    )?;

    let page = layout_page(
        &candidate,
        session.metadata.current_offset,
        columns,
        body_rows,
    );

    writeln!(stdout, "{}", page.text)?;
    execute!(
        stdout,
        cursor::MoveTo(0, rows.saturating_sub(1)),
        terminal::Clear(ClearType::CurrentLine)
    )?;

    let file_len = text_source.file_len();
    let progress = if file_len == 0 {
        0.0
    } else {
        session.metadata.current_offset as f64 / file_len as f64 * 100.0
    };

    write!(
        stdout,
        "[offset {}/{} {:.2}%] n: next | p: previous | q: quit",
        session.metadata.current_offset,
        file_len,
        progress
    )?;

    stdout.flush()?;

    Ok(page)
}
