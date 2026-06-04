# Reading CLI v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 由学习者亲手实现一个 Rust 终端小说阅读器 v0.1：打开本地 `.txt`，按固定文本块切页，并保存阅读进度。

**Architecture:** 程序分成 CLI 入口、元数据、阅读会话状态机、文本源、阅读循环五部分。`ReadingSession` 只保存轻量状态；`TextSource` 负责正文访问、块索引和当前块上下各 10 块缓存。

**Tech Stack:** Rust, Cargo, Git Bash, clap, serde, serde_json, anyhow, encoding_rs, tempfile(dev)

---

## 协作方式

这份计划给你在 **VS Code 内置终端的 Git Bash** 中亲手编码使用。

你负责创建文件、写代码、运行命令；我负责解释、review、排错和拆下一步。每次只做一个小检查点。完成后，把命令输出、代码片段或报错发给我。

暂时不 commit。等你熟悉基本流程后再引入 git 节奏。

## 终端约定

默认终端：

```text
VS Code Integrated Terminal + Git Bash
```

默认工作目录：

```bash
/e/LuciusProject/ReadingCLI
```

常用命令：

```bash
pwd                 # 查看当前目录
ls                  # 查看当前目录文件
mkdir -p src        # 创建目录
touch src/main.rs   # 创建空文件
cargo check         # 检查编译
cargo run           # 运行程序
cargo test          # 运行测试
```

## 文件结构目标

最终会逐步创建：

```text
Cargo.toml
src/
  main.rs
  lib.rs
  app.rs
  cli.rs
  metadata.rs
  session.rs
  text_source.rs
tests/
  session_tests.rs
  metadata_tests.rs
  text_source_tests.rs
README.md
```

不用一次创建完。按阶段推进。

## Phase 0: 环境确认

目标：确认 Rust 工具链可用，并且当前 Git Bash 位于项目目录。

- [ ] **0.1 进入项目目录**

运行：

```bash
cd /e/LuciusProject/ReadingCLI
pwd
```

期望看到：

```text
/e/LuciusProject/ReadingCLI
```

- [ ] **0.2 检查 Rust 和 Cargo**

运行：

```bash
rustc --version
cargo --version
```

期望看到类似：

```text
rustc 1.xx.x
cargo 1.xx.x
```

如果命令不存在，把完整输出发给我。

## Phase 1: 创建最小 Rust 项目

目标：让目录变成 Cargo 能识别的 Rust 项目。

### Task 1.1: 创建 `Cargo.toml`

- [ ] **创建文件**

在项目根目录创建 `Cargo.toml`，内容：

```toml
[package]
name = "reading"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
encoding_rs = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tempfile = "3"
```

可以用 VS Code 新建文件，也可以在 Git Bash 里运行：

```bash
touch Cargo.toml
```

然后用编辑器写入内容。

- [ ] **检查文件**

运行：

```bash
ls
```

应该能看到：

```text
Cargo.toml
```

### Task 1.2: 创建 `src/main.rs`

- [ ] **创建目录和文件**

运行：

```bash
mkdir -p src
touch src/main.rs
```

`src/main.rs` 内容：

```rust
fn main() {
    println!("Reading CLI");
}
```

- [ ] **运行程序**

运行：

```bash
cargo run
```

期望看到：

```text
Reading CLI
```

完成后，把 `cargo run` 输出发给我。

## Phase 2: 引入库结构

目标：把项目从单文件程序变成可测试的模块结构。

### Task 2.1: 创建 `src/lib.rs`

- [ ] **创建文件**

运行：

```bash
touch src/lib.rs
```

内容：

```rust
pub mod app;
```

### Task 2.2: 创建 `src/app.rs`

- [ ] **创建文件**

运行：

```bash
touch src/app.rs
```

内容：

```rust
use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Reading CLI");
    Ok(())
}
```

### Task 2.3: 修改 `src/main.rs`

- [ ] **改成调用 app 层**

`src/main.rs`：

```rust
fn main() {
    if let Err(err) = reading::app::run() {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}
```

- [ ] **验证编译**

运行：

```bash
cargo check
```

期望看到：

```text
Finished `dev` profile
```

完成后，把输出发给我。

## Phase 3: CLI 命令解析

目标：支持命令：

```bash
cargo run -- open novel.txt
```

先只解析命令，不真的打开文件。

### Task 3.1: 创建 `src/cli.rs`

- [ ] **创建文件**

运行：

```bash
touch src/cli.rs
```

内容：

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "reading")]
#[command(about = "终端小说阅读器")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Open { path: PathBuf },
}
```

### Task 3.2: 修改 `src/lib.rs`

内容：

```rust
pub mod app;
pub mod cli;
```

### Task 3.3: 修改 `src/app.rs`

内容：

```rust
use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Open { path } => {
            println!("准备打开: {}", path.display());
        }
    }

    Ok(())
}
```

### Task 3.4: 验证 CLI

- [ ] **查看帮助**

运行：

```bash
cargo run -- --help
```

应该能看到 `open` 子命令。

- [ ] **测试 open 命令**

运行：

```bash
cargo run -- open novel.txt
```

期望看到：

```text
准备打开: novel.txt
```

完成后，把输出发给我。

## Phase 4: 状态机

目标：先实现 `n/p/q` 状态变化，不碰文件 I/O。

### Task 4.1: 创建 `src/metadata.rs`

运行：

```bash
touch src/metadata.rs
```

内容：

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookMetadata {
    pub path: PathBuf,
    pub title: String,
    pub encoding: String,
    pub file_size: u64,
    pub modified_at: u64,
    pub total_chars: usize,
    pub total_blocks: usize,
    pub current_block_index: usize,
    pub block_size: usize,
}
```

### Task 4.2: 创建 `src/session.rs`

运行：

```bash
touch src/session.rs
```

内容：

```rust
use crate::metadata::BookMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderEvent {
    Next,
    Previous,
    Quit,
}

#[derive(Debug, Clone)]
pub struct ReadingSession {
    pub metadata: BookMetadata,
    pub running: bool,
}

impl ReadingSession {
    pub fn new(metadata: BookMetadata) -> Self {
        Self {
            metadata,
            running: true,
        }
    }

    pub fn apply(&mut self, event: ReaderEvent) {
        match event {
            ReaderEvent::Next => {
                if self.metadata.total_blocks > 0 {
                    self.metadata.current_block_index =
                        (self.metadata.current_block_index + 1)
                            .min(self.metadata.total_blocks - 1);
                }
            }
            ReaderEvent::Previous => {
                self.metadata.current_block_index =
                    self.metadata.current_block_index.saturating_sub(1);
            }
            ReaderEvent::Quit => {
                self.running = false;
            }
        }
    }
}
```

### Task 4.3: 修改 `src/lib.rs`

内容：

```rust
pub mod app;
pub mod cli;
pub mod metadata;
pub mod session;
```

### Task 4.4: 创建状态机测试

运行：

```bash
mkdir -p tests
touch tests/session_tests.rs
```

内容：

```rust
use reading::metadata::BookMetadata;
use reading::session::{ReaderEvent, ReadingSession};

fn metadata_at(block: usize, total_blocks: usize) -> BookMetadata {
    BookMetadata {
        path: "novel.txt".into(),
        title: "novel".into(),
        encoding: "utf-8".into(),
        file_size: 100,
        modified_at: 1,
        total_chars: 3600,
        total_blocks,
        current_block_index: block,
        block_size: 1200,
    }
}

#[test]
fn next_moves_forward_one_block() {
    let mut session = ReadingSession::new(metadata_at(1, 4));
    session.apply(ReaderEvent::Next);
    assert_eq!(session.metadata.current_block_index, 2);
}

#[test]
fn next_stops_at_last_block() {
    let mut session = ReadingSession::new(metadata_at(3, 4));
    session.apply(ReaderEvent::Next);
    assert_eq!(session.metadata.current_block_index, 3);
}

#[test]
fn previous_moves_back_one_block() {
    let mut session = ReadingSession::new(metadata_at(2, 4));
    session.apply(ReaderEvent::Previous);
    assert_eq!(session.metadata.current_block_index, 1);
}

#[test]
fn previous_stops_at_first_block() {
    let mut session = ReadingSession::new(metadata_at(0, 4));
    session.apply(ReaderEvent::Previous);
    assert_eq!(session.metadata.current_block_index, 0);
}

#[test]
fn quit_stops_running() {
    let mut session = ReadingSession::new(metadata_at(1, 4));
    session.apply(ReaderEvent::Quit);
    assert!(!session.running);
}
```

### Task 4.5: 运行状态机测试

运行：

```bash
cargo test --test session_tests
```

期望：

```text
5 passed
```

失败就把完整输出发给我。

## Phase 5: 元数据文件读写

目标：把 `BookMetadata` 保存到 `.reading/current-book.json`，再读回来。

### Task 5.1: 修改 `src/metadata.rs`

在顶部补充：

```rust
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
```

注意：你已经有 `use std::path::PathBuf;`，不要删掉。

在结构体后面补充：

```rust
const READING_DIR: &str = ".reading";
const CURRENT_BOOK: &str = "current-book.json";

pub fn metadata_path(workspace: &Path) -> PathBuf {
    workspace.join(READING_DIR).join(CURRENT_BOOK)
}
```

### Task 5.2: 增加保存函数

继续在 `src/metadata.rs` 后面补充：

```rust
pub fn save_current_metadata(workspace: &Path, metadata: &BookMetadata) -> Result<()> {
    let dir = workspace.join(READING_DIR);
    fs::create_dir_all(&dir)
        .with_context(|| format!("无法创建元数据目录: {}", dir.display()))?;

    let path = metadata_path(workspace);
    let content = serde_json::to_string_pretty(metadata)?;
    fs::write(&path, content)
        .with_context(|| format!("无法写入元数据文件: {}", path.display()))?;

    Ok(())
}
```

### Task 5.3: 增加读取函数

继续补充：

```rust
pub fn load_current_metadata(workspace: &Path) -> Result<Option<BookMetadata>> {
    let path = metadata_path(workspace);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("无法读取元数据文件: {}", path.display()))?;
    let metadata = serde_json::from_str(&content)
        .with_context(|| format!("元数据文件格式无效: {}", path.display()))?;

    Ok(Some(metadata))
}
```

### Task 5.4: 创建元数据测试

运行：

```bash
touch tests/metadata_tests.rs
```

内容：

```rust
use reading::metadata::{load_current_metadata, save_current_metadata, BookMetadata};

#[test]
fn saves_and_loads_current_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let metadata = BookMetadata {
        path: "novel.txt".into(),
        title: "novel".into(),
        encoding: "gbk".into(),
        file_size: 100,
        modified_at: 1,
        total_chars: 2400,
        total_blocks: 2,
        current_block_index: 1,
        block_size: 1200,
    };

    save_current_metadata(dir.path(), &metadata).unwrap();
    let loaded = load_current_metadata(dir.path()).unwrap().unwrap();

    assert_eq!(loaded, metadata);
}

#[test]
fn missing_metadata_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = load_current_metadata(dir.path()).unwrap();
    assert!(loaded.is_none());
}
```

### Task 5.5: 运行元数据测试

运行：

```bash
cargo test --test metadata_tests
```

期望：

```text
2 passed
```

## Phase 6: TextSource UTF-8 最小版

目标：先只支持 UTF-8，跑通 `TextSource::open` 和 `get_block`。

### Task 6.1: 创建 `src/text_source.rs`

运行：

```bash
touch src/text_source.rs
```

内容：

```rust
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Result};

const CACHE_BEFORE: usize = 10;
const CACHE_AFTER: usize = 10;

pub struct TextSource {
    path: PathBuf,
    encoding: String,
    block_size: usize,
    total_chars: usize,
    total_blocks: usize,
    cache: HashMap<usize, String>,
}

impl TextSource {
    pub fn open(path: PathBuf, encoding: String, block_size: usize) -> Result<Self> {
        if encoding != "utf-8" {
            bail!("当前步骤只支持 utf-8");
        }

        let content = fs::read_to_string(&path)?;
        let total_chars = content.chars().count();
        let total_blocks = total_chars.div_ceil(block_size);

        Ok(Self {
            path,
            encoding,
            block_size,
            total_chars,
            total_blocks,
            cache: HashMap::new(),
        })
    }

    pub fn total_chars(&self) -> usize {
        self.total_chars
    }

    pub fn total_blocks(&self) -> usize {
        self.total_blocks
    }
}
```

### Task 6.2: 修改 `src/lib.rs`

内容：

```rust
pub mod app;
pub mod cli;
pub mod metadata;
pub mod session;
pub mod text_source;
```

### Task 6.3: 创建 TextSource 测试

运行：

```bash
touch tests/text_source_tests.rs
```

内容：

```rust
use std::fs;

use reading::text_source::TextSource;

#[test]
fn opens_utf8_text_source() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("novel.txt");
    fs::write(&path, "一二三四五六七八九十").unwrap();

    let source = TextSource::open(path, "utf-8".to_string(), 3).unwrap();

    assert_eq!(source.total_chars(), 10);
    assert_eq!(source.total_blocks(), 4);
}
```

### Task 6.4: 运行测试

运行：

```bash
cargo test --test text_source_tests
```

期望：

```text
1 passed
```

### Task 6.5: 增加 `get_block`

在 `impl TextSource` 中补充：

```rust
pub fn get_block(&mut self, block_index: usize) -> Result<String> {
    if let Some(block) = self.cache.get(&block_index) {
        return Ok(block.clone());
    }

    let content = fs::read_to_string(&self.path)?;
    let start = block_index * self.block_size;

    let block: String = content
        .chars()
        .skip(start)
        .take(self.block_size)
        .collect();

    self.cache.insert(block_index, block.clone());
    Ok(block)
}
```

这是临时实现：未命中时会重新读文件，但调用接口是正确的。后面再升级成索引和窗口缓存。

### Task 6.6: 补充块读取测试

在 `tests/text_source_tests.rs` 后面添加：

```rust
#[test]
fn reads_utf8_block_by_index() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("novel.txt");
    fs::write(&path, "一二三四五六七八九十").unwrap();

    let mut source = TextSource::open(path, "utf-8".to_string(), 3).unwrap();

    assert_eq!(source.get_block(0).unwrap(), "一二三");
    assert_eq!(source.get_block(1).unwrap(), "四五六");
    assert_eq!(source.get_block(3).unwrap(), "十");
}
```

### Task 6.7: 运行测试

运行：

```bash
cargo test --test text_source_tests
```

期望：

```text
2 passed
```

## Phase 7: TextSource 缓存窗口

目标：把 `get_block` 升级成当前块上下各 10 块缓存。

这一步先不展开完整代码。做到 Phase 6 通过后，把你的 `src/text_source.rs` 发给我，我会带你逐行改。

预期逻辑：

```rust
let start_block = block_index.saturating_sub(CACHE_BEFORE);
let end_block = (block_index + CACHE_AFTER).min(self.total_blocks.saturating_sub(1));
```

然后重新填充 `cache`。

## Phase 8: GBK 支持

目标：支持网上下载的中文 GBK `.txt`。

这一步也先不独立做。等 UTF-8 版稳定后，我们一起处理：

- `encoding_rs::GBK`
- GBK 测试文件
- 字符边界
- 块索引策略

## Phase 9: 阅读循环

目标：把 CLI、状态机、元数据、TextSource 串起来。

预期步骤：

- 校验文件存在。
- 校验扩展名是 `.txt`。
- 创建 `TextSource`。
- 创建 `BookMetadata`。
- 创建 `ReadingSession`。
- 显示当前块。
- 读取用户输入。
- `n/p/q` 转成 `ReaderEvent`。
- 退出时保存 metadata。

等 Phase 6 或 Phase 7 稳定后再开始。

## Phase 10: README

目标：写最小中文使用说明。

`README.md`：

```markdown
# Reading CLI

Rust 终端小说阅读器实验项目。

## v0.1 功能

- 打开本地 txt
- 固定文本块切页
- n/p/q
- 保存当前阅读进度

## 使用

```bash
cargo run -- open novel.txt
```
```

## 现在从哪里开始

从 Phase 0 开始，在 Git Bash 里运行：

```bash
cd /e/LuciusProject/ReadingCLI
pwd
rustc --version
cargo --version
```

把输出发给我，我会带你进入 Phase 1。
