# ReadingCLI 当前代码导读

这份文档用于重新熟悉当前代码。它不是 README，也不是未来规划，而是解释现在 `src/` 里的文件、数据结构和主要函数为什么存在。

## 1. 阅读顺序建议

建议按这个顺序读：

1. `src/main.rs`：程序入口。
2. `src/app.rs`：启动流程、书架加载、进入 TUI。
3. `src/cli.rs`：命令行参数。
4. `src/metadata.rs`：一本书的可持久化状态。
5. `src/library.rs`：多书目书架状态。
6. `src/session.rs`：当前运行会话。
7. `src/text_source.rs`：txt 解码和按 offset 读取。
8. `src/page_layout.rs`：把文本排成一页。
9. `src/page_index.rs`：建立当前终端尺寸下的页索引。
10. `src/tui.rs`：Home / Reading / OpenInput / Select / Cover 的界面和按键。

`tui.rs` 依赖最多，最后读更容易。

## 2. 当前架构

```text
main.rs
  -> app.rs
      -> cli.rs
      -> library.rs
      -> metadata.rs
      -> session.rs
      -> text_source.rs
      -> tui.rs
          -> page_index.rs
          -> page_layout.rs
```

重要边界：

- `BookMetadata` 保存一本书的持久化状态。
- `BookLibrary` 保存所有书和当前书指针。
- `ReadingSession` 是运行时会话，不保存正文。
- `TextSource` 负责编码处理和正文读取。
- `PageIndex` 是运行时分页索引，不持久化。
- `tui.rs` 负责界面模式、按键、渲染和 TUI 内切书。

## 3. 存储模型

当前主存储是：

```text
.reading/library.json
```

旧文件：

```text
.reading/current-book.json
```

只作为迁移来源。如果 `library.json` 不存在但 `current-book.json` 存在，程序会用旧 metadata 创建新的 library。

`default.txt` 是无历史书目时的默认 txt。它可以作为当前书进入 TUI，但 Select 页面会过滤它。

## 4. 核心数据结构

### `BookMetadata`

位置：`src/metadata.rs`

表示一本书的可持久化阅读状态：

```rust
pub struct BookMetadata {
    pub book_path: PathBuf,
    pub current_offset: u64,
    pub last_opened_at: u64,
    pub file_len: u64,
}
```

- `book_path`：书籍路径。
- `current_offset`：当前阅读位置，UTF-8 字节偏移。
- `last_opened_at`：最近打开时间，Unix 秒。
- `file_len`：UTF-8 文本字节长度，用于计算百分比。

### `BookLibrary`

位置：`src/library.rs`

表示整个书架：

```rust
pub struct BookLibrary {
    pub current_book_path: PathBuf,
    pub books: Vec<BookMetadata>,
}
```

- `current_book_path`：Continue 应该打开哪本书。
- `books`：已导入书目。

### `ReadingSession`

位置：`src/session.rs`

表示当前运行中的阅读会话：

```rust
pub struct ReadingSession {
    pub metadata: BookMetadata,
    pub running: bool,
}
```

`running` 用来控制 TUI 主循环退出。

### `TextSource`

位置：`src/text_source.rs`

表示当前文本源：

```rust
pub struct TextSource {
    path: PathBuf,
    file_len: u64,
}
```

如果原文件是 GBK 或带 UTF-8 BOM，会写入 `.reading/current-text.utf8.txt`，后续统一按 UTF-8 offset 读取。

### `PageIndex`

位置：`src/page_index.rs`

表示当前终端尺寸下的页索引：

```rust
pub struct PageIndex {
    pub columns: u16,
    pub rows: u16,
    pub page_starts: Vec<u64>,
}
```

`page_starts` 保存每一页起始 offset。终端尺寸变化后需要重建。

### `TuiState`

位置：`src/tui.rs`

保存 TUI 界面状态：

```rust
struct TuiState {
    app_mode: AppMode,
    columns: u16,
    body_rows: u16,
    page_index: PageIndex,
    current_page_index: usize,
    selected_home_item: usize,
    open_input: Input,
    open_error: Option<String>,
    selected_book_index: usize,
}
```

## 5. 程序启动流程

### `cargo run`

1. `main()` 调用 `app::run()`。
2. `app::run()` 解析 CLI。
3. 加载或迁移 `.reading/library.json`。
4. 如果没有参数，从 `library.current_book()` 获取当前书。
5. 如果没有当前书，使用 `default.txt`。
6. 创建 `ReadingSession`。
7. 创建 `TextSource`。
8. 把 `TextSource::file_len()` 写入 `session.metadata.file_len`。
9. 调用 `tui::run_reader(...)`。
10. 退出后把最终 metadata 写回 library，并保存 `library.json`。

### `cargo run -- open novel.txt`

1. CLI 得到 `Command::Open { path }`。
2. `app.rs` 标准化路径。
3. 调用 `library.activate_book(path, current_timestamp())`。
4. 如果书已存在，复用原来的 offset。
5. 如果书不存在，创建新 metadata。
6. 后续流程同无参数启动。

## 6. `src/app.rs` 函数

### `run()`

应用主流程。负责加载书架、选择当前书、创建运行时对象、进入 TUI、退出后保存书架。

### `open_book_metadata(path, library)`

处理命令行 `open` 路径。它会调用 `library.activate_book`，因此不会把已存在书目的进度清零。

### `normalize_book_path(path)`

尽量把路径转成标准绝对路径。失败时保留原路径。

## 7. `src/library.rs` 函数

### `BookLibrary::new(default_book_path)`

创建默认书架。

### `current_book()`

根据 `current_book_path` 返回当前书 metadata。

### `upsert_book(book)`

插入或更新一本书。

### `activate_book(book_path, opened_at)`

把一本书设置为当前书。如果已存在，保留 offset 和 file_len，只更新 `last_opened_at`。

### `visible_books(default_book_path)`

返回 Select 页面展示的书目。过滤 default.txt，并按最近打开时间倒序排序。

### `load_library(path)`

从 JSON 文件读取书架。文件不存在时返回 `None`。

### `load_or_migrate_library(...)`

优先读取 `library.json`。如果不存在，则尝试从旧 `current-book.json` 迁移。两者都没有时创建默认书架。

### `save_library(path, library)`

把书架保存成格式化 JSON。

### `current_timestamp()`

返回当前 Unix 秒时间戳。

## 8. `src/text_source.rs` 函数

### `TextSource::new(path)`

读取 txt，识别 UTF-8 / UTF-8 BOM / GBK，并统一成后续可按 UTF-8 offset 读取的文本源。

### `read_from_offset(offset, max_bytes)`

从指定字节 offset 读取一段合法 UTF-8 字符串。

### `read_before_offset(offset, max_bytes)`

从 offset 往前读取一段合法 UTF-8 字符串。当前主流程较少使用。

### `file_len()`

返回当前 UTF-8 文本源字节长度。

### `decode_text(bytes)`

把原始字节解码成 UTF-8 字符串。

### `write_utf8_cache(content)`

把 GBK 或带 BOM 的文本写成 UTF-8 缓存文件。

## 9. `src/page_layout.rs` 函数

### `layout_page(text, start_offset, columns, rows)`

模拟终端排版，生成当前页文本和下一页 offset。

### `char_display_width(ch)`

计算字符终端列宽。中文和全角字符通常占 2 列。

## 10. `src/page_index.rs` 函数

### `PageIndex::build(text_source, columns, rows)`

从头扫描文本，建立当前终端尺寸下每一页的起始 offset。

### `page_count()`

返回总页数。

### `find_page_by_offset(offset)`

根据持久化 offset 找到当前终端尺寸下对应页。

### `page_start(page_index)`

返回指定页起始 offset。

## 11. `src/tui.rs` 模式

```rust
enum AppMode {
    Home,
    Reading,
    Cover,
    OpenInput,
    Select,
}
```

- `Home`：首页菜单。
- `Reading`：阅读正文。
- `Cover`：伪装输出。
- `OpenInput`：输入 txt 路径。
- `Select`：选择已导入书目。

## 12. `src/tui.rs` 主要函数

### `TuiState::new(...)`

创建 TUI 状态并建立初始页索引。

### `resize_if_needed(...)`

终端尺寸变化时重建页索引。

### `next_page(session)`

翻到下一页，并同步更新 offset。

### `previous_page(session)`

翻到上一页，并同步更新 offset。

### `run_reader(session, text_source, library)`

TUI 主循环。负责绘制、读取按键、分发模式、检测退出。

### `handle_home_key(...)`

处理首页菜单按键。

### `handle_open_input_key(...)`

处理路径输入、路径校验和打开新书。

### `handle_select_key(...)`

处理 Select 页面移动和打开书目。

### `handle_reading_key(...)`

处理阅读页翻页、返回首页、进入 Cover 和退出。

### `handle_cover_key(...)`

处理 Cover 页面返回和退出。

### `switch_book(...)`

切换当前书。它会保存旧书进度、激活新书、重建 `TextSource` 和 `PageIndex`。

### `draw(...)`

根据当前 `AppMode` 分发到具体页面绘制函数。

### `draw_home(...)`

渲染首页菜单。

### `draw_open_input(...)`

渲染路径输入框和错误信息。

### `draw_select(...)`

渲染书架列表、阅读百分比、offset 和最近打开时间。

### `render_input_line(input)`

把 `tui-input` 的输入内容和光标位置渲染成高亮文本。

### `book_title(path)`

从路径中提取不带扩展名的文件名。

### `progress_percent(current_offset, file_len)`

计算阅读百分比。

### `format_timestamp(timestamp)`

把 Unix 秒格式化成 `YYYY-MM-DD HH:MM UTC`。

### `TerminalGuard::enter()`

进入 raw mode、alternate screen，并隐藏光标。

### `Drop for TerminalGuard`

退出 TUI 时恢复光标、退出 alternate screen、关闭 raw mode。

## 13. 关键数据流

### 阅读位置

```text
BookMetadata.current_offset
  -> ReadingSession.metadata.current_offset
  -> PageIndex.find_page_by_offset
  -> 翻页更新 offset
  -> 退出时 upsert 回 BookLibrary
  -> save_library 写入 library.json
```

### 打开新书

```text
OpenInput / Select / open 命令
  -> library.activate_book
  -> session.metadata = metadata
  -> TextSource::new
  -> PageIndex::build
  -> AppMode::Reading
```

### Select 展示

```text
library.visible_books(default.txt)
  -> book_title
  -> progress_percent
  -> format_timestamp
  -> draw_select
```

## 14. 修改边界

- 不要把正文放进 `ReadingSession`。
- 不要把 `PageIndex` 持久化。
- 不要用页码保存进度，保存 offset。
- 不要让 `app.rs` 处理具体 TUI 按键。
- 不要让 `TextSource` 处理界面排版。
