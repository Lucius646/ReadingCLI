use std::io;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, terminal,
};

pub(super) struct TerminalGuard;

impl TerminalGuard {
    /// 进入 raw mode、alternate screen，并隐藏光标。
    pub(super) fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            EnableMouseCapture
        )?;

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    /// 离开 TUI 时尽量恢复终端状态。
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            DisableMouseCapture,
            cursor::Show,
            terminal::LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
