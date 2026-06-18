use std::time::Instant;

use super::{CoverContext, CoverFrame, CoverModule, CoverOp, CoverRegistry, CoverTerminal};

pub struct CoverEngine {
    terminal: CoverTerminal,
    registry: CoverRegistry,
    current_module: Option<Box<dyn CoverModule>>,
    next_step_at: Instant,
}

impl CoverEngine {
    /// 创建 Cover Engine，并从给定时间开始调度模块。
    pub fn new(registry: CoverRegistry, max_lines: usize, now: Instant) -> Self {
        Self {
            terminal: CoverTerminal::new(max_lines),
            registry,
            current_module: None,
            next_step_at: now,
        }
    }

    /// 在时间到达时推进当前模块一步。
    pub fn tick(&mut self, now: Instant, context: &CoverContext) {
        if now < self.next_step_at {
            return;
        }

        if self.current_module.is_none() {
            self.start_random_module(now);
            return;
        }

        let frame = self
            .current_module
            .as_mut()
            .and_then(|module| module.next_frame(context));

        match frame {
            Some(frame) => self.execute_frame(frame, now),
            None => self.current_module = None,
        }
    }

    /// 返回虚拟终端的只读引用。
    pub fn terminal(&self) -> &CoverTerminal {
        &self.terminal
    }

    /// 返回虚拟终端的可变引用，用于处理滚动操作。
    pub fn terminal_mut(&mut self) -> &mut CoverTerminal {
        &mut self.terminal
    }

    fn start_random_module(&mut self, now: Instant) {
        let Some(module) = self.registry.create_random() else {
            return;
        };

        let signature = module.signature();
        self.terminal
            .apply(CoverOp::Write(format!("$ {signature}")));
        self.terminal.apply(CoverOp::NewLine);

        self.current_module = Some(module);
        self.next_step_at = now;
    }

    fn execute_frame(&mut self, frame: CoverFrame, now: Instant) {
        for op in frame.ops {
            self.terminal.apply(op);
        }

        self.next_step_at = now + frame.delay;
    }
}
