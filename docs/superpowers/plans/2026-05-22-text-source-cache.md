# TextSource Cache Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 `TextSource` 模块，让 CLI 能按固定字符块从 UTF-8 `.txt` 文件读取正文，并缓存当前块上下若干块。

**Architecture:** `ReadingSession` 继续只维护 `current_block` 和运行状态；`TextSource` 负责扫描 txt、建立块偏移索引、读取文本块、维护缓存；`app.rs` 只负责把 session 的当前块编号交给 `TextSource` 并打印返回文本。第一版只支持 UTF-8，不处理 GBK，不持久化 block index。

**Tech Stack:** Rust, Cargo, Git Bash, anyhow, tempfile(dev), std::fs, std::io::{Read, Seek, SeekFrom}, HashMap

---

## 协作约定

这份计划给用户在 VS Code 内置 Git Bash 中手动执行。暂时不要求 commit，不使用严格 TDD，只保留一个关键测试来验证“按字符切块”没有写偏。

默认工作目录：

```bash
/e/LuciusProject/ReadingCLI
```

常用验证命令：

```bash
cargo check
cargo test
cargo run -- open novel.txt
```

## 当前状态

已经完成：

- `src/cli.rs`：支持 `open <path>`。
- `src/metadata.rs`：定义 `BookMetadata`，支持 JSON 序列化/反序列化。
- `src/session.rs`：定义 `ReadingSession`，支持 `n/p/q` 状态变化。
- `src/app.rs`：能读取/保存 `.reading/current-book.json`，并处理 `n/p/q` 输入。

待完成：

- `src/text_source.rs`：当前是半成品，需要修正结构并实现缓存读取。
- `tests/text_source_tests.rs`：当前存在模块名拼写错误和字符串问题，需要修正。
- `src/app.rs`：还没有接入 `TextSource` 显示正文。

## 文件职责

- `src/text_source.rs`
  - 定义 `TextSource`。
  - 启动时扫描 UTF-8 txt，建立 block 起止字节偏移。
  - `read_block(block_index)` 返回指定块文本。
  - 缓存当前块上下 `cache_radius` 块。

- `tests/text_source_tests.rs`
  - 验证中文 UTF-8 文本能按“字符数”切块，而不是按字节切块。

- `src/app.rs`
  - 创建 `TextSource`。
  - 每轮循环先显示当前文本块，再读取用户命令。

## Task 1: 修正 `TextSource` 骨架

**Files:**

- Modify: `src/text_source.rs`

- [ ] **Step 1: 确认当前文件内容**

运行：

```bash
sed -n '1,200p' src/text_source.rs
```

如果 Git Bash 没有 `sed`，就在 VS Code 里直接打开 `src/text_source.rs`。

- [ ] **Step 2: 写入正确的 imports 和结构体**

目标内容：

```rust
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;

pub struct TextSource {
    path: PathBuf,
    block_size: usize,
    cache_radius: usize,
    block_offsets: Vec<u64>,
    cache: HashMap<usize, String>,
}
```

字段含义：

- `path`：txt 文件路径。
- `block_size`：每块多少字符。
- `cache_radius`：当前块上下缓存多少块。
- `block_offsets`：每个文本块的起始字节位置，最后额外保存 EOF 位置作为结束哨兵。
- `cache`：已经读出的文本块缓存。

## Task 2: 实现 `new()`

**Files:**

- Modify: `src/text_source.rs`

- [ ] **Step 1: 在 `TextSource` 后添加 `impl TextSource`**

写入：

```rust
impl TextSource {
    pub fn new(path: PathBuf, block_size: usize, cache_radius: usize) -> Result<Self> {
        let content = fs::read_to_string(&path)?;

        let mut block_offsets = vec![0];

        for (char_count, (byte_index, _ch)) in content.char_indices().enumerate() {
            if char_count > 0 && char_count % block_size == 0 {
                block_offsets.push(byte_index as u64);
            }
        }

        let file_len = content.len() as u64;
        if block_offsets.last().copied() != Some(file_len) {
            block_offsets.push(file_len);
        }

        Ok(Self {
            path,
            block_size,
            cache_radius,
            block_offsets,
            cache: HashMap::new(),
        })
    }
}
```

说明：

- `char_indices()` 给出每个字符的起始字节位置。
- `enumerate()` 给出当前是第几个字符。
- `block_offsets` 记录每个块从哪个字节开始。
- 最后把 `file_len` 推入 `block_offsets`，方便读取最后一块。

- [ ] **Step 2: 检查编译**

运行：

```bash
cargo check
```

预期：

```text
Finished `dev` profile ...
```

如果报错，先看是否是括号位置错误。`Ok(Self { ... })` 必须在 `new()` 函数内部。

## Task 3: 实现 `read_block()`

**Files:**

- Modify: `src/text_source.rs`

- [ ] **Step 1: 扩展 imports**

把 imports 改成需要文件定位读取的版本：

```rust
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
```

- [ ] **Step 2: 在 `impl TextSource` 里添加公开方法**

```rust
pub fn read_block(&mut self, block_index: usize) -> Result<String> {
    if let Some(block) = self.cache.get(&block_index) {
        return Ok(block.clone());
    }

    self.load_cache_around(block_index)?;

    Ok(self.cache.get(&block_index).cloned().unwrap_or_default())
}
```

行为：

- 缓存里有，直接返回。
- 缓存里没有，加载当前块附近的缓存窗口。
- 如果块编号超出范围，返回空字符串。

## Task 4: 实现缓存窗口

**Files:**

- Modify: `src/text_source.rs`

- [ ] **Step 1: 添加 `block_count()`**

```rust
fn block_count(&self) -> usize {
    self.block_offsets.len().saturating_sub(1)
}
```

- [ ] **Step 2: 添加 `load_cache_around()`**

```rust
fn load_cache_around(&mut self, block_index: usize) -> Result<()> {
    self.cache.clear();

    let block_count = self.block_count();
    if block_count == 0 || block_index >= block_count {
        return Ok(());
    }

    let start = block_index.saturating_sub(self.cache_radius);
    let end = block_index
        .saturating_add(self.cache_radius)
        .min(block_count - 1);

    for index in start..=end {
        let block = self.read_block_from_file(index)?;
        self.cache.insert(index, block);
    }

    Ok(())
}
```

说明：

- 第一版直接 `cache.clear()`，让缓存始终只保存当前窗口。
- `saturating_sub` 避免第 0 块往前减时溢出。
- `min(block_count - 1)` 避免超过最后一块。

## Task 5: 实现按文件偏移读取块

**Files:**

- Modify: `src/text_source.rs`

- [ ] **Step 1: 添加 `read_block_from_file()`**

```rust
fn read_block_from_file(&self, block_index: usize) -> Result<String> {
    let start = self.block_offsets[block_index];
    let end = self.block_offsets[block_index + 1];

    let mut file = File::open(&self.path)?;
    file.seek(SeekFrom::Start(start))?;

    let mut buffer = vec![0; (end - start) as usize];
    file.read_exact(&mut buffer)?;

    let text = String::from_utf8(buffer)?;
    Ok(text)
}
```

说明：

- `seek` 跳到该块起始字节。
- 只读取这一块对应的字节。
- `String::from_utf8` 把 UTF-8 字节转回 `String`。

- [ ] **Step 2: 检查编译**

运行：

```bash
cargo check
```

预期：

```text
Finished `dev` profile ...
```

## Task 6: 修正并运行 TextSource 测试

**Files:**

- Modify: `tests/text_source_tests.rs`

- [ ] **Step 1: 修正测试文件**

目标内容：

```rust
use std::fs;

use reading_cli::text_source::TextSource;

#[test]
fn text_source_reads_utf8_text_by_character_block() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join("novel.txt");

    fs::write(&file_path, "一二三四五六七八九十")?;

    let mut source = TextSource::new(file_path, 4, 1)?;

    assert_eq!(source.read_block(0)?, "一二三四");
    assert_eq!(source.read_block(1)?, "五六七八");
    assert_eq!(source.read_block(2)?, "九十");

    Ok(())
}
```

注意：

- 模块名是 `text_source`，不是 `text_sourece`。
- `TextSource::new(...)` 返回 `Result<Self>`，所以后面要加 `?`。

- [ ] **Step 2: 运行测试**

运行：

```bash
cargo test
```

预期：

```text
test result: ok
```

## Task 7: 接入 `app.rs` 显示正文

**Files:**

- Modify: `src/app.rs`

- [ ] **Step 1: 引入 `TextSource`**

在 imports 中添加：

```rust
use crate::text_source::TextSource;
```

- [ ] **Step 2: 创建 `TextSource`**

在创建 `ReadingSession` 后添加：

```rust
let mut text_source = TextSource::new(
    session.metadata.book_path.clone(),
    session.metadata.block_size,
    10,
)?;
```

- [ ] **Step 3: 在循环里显示当前块正文**

在 `while session.running {` 里面，显示 `current block` 之前添加：

```rust
let text = text_source.read_block(session.metadata.current_block)?;
println!("{text}");
```

建议循环顺序变成：

```text
显示当前文本块
显示 current block
读取 command
根据 n/p/q 改状态
```

- [ ] **Step 4: 检查编译**

运行：

```bash
cargo check
```

预期：

```text
Finished `dev` profile ...
```

## Task 8: 手动验收

**Files:**

- No code changes

- [ ] **Step 1: 准备测试小说**

如果项目根目录还没有 `novel.txt`，创建一个：

```bash
touch novel.txt
```

然后在 VS Code 里写入一些中文内容。

- [ ] **Step 2: 运行程序**

```bash
cargo run -- open novel.txt
```

手动输入：

```text
n
n
p
q
```

预期：

- 每次循环能显示一块正文。
- `n` 后显示下一块。
- `p` 后回到上一块。
- `q` 后退出并保存 `.reading/current-book.json`。

## 暂不处理的问题

第一版不做这些：

- GBK 解码。
- `.reading/cache/current-book.index.json` 持久化索引。
- 多书籍 metadata。
- TUI 界面。
- 缓存命中率统计。
- 复用同一个 `File` 句柄优化读取。

这些不是被否定，而是等 UTF-8 + seek + cache 流程跑通后再做。
