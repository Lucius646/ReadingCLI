use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

use super::CoverOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ViewPosition {
    line: u64,
    wrapped_row: usize,
}

struct VisualRow {
    position: ViewPosition,
    text: String,
}

pub struct CoverTerminal {
    lines: VecDeque<String>,
    first_line: u64,
    cursor_line: u64,
    max_lines: usize,
    view_start: Option<ViewPosition>,
}

impl CoverTerminal {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::from([String::new()]),
            first_line: 0,
            cursor_line: 0,
            max_lines: max_lines.max(1),
            view_start: None,
        }
    }

    pub fn apply(&mut self, op: CoverOp) {
        match op {
            CoverOp::Write(text) => self.write(&text),
            CoverOp::NewLine => self.new_line(),
            CoverOp::EraseLine => self.erase_line(),
            CoverOp::CursorUp(rows) => self.cursor_up(rows),
        }
    }

    pub fn visible_rows(&self, width: usize, height: usize) -> Vec<String> {
        let rows = self.layout_rows(width);
        let start_index = self.view_start_index(&rows, height);

        rows.into_iter()
            .skip(start_index)
            .take(height)
            .map(|row| row.text)
            .collect()
    }

    pub fn scroll_up(&mut self, amount: usize, width: usize, height: usize) {
        if width == 0 || height == 0 {
            return;
        }

        let rows = self.layout_rows(width);
        let current_start = self.view_start_index(&rows, height);
        let next_start = current_start.saturating_sub(amount);

        if next_start < current_start {
            self.view_start = rows.get(next_start).map(|row| row.position);
        }
    }

    pub fn scroll_down(&mut self, amount: usize, width: usize, height: usize) {
        if width == 0 || height == 0 {
            return;
        }

        let rows = self.layout_rows(width);
        let bottom_start = rows.len().saturating_sub(height);
        let current_start = self.view_start_index(&rows, height);
        let next_start = current_start.saturating_add(amount).min(bottom_start);

        self.view_start = if next_start == bottom_start {
            None
        } else {
            rows.get(next_start).map(|row| row.position)
        };
    }

    pub fn scroll_to_bottom(&mut self) {
        self.view_start = None;
    }

    fn write(&mut self, text: &str) {
        let index = self.line_index(self.cursor_line);
        self.lines[index].push_str(text);
    }

    fn new_line(&mut self) {
        if self.cursor_line < self.last_line() {
            self.cursor_line += 1;
            return;
        }

        self.cursor_line += 1;
        self.lines.push_back(String::new());
        self.trim_history();
    }

    fn erase_line(&mut self) {
        let index = self.line_index(self.cursor_line);
        self.lines[index].clear();
    }

    fn cursor_up(&mut self, rows: usize) {
        self.cursor_line = self
            .cursor_line
            .saturating_sub(rows as u64)
            .max(self.first_line);
    }

    fn trim_history(&mut self) {
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
            self.first_line += 1;
        }

        if let Some(view_start) = self.view_start
            && view_start.line < self.first_line
        {
            self.view_start = Some(ViewPosition {
                line: self.first_line,
                wrapped_row: 0,
            });
        }
    }

    fn line_index(&self, absolute_line: u64) -> usize {
        let relative_line = absolute_line
            .checked_sub(self.first_line)
            .expect("absolute line is older than retained history");
        let index = usize::try_from(relative_line).expect("absolute line does not fit into usize");

        assert!(
            index < self.lines.len(),
            "absolute line is outside retained history"
        );

        index
    }

    fn last_line(&self) -> u64 {
        self.first_line + self.lines.len() as u64 - 1
    }

    fn view_start_index(&self, rows: &[VisualRow], height: usize) -> usize {
        let bottom_start = rows.len().saturating_sub(height);

        let Some(view_start) = self.view_start else {
            return bottom_start;
        };

        let Some(first_row) = rows
            .iter()
            .position(|row| row.position.line == view_start.line)
        else {
            return bottom_start;
        };
        let row_count = rows[first_row..]
            .iter()
            .take_while(|row| row.position.line == view_start.line)
            .count();
        let wrapped_row = view_start.wrapped_row.min(row_count.saturating_sub(1));

        (first_row + wrapped_row).min(bottom_start)
    }

    fn layout_rows(&self, width: usize) -> Vec<VisualRow> {
        let width = width.max(1);
        let mut rows = Vec::new();

        for (line_index, line) in self.lines.iter().enumerate() {
            let absolute_line = self.first_line + line_index as u64;
            let mut wrapped_row = 0;
            let mut current_text = String::new();
            let mut current_width = 0;

            for character in line.chars() {
                let character_width = character.width().unwrap_or(0);

                if current_width > 0 && current_width + character_width > width {
                    rows.push(VisualRow {
                        position: ViewPosition {
                            line: absolute_line,
                            wrapped_row,
                        },
                        text: current_text,
                    });
                    wrapped_row += 1;
                    current_text = String::new();
                    current_width = 0;
                }

                current_text.push(character);
                current_width += character_width;
            }

            rows.push(VisualRow {
                position: ViewPosition {
                    line: absolute_line,
                    wrapped_row,
                },
                text: current_text,
            });
        }

        rows
    }
}
