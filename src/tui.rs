use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, 
    terminal::{self, ClearType},
};

use crate::page_layout::{layout_page, Page};

use crate::session::ReadingSession;
use crate::text_source::TextSource;

pub fn run_reader(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

    loop {
        let current_page = draw(session, text_source)?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('n') => {
                    session.move_to_offset(current_page.end_offset);
                }
                KeyCode::Char('p') => {
                    let (columns, rows) = terminal::size()?;
                    let body_rows = rows.saturating_sub(2);

                    let previous_offset = find_previous_page_start(
                        text_source, 
                        session.metadata.current_offset, 
                        columns, 
                        body_rows
                    )?;
                    session.metadata.current_offset = previous_offset;
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
    writeln!(
        stdout,
        "[offset {}] n: next | p: previous | q: quit",
        session.metadata.current_offset
    )?;

    stdout.flush()?;

    Ok(page)
}

fn find_previous_page_start(text_source: &TextSource, current_offset: u64, columns: u16, rows: u16) -> Result<u64> {
    if current_offset == 0 {
        return Ok(0);
    }

    let window_bytes = estimate_page_window_bytes(columns, rows);

    let (candidate_start, candidate) = 
        text_source.read_before_offset(current_offset, window_bytes)?;

    if candidate.is_empty() {
        return Ok(0);
    }

    let valid_offsets: Vec<u64> = candidate
        .char_indices()
        .map(|(byte_index, _ch)| candidate_start + byte_index as u64)
        .filter(|offset| * offset < current_offset)
        .collect();

    if valid_offsets.is_empty() {
        return Ok(0);
    }
    
    let mut low = 0usize;
    let mut high = valid_offsets.len();
    while low < high {
        let mid = low + (high - low) / 2;
        let start_offset = valid_offsets[mid];

        let candidate = text_source.read_from_offset(start_offset, window_bytes)?;
        let page = layout_page(&candidate, start_offset, columns, rows);

        if page.end_offset >= current_offset {
            let overlap = page.end_offset - current_offset;
            
            if overlap <= 4 {
                return Ok(start_offset);
            }
            // 一个 utf8 字符最多 4 字节
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if low < valid_offsets.len() {
        Ok(valid_offsets[low])
    } else {
        Ok(valid_offsets[valid_offsets.len() - 1])
    }
}

fn estimate_page_window_bytes(columns: u16, rows: u16) -> usize {
    let cells = columns as usize * rows as usize;
    cells.saturating_mul(4)
}
