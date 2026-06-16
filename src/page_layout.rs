use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub text: String,
    pub end_offset: u64,
}

// 按终端宽高把候选文本排成一页，并返回下一页起始 offset。
pub fn layout_page(text: &str, start_offset: u64, columns: u16, rows: u16) -> Page {
    let max_columns = columns as usize;
    let max_rows = rows as usize;

    let mut output = String::new();
    let mut current_row = 0usize;
    let mut current_col = 0usize;
    let mut end_offset = start_offset;

    for (byte_index, ch) in text.char_indices() {
        if current_row >= max_rows {
            break;
        }

        if ch == '\n' {
            output.push(ch);
            current_row += 1;
            current_col = 0;
            end_offset = start_offset + byte_index as u64 + ch.len_utf8() as u64;
            continue;
        }

        let char_width = char_display_width(ch);

        if current_col + char_width > max_columns {
            current_row += 1;
            current_col = 0;

            if current_row >= max_rows {
                break;
            }
            output.push('\n');
        }

        output.push(ch);
        current_col += char_width;
        end_offset = start_offset + byte_index as u64 + ch.len_utf8() as u64;
    }

    Page {
        text: output,
        end_offset,
    }
}

// 计算字符在终端中占用的列宽。
fn char_display_width(ch: char) -> usize {
    ch.width().unwrap_or(0)
}
