use std::time::Duration;

use chrono::Local;
use fake::Fake;
use fake::faker::internet::en::{IPv4, IPv6, UserAgent};
use fake::faker::lorem::en::Words;
use rand::RngExt;
use rand::seq::IndexedRandom;

use crate::cover::data::{EXTENSIONS_LIST, PACKAGES_LIST};
use crate::cover::generators::gen_file_path;
use crate::cover::{CoverContext, CoverFrame, CoverModule, CoverOp};

static HTTP_CODES: &[u16] = &[200, 201, 400, 401, 403, 404, 500, 502, 503];

pub struct WeblogModule {
    remaining_lines: usize,
    burst_remaining: usize,
}

impl Default for WeblogModule {
    fn default() -> Self {
        Self::new()
    }
}

impl WeblogModule {
    /// 创建一次长度随机的 Web 访问日志会话。
    pub fn new() -> Self {
        Self {
            remaining_lines: rand::rng().random_range(50..200),
            burst_remaining: 0,
        }
    }
}

impl CoverModule for WeblogModule {
    fn name(&self) -> &'static str {
        "weblog"
    }

    fn signature(&self) -> String {
        "tail -f /var/log/nginx/access.log".to_string()
    }

    fn next_frame(&mut self, _context: &CoverContext) -> Option<CoverFrame> {
        if self.remaining_lines == 0 {
            return None;
        }

        let mut rng = rand::rng();
        let ip = if rng.random_bool(0.5) {
            IPv4().fake()
        } else {
            IPv6().fake::<String>().to_lowercase()
        };
        let date = Local::now().format("%e/%b/%Y:%T %z");
        let directories: Vec<String> = Words(20..21).fake();
        let path = gen_file_path(&mut rng, &PACKAGES_LIST, EXTENSIONS_LIST, &directories);
        let http_code = HTTP_CODES.choose(&mut rng).unwrap_or(&200);
        let size = rng.random_range(99..5_000_000);
        let user_agent: String = UserAgent().fake();
        let line = format!(
            "{ip} - - [{date}] \"GET {path} HTTP/1.0\" {http_code} {size} \"-\" \"{user_agent}\""
        );

        let delay = if self.burst_remaining > 0 {
            self.burst_remaining -= 1;
            30
        } else if rng.random_bool(1.0 / 20.0) {
            self.burst_remaining = rng.random_range(9..49);
            30
        } else {
            rng.random_range(10..1000)
        };

        self.remaining_lines -= 1;

        Some(CoverFrame::new(
            vec![CoverOp::Write(line), CoverOp::NewLine],
            Duration::from_millis(delay),
        ))
    }
}
