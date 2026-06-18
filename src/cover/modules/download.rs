use std::time::Duration;

use humansize::{FormatSizeOptions, format_size};
use humantime::format_duration;
use progress_string::BarBuilder;
use rand::RngExt;
use rand::seq::IndexedRandom;

use crate::cover::data::{CFILES_LIST, EXTENSIONS_LIST};
use crate::cover::generators::gen_file_name_with_ext;
use crate::cover::{CoverContext, CoverFrame, CoverModule, CoverOp};

struct DownloadState {
    file_name: String,
    file_bytes: u64,
    bytes_downloaded: u64,
}

pub struct DownloadModule {
    extension: &'static str,
    download_speed: i64,
    remaining_files: usize,
    current: Option<DownloadState>,
}

impl Default for DownloadModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadModule {
    /// 创建一组使用相同扩展名和近似速度的模拟下载任务。
    pub fn new() -> Self {
        let mut rng = rand::rng();

        Self {
            extension: EXTENSIONS_LIST.choose(&mut rng).copied().unwrap_or("bin"),
            download_speed: rng.random_range(10_000_000..100_000_000),
            remaining_files: rng.random_range(3..10),
            current: None,
        }
    }

    fn start_file(&mut self) {
        let mut rng = rand::rng();
        self.current = Some(DownloadState {
            file_name: gen_file_name_with_ext(&mut rng, &CFILES_LIST, self.extension),
            file_bytes: rng.random_range(30_000_000..300_000_000),
            bytes_downloaded: 0,
        });
    }
}

impl CoverModule for DownloadModule {
    fn name(&self) -> &'static str {
        "download"
    }

    fn signature(&self) -> String {
        "wget -i downloads.txt".to_string()
    }

    fn next_frame(&mut self, context: &CoverContext) -> Option<CoverFrame> {
        if self.current.is_none() {
            if self.remaining_files == 0 {
                return None;
            }

            if context.output_width < 55 {
                self.remaining_files -= 1;
                return Some(CoverFrame::new(
                    vec![
                        CoverOp::Write(
                            "Terminal too small to display download progress".to_string(),
                        ),
                        CoverOp::NewLine,
                    ],
                    Duration::from_millis(100),
                ));
            }

            self.start_file();
        }

        let state = self
            .current
            .as_mut()
            .expect("download state was initialized");
        let mut rng = rand::rng();
        let speed_offset = rng.random_range(-5_000_000i64..5_000_000i64);
        let actual_speed = (self.download_speed + speed_offset).max(100_000) as u64;
        let bytes_incoming = (actual_speed / 1000) * 50;
        let displayed_bytes = state.bytes_downloaded.min(state.file_bytes);
        let percent = 100.0 * displayed_bytes as f64 / state.file_bytes as f64;
        let completed = displayed_bytes >= state.file_bytes;
        let remaining_secs = state.file_bytes.saturating_sub(displayed_bytes) / actual_speed;
        let eta = Duration::from_secs(remaining_secs);

        let stats_width = 32;
        let rest_padding = 16;
        let remaining_width = context.output_width.saturating_sub(stats_width);
        let file_name_width = remaining_width / 3;
        let progress_width = remaining_width
            .saturating_sub(file_name_width + rest_padding)
            .max(1);
        let mut progress_bar = BarBuilder::new()
            .total(state.file_bytes as usize)
            .full_char('=')
            .width(progress_width)
            .build();
        progress_bar.replace(displayed_bytes as usize);

        let size_options = FormatSizeOptions::from(humansize::BINARY).space_after_value(false);
        let speed_options = FormatSizeOptions::from(humansize::BINARY)
            .space_after_value(false)
            .suffix("/s");
        let file_name: String = state.file_name.chars().take(file_name_width).collect();
        let line = format!(
            "{file_name:<file_name_width$} {percent:>4.0}%{progress_bar} {incoming:<10} {speed:<12} eta {eta:<10}",
            incoming = format_size(bytes_incoming, size_options),
            speed = format_size(actual_speed, speed_options),
            eta = format_duration(eta),
        );

        let mut ops = vec![CoverOp::EraseLine, CoverOp::Write(line)];

        if completed {
            ops.push(CoverOp::NewLine);
            self.current = None;
            self.remaining_files -= 1;
        } else {
            state.bytes_downloaded = state.bytes_downloaded.saturating_add(bytes_incoming);
        }

        Some(CoverFrame::new(ops, Duration::from_millis(50)))
    }
}
