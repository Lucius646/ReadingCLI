use std::time::{Duration, Instant};

use reading_cli::cover::modules::{CargoModule, DownloadModule, WeblogModule};
use reading_cli::cover::{
    CoverContext, CoverEngine, CoverFrame, CoverModule, CoverOp, CoverRegistry, CoverTerminal,
};

fn visible_lines(terminal: &CoverTerminal, height: usize) -> Vec<String> {
    terminal.visible_rows(100, height)
}

#[test]
fn terminal_executes_write_erase_and_cursor_operations() {
    let mut terminal = CoverTerminal::new(10);

    terminal.apply(CoverOp::Write("first".to_string()));
    terminal.apply(CoverOp::NewLine);
    terminal.apply(CoverOp::Write("second".to_string()));
    terminal.apply(CoverOp::CursorUp(1));
    terminal.apply(CoverOp::EraseLine);
    terminal.apply(CoverOp::Write("replaced".to_string()));

    assert_eq!(visible_lines(&terminal, 10), ["replaced", "second"]);
}

#[test]
fn terminal_trims_old_lines_at_its_history_limit() {
    let mut terminal = CoverTerminal::new(3);

    for (index, line) in ["one", "two", "three", "four"].into_iter().enumerate() {
        if index > 0 {
            terminal.apply(CoverOp::NewLine);
        }
        terminal.apply(CoverOp::Write(line.to_string()));
    }

    assert_eq!(visible_lines(&terminal, 10), ["two", "three", "four"]);
}

#[test]
fn terminal_scrolls_history_and_returns_to_follow_mode() {
    let mut terminal = CoverTerminal::new(10);

    for (index, line) in ["zero", "one", "two", "three", "four"]
        .into_iter()
        .enumerate()
    {
        if index > 0 {
            terminal.apply(CoverOp::NewLine);
        }
        terminal.apply(CoverOp::Write(line.to_string()));
    }

    assert_eq!(visible_lines(&terminal, 2), ["three", "four"]);

    terminal.scroll_up(1, 100, 2);
    assert_eq!(visible_lines(&terminal, 2), ["two", "three"]);

    terminal.scroll_down(1, 100, 2);
    assert_eq!(visible_lines(&terminal, 2), ["three", "four"]);
}

#[test]
fn terminal_wraps_long_lines_to_the_current_width() {
    let mut terminal = CoverTerminal::new(10);
    terminal.apply(CoverOp::Write("abcdefghij".to_string()));

    assert_eq!(terminal.visible_rows(4, 10), ["abcd", "efgh", "ij"]);
}

#[test]
fn terminal_wraps_wide_unicode_characters_by_display_width() {
    let mut terminal = CoverTerminal::new(10);
    terminal.apply(CoverOp::Write("一二三A".to_string()));

    assert_eq!(terminal.visible_rows(4, 10), ["一二", "三A"]);
}

#[test]
fn terminal_reflows_existing_lines_after_resize() {
    let mut terminal = CoverTerminal::new(10);
    terminal.apply(CoverOp::Write("abcdefghij".to_string()));

    assert_eq!(terminal.visible_rows(6, 10), ["abcdef", "ghij"]);
    assert_eq!(terminal.visible_rows(3, 10), ["abc", "def", "ghi", "j"]);
}

#[test]
fn terminal_scrolls_by_visual_rows() {
    let mut terminal = CoverTerminal::new(10);
    terminal.apply(CoverOp::Write("abcdefghijkl".to_string()));

    assert_eq!(terminal.visible_rows(4, 2), ["efgh", "ijkl"]);

    terminal.scroll_up(1, 4, 2);
    assert_eq!(terminal.visible_rows(4, 2), ["abcd", "efgh"]);

    terminal.scroll_down(1, 4, 2);
    assert_eq!(terminal.visible_rows(4, 2), ["efgh", "ijkl"]);
}

#[test]
fn frame_keeps_operations_and_delay_together() {
    let frame = CoverFrame::new(
        vec![CoverOp::Write("line".to_string()), CoverOp::NewLine],
        Duration::from_millis(250),
    );

    assert_eq!(frame.ops.len(), 2);
    assert_eq!(frame.delay, Duration::from_millis(250));
}

struct OneShotModule {
    emitted: bool,
}

impl CoverModule for OneShotModule {
    fn name(&self) -> &'static str {
        "one-shot"
    }

    fn signature(&self) -> String {
        "one-shot".to_string()
    }

    fn next_frame(&mut self, _context: &CoverContext) -> Option<CoverFrame> {
        if self.emitted {
            return None;
        }

        self.emitted = true;
        Some(CoverFrame::new(
            vec![CoverOp::Write("frame".to_string()), CoverOp::NewLine],
            Duration::ZERO,
        ))
    }
}

fn create_one_shot_module() -> Box<dyn CoverModule> {
    Box::new(OneShotModule { emitted: false })
}

#[test]
fn registry_creates_registered_modules() {
    let mut registry = CoverRegistry::new();
    registry.register("one-shot", create_one_shot_module);

    assert_eq!(registry.names().collect::<Vec<_>>(), ["one-shot"]);
    assert_eq!(registry.create_random().unwrap().name(), "one-shot");
}

#[test]
fn engine_runs_a_module_and_starts_another_after_it_finishes() {
    let mut registry = CoverRegistry::new();
    registry.register("one-shot", create_one_shot_module);

    let now = Instant::now();
    let context = CoverContext { output_width: 80 };
    let mut engine = CoverEngine::new(registry, 20, now);

    engine.tick(now, &context);
    engine.tick(now, &context);
    engine.tick(now, &context);
    engine.tick(now, &context);

    assert_eq!(
        visible_lines(engine.terminal(), 20),
        ["$ one-shot", "frame", "$ one-shot", ""]
    );
}

#[test]
fn cargo_module_runs_all_stages_and_finishes() {
    let context = CoverContext { output_width: 100 };
    let mut module = CargoModule::new();
    let mut output = String::new();
    let mut frame_count = 0;

    while let Some(frame) = module.next_frame(&context) {
        frame_count += 1;
        assert!(frame_count <= 200);

        for op in frame.ops {
            if let CoverOp::Write(text) = op {
                output.push_str(&text);
                output.push('\n');
            }
        }
    }

    assert!(output.contains("Downloading"));
    assert!(output.contains("Compiling"));
    assert!(output.contains("Finished"));
}

#[test]
fn weblog_module_generates_http_lines_and_finishes() {
    let context = CoverContext { output_width: 100 };
    let mut module = WeblogModule::new();
    let mut frame_count = 0;

    while let Some(frame) = module.next_frame(&context) {
        frame_count += 1;
        assert!(frame_count < 200);
        assert!(
            frame
                .ops
                .iter()
                .any(|op| matches!(op, CoverOp::Write(text) if text.contains("HTTP/1.0")))
        );
    }

    assert!((50..200).contains(&frame_count));
}

#[test]
fn download_module_produces_an_in_place_progress_frame() {
    let context = CoverContext { output_width: 100 };
    let mut module = DownloadModule::new();
    let frame = module.next_frame(&context).expect("download frame");

    assert!(matches!(frame.ops.first(), Some(CoverOp::EraseLine)));
    assert!(
        frame
            .ops
            .iter()
            .any(|op| matches!(op, CoverOp::Write(text) if text.contains("eta")))
    );
}

#[test]
fn download_module_skips_files_when_the_terminal_is_too_narrow() {
    let context = CoverContext { output_width: 40 };
    let mut module = DownloadModule::new();
    let mut frame_count = 0;

    while let Some(frame) = module.next_frame(&context) {
        frame_count += 1;
        assert!(frame_count < 10);
        assert!(
            frame.ops.iter().any(
                |op| matches!(op, CoverOp::Write(text) if text.contains("Terminal too small"))
            )
        );
    }

    assert!((3..10).contains(&frame_count));
}
