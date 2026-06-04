use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, 
    terminal::{self, ClearType},
};

use crate::session::ReadingSession;
use crate::text_source::TextSource;

pub fn run_reader(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

    loop {
        draw(session, text_source)?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('n') => session.next_block(),
                KeyCode::Char('p') => session.previous_block(),
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

fn draw(session: &mut ReadingSession, text_source: &mut TextSource) -> Result<()> {
    let mut stdout = io::stdout();

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All)
    )?;

    let text = text_source.read_block(session.metadata.current_block)?;

    writeln!(stdout, "{text}")?;
    writeln!(stdout)?;
    writeln!(
        stdout,
        "[block {}] n: next | p: previous | q: quit",
        session.metadata.current_block
    )?;

    stdout.flush()?;

    Ok(())
}

