# Jieba POS Highlight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用 `jieba-rs` 为 ReadingCLI 增加基于词性的阅读高亮，让正文在 TUI 中按名词、动词、代词、副词、形容词呈现低干扰的视觉层次。

**Architecture:** 第一版不做主谓宾句法分析，只使用 jieba 的分词和词性标注。分析结果保存为 UTF-8 byte offset annotation，阅读页渲染时只取当前页面范围内的标注并转换成 ratatui `Span`。

**Tech Stack:** Rust 2024, `jieba-rs`, `serde`, `serde_json`, `ratatui`, existing `TextSource` / `Page` / `BookLibrary` offset model.

---

## 设计边界

- 不做实时逐页 NLP。打开书时如果没有高亮缓存，则分析整本书并保存。
- 不做主语、谓语、宾语规则。第一版只做词性高亮。
- 不高亮助词、标点、未知词，避免一屏幕颜色噪音。
- 所有标注都使用 UTF-8 byte offset，保持和当前分页、进度、GBK 转 UTF-8 缓存机制一致。
- 第一版 annotation 文件可以按当前书路径生成简单文件名，后续再升级为更稳定的 book fingerprint。

## 文件结构

### 新增

- `src/highlight/mod.rs`
  - 统一导出高亮模块。
- `src/highlight/annotation.rs`
  - 定义 `Annotation` 和 `AnnotationKind`。
- `src/highlight/jieba_analyzer.rs`
  - 调用 `jieba-rs`，把词性 tag 映射成 annotation。
- `src/highlight/store.rs`
  - 保存、读取、按页面范围查询 annotation。
- `src/highlight/render.rs`
  - 把 `Page` 和 annotation 转换成 ratatui `Line` / `Span`。
- `tests/highlight_tests.rs`
  - 验证词性映射、offset、页面范围查询、渲染不丢字。

### 修改

- `Cargo.toml`
  - 添加 `jieba-rs`。
- `src/lib.rs`
  - 暴露 `highlight` 模块。
- `src/tui.rs`
  - 创建/加载 annotation store。
  - 阅读页使用高亮渲染，而不是纯文本 `Paragraph`。

## 词性映射

第一版只映射这些 tag：

| jieba tag | 含义 | AnnotationKind |
| --- | --- | --- |
| `n`, `nr`, `ns`, `nt`, `nz` | 名词、人名、地名、机构名、专名 | `Noun` |
| `v`, `vd`, `vn` | 动词、动副词、名动词 | `Verb` |
| `r` | 代词 | `Pronoun` |
| `d` | 副词 | `Adverb` |
| `a`, `ad`, `an` | 形容词、形副词、名形词 | `Adjective` |

其余 tag 暂时忽略。

## 颜色方案

参考 VSCode 暗色主题，但保持克制：

- `Verb`: `Color::Blue`
- `Noun`: `Color::Green`
- `Pronoun`: `Color::Cyan`
- `Adverb`: `Color::Yellow`
- `Adjective`: `Color::Magenta`

后续可以把颜色抽成 theme 配置。

## Task 1: 建立高亮基础类型

**Files:**
- Create: `src/highlight/mod.rs`
- Create: `src/highlight/annotation.rs`
- Modify: `src/lib.rs`

- [ ] 定义 `AnnotationKind`。
- [ ] 定义 `Annotation { start_offset, end_offset, kind }`。
- [ ] 派生 `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`。
- [ ] 在 `src/lib.rs` 中增加 `pub mod highlight;`。
- [ ] 运行 `cargo check`。

## Task 2: 接入 jieba-rs 分析器

**Files:**
- Modify: `Cargo.toml`
- Create: `src/highlight/jieba_analyzer.rs`
- Modify: `src/highlight/mod.rs`
- Test: `tests/highlight_tests.rs`

- [ ] 添加 `jieba-rs = "0.10"`。
- [ ] 创建 `JiebaAnalyzer`。
- [ ] 实现 `analyze(&self, text: &str, base_offset: u64) -> Vec<Annotation>`。
- [ ] 实现 `map_jieba_tag(tag: &str) -> Option<AnnotationKind>`。
- [ ] 测试中文 byte offset 不错位。
- [ ] 运行 `cargo test highlight`。

## Task 3: 持久化 annotation

**Files:**
- Create: `src/highlight/store.rs`
- Modify: `src/highlight/mod.rs`
- Test: `tests/highlight_tests.rs`

- [ ] 定义 `AnnotationStore { annotations: Vec<Annotation> }`。
- [ ] 实现 `save(path)` 和 `load(path)`。
- [ ] 实现 `query(start_offset, end_offset)`。
- [ ] 实现 `annotation_path_for_book(book_path)`，第一版用路径字符串 hash 生成文件名。
- [ ] 测试保存读取和页面范围查询。

## Task 4: 渲染高亮页面

**Files:**
- Create: `src/highlight/render.rs`
- Modify: `src/highlight/mod.rs`
- Test: `tests/highlight_tests.rs`

- [ ] 实现 `render_highlighted_page(page: &Page, annotations: &[Annotation]) -> Vec<Line<'static>>`。
- [ ] 根据 annotation offset 切分 page text。
- [ ] 对不同 `AnnotationKind` 映射不同 `Style`。
- [ ] 保证没有 annotation 的文本仍然原样显示。
- [ ] 测试渲染后字符不丢失、不重复。

## Task 5: 接入 TUI 阅读页

**Files:**
- Modify: `src/tui.rs`

- [ ] 在 `TuiState` 或 `run_reader` 生命周期中加载当前书的 annotation。
- [ ] 如果 annotation 文件不存在，则读取全文并分析保存。
- [ ] 切换书籍时重新加载/生成 annotation。
- [ ] `draw_reading_screen` 根据当前 page offset 查询 annotation。
- [ ] 把正文 `Paragraph::new(page.text.as_str())` 替换成高亮 `Paragraph::new(lines)`。
- [ ] 手动验证 `cargo run -- open novel.txt`。

## Task 6: 验证与收尾

**Files:**
- Modify as needed: `README.md`
- Modify as needed: `docs/code-walkthrough.md`

- [ ] 运行 `cargo fmt --check`。
- [ ] 运行 `cargo clippy --all-targets --all-features -- -D warnings`。
- [ ] 运行 `cargo test`。
- [ ] 更新 README 的 v0.4 草案或 Roadmap。
- [ ] 提交 commit，建议 message：`feat(highlight): add jieba pos reading highlights`。

## 后续扩展

- 如果词性高亮效果可以接受，再增加 `predicate.rs` 做主谓宾近似规则。
- 如果 annotation JSON 文件过大，改为 JSONL 或二进制格式。
- 如果 jieba 对网文人名/地名切分不好，增加用户词典。
- 如果颜色太吵，增加 HomePage 设置项开关不同词性。
