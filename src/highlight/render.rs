use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthChar;

use crate::highlight::annotation::{Annotation, AnnotationKind};

#[derive(Debug, Clone)]
pub struct HighlightedPage {
    pub lines: Vec<Line<'static>>,
    pub end_offset: u64,
}

pub fn render_highlighted_page(
    text: &str,
    start_offset: u64,
    columns: u16,
    rows: u16,
    annotations: &[Annotation],
) -> HighlightedPage {
    let max_columns = columns as usize;
    let max_rows = rows as usize;

    let mut lines = vec![Line::default()];
    let mut current_row = 0usize;
    let mut current_col = 0usize;
    let mut end_offset = start_offset;

    for (byte_index, ch) in text.char_indices() {
        if current_row >= max_rows {
            break;
        }

        let char_start = start_offset + byte_index as u64;
        let char_end = char_start + ch.len_utf8() as u64;

        if ch == '\r' {
            end_offset = char_end;
            continue;
        }

        if ch == '\n' {
            current_row += 1;
            current_col = 0;
            end_offset = char_end;

            if current_row < max_rows {
                lines.push(Line::default());
            }
            continue;
        }

        let char_width = ch.width().unwrap_or(0);

        if current_col + char_width > max_columns {
            current_row += 1;
            current_col = 0;

            if current_row >= max_rows {
                break;
            }
            lines.push(Line::default());
        }

        push_char(
            &mut lines[current_row],
            ch,
            char_start,
            char_end,
            annotations,
        );
        current_col += char_width;
        end_offset = char_end;
    }

    HighlightedPage { lines, end_offset }
}

fn push_char(
    line: &mut Line<'static>,
    ch: char,
    char_start: u64,
    char_end: u64,
    annotations: &[Annotation],
) {
    let style = annotations
        .iter()
        .find(|annotation| annotation.start_offset < char_end && annotation.end_offset > char_start)
        .map(|annotation| style_for_kind(annotation.kind))
        .unwrap_or_default();

    push_styled_char(line, ch, style);
}

fn push_styled_char(line: &mut Line<'static>, ch: char, style: Style) {
    if let Some(last_span) = line.spans.last_mut()
        && last_span.style == style
    {
        last_span.content.to_mut().push(ch);
        return;
    }

    line.spans.push(Span::styled(ch.to_string(), style));
}

fn style_for_kind(kind: AnnotationKind) -> Style {
    match kind {
        AnnotationKind::Noun => Style::default().fg(Color::Green),
        AnnotationKind::Verb => Style::default().fg(Color::Blue),
        AnnotationKind::Pronoun => Style::default().fg(Color::Cyan),
        AnnotationKind::Adverb => Style::default().fg(Color::Yellow),
        AnnotationKind::Adjective => Style::default().fg(Color::Magenta),
        AnnotationKind::Person => Style::default().fg(Color::LightRed),
        AnnotationKind::Location => Style::default().fg(Color::LightBlue),
        AnnotationKind::Organization => Style::default().fg(Color::LightYellow),
    }
}
