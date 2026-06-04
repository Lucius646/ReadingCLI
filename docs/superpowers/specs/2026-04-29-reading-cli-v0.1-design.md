# Reading CLI v0.1 设计文档

日期：2026-04-29

## 目标

v0.1 的目标是做出一个可运行的 Rust 终端小说阅读器最小版本：

```text
reading open novel.txt
```

进入阅读后，程序显示固定大小的文本块，用户通过 `n`、`p`、`q` 切换和退出：

```text
n 下一块
p 上一块
q 退出并保存进度
```

这个版本优先学习工程基本功：项目结构、数据建模、状态转移、文件读取、编码处理、缓存、持久化和测试。它不追求最终 TUI 体验。

## 范围

v0.1 支持：

- 只打开本地 `.txt` 小说文件。
- 支持 UTF-8 和 GBK 中文文本。
- 使用固定文本块切页。
- 使用 `n`、`p`、`q` 进行阅读操作。
- 将当前阅读进度保存到工作区内的元数据文件。
- `TextSource` 按需读取文本，并缓存当前块上下各 10 块。
- CLI 和状态机不一次性加载或持有整本小说正文。

v0.1 不支持：

- 全屏 TUI。
- 上下滚动阅读。
- 多书籍书架交互。
- 书籍 ID 设计。
- 章节识别。
- 搜索、书签、笔记。
- 根据终端窗口大小动态排版。

## 核心设计原则

核心边界是：

```text
ReadingSession 管状态
TextSource 管正文访问
BookMetadata 管持久化元数据
```

状态机只记录“当前读到哪里”和“是否继续运行”，不拥有整本正文。正文访问由 `TextSource` 负责，它可以使用文件读取、块索引和缓存来提供当前要显示的文本块。

这个边界很重要。以后即使从普通 CLI 升级为 TUI，或者从固定文本块升级为按屏幕动态排版，也不需要推翻状态模型。

## 数据结构

### BookMetadata

`BookMetadata` 是可以保存到文件里的书籍元数据。

```text
BookMetadata
  path: 文本文件路径
  title: 书名，v0.1 先使用文件名
  encoding: utf-8 或 gbk
  file_size: 文件字节大小
  modified_at: 文件最后修改时间
  total_chars: 解码后的总字符数
  total_blocks: 总块数
  current_block_index: 当前阅读块号
  block_size: 每块显示多少字符，v0.1 默认 1200
```

`file_size` 和 `modified_at` 用来判断缓存索引是否还可信。如果小说文件被替换或修改，旧索引应该废弃并重建。

### ReadingSession

`ReadingSession` 是程序运行时的轻量状态。

```text
ReadingSession
  metadata: BookMetadata
  running: 是否继续阅读
```

它不包含小说正文。正文由 `TextSource` 按需提供。

### ReaderEvent

用户输入会被转换成事件：

```text
ReaderEvent
  Next
  Previous
  Quit
```

状态转移规则：

```text
Next:
  current_block_index = min(current_block_index + 1, total_blocks - 1)

Previous:
  current_block_index = max(current_block_index - 1, 0)

Quit:
  running = false
```

每次事件处理后，显示层再向 `TextSource` 请求当前块内容。

## 存储布局

v0.1 在当前工作区创建 `.reading` 目录：

```text
.reading/
  current-book.json
  cache/
    current-book.index.json
```

`current-book.json` 保存当前文本的元数据。v0.1 不处理多本书交互，因此暂时不设计书籍 ID。

未来书架版本可以演进为：

```text
.reading/
  library.json
  books/
    <book-id>.json
  cache/
    <book-id>.index.json
```

## TextSource 设计

`TextSource` 负责文本访问。它的对外接口可以理解为：

```text
TextSource.open(path, encoding, block_size)
TextSource.get_block(block_index)
```

它内部维护：

```text
TextSource
  path
  encoding
  block_size
  block_index
  cache_window
```

### 块索引

为了避免每次切页都从文件开头扫描，首次打开文件时建立块索引：

```text
block 0 -> byte offset 0
block 1 -> byte offset ...
block 2 -> byte offset ...
```

索引记录“每个文本块从文件的哪个字节位置开始”。索引构建时需要顺序扫描文件、按编码解码、按字符数计数，但不保存整本正文。

注意：UTF-8 和 GBK 都不是简单的“一个字符等于一个字节”。因此索引不能用 `block_index * block_size` 直接计算字节位置，必须通过解码扫描得到安全的字节边界。

### 读取文本块

当阅读器请求某一块：

```text
get_block(block_index)
```

`TextSource` 执行：

```text
1. 先查内存缓存。
2. 如果命中，直接返回当前块文本。
3. 如果未命中，根据块索引 seek 到文件对应 byte offset。
4. 从该位置读取当前块附近的文本。
5. 解码并切成文本块。
6. 更新缓存窗口。
7. 返回请求的文本块。
```

### 缓存窗口

v0.1 使用当前块上下各 10 块缓存：

```text
current_block - 10
...
current_block
...
current_block + 10
```

最多缓存 21 块。若每块 1200 字符，约缓存 25200 字符。这个内存占用对普通中文小说很小，但能让连续翻页基本命中缓存。

缓存属于 `TextSource` 的内部性能细节，不保存到 `ReadingSession`。

## 编码策略

v0.1 只处理 UTF-8 和 GBK。

打开文件时：

```text
1. 先尝试按 UTF-8 解码。
2. 如果失败，再按 GBK 解码。
3. 记录最终使用的 encoding。
```

如果后续实现中发现 UTF-8 检测和 GBK 回退不够稳定，再引入更明确的编码探测策略。v0.1 先保持规则简单。

## 用户流程

### 首次打开

```text
reading open novel.txt
```

流程：

```text
1. 校验文件存在且扩展名为 .txt。
2. 检测编码。
3. 建立或读取块索引。
4. 创建 BookMetadata。
5. 创建 ReadingSession。
6. TextSource 读取当前块及附近缓存。
7. 显示当前块。
8. 等待用户输入 n/p/q。
```

### 再次打开

如果 `.reading/current-book.json` 指向同一个文件，且文件大小和修改时间未变：

```text
1. 读取已有 BookMetadata。
2. 读取或复用块索引。
3. 从 current_block_index 继续显示。
```

如果文件发生变化：

```text
1. 废弃旧索引。
2. 重新建立索引。
3. 如果 current_block_index 超出范围，回退到最后一块。
```

### 退出

用户输入 `q`：

```text
1. ReaderEvent::Quit 修改 running = false。
2. 将当前 BookMetadata 写入 .reading/current-book.json。
3. 程序退出。
```

## 错误处理

v0.1 需要明确处理这些错误：

- 命令格式错误：提示正确用法。
- 文件不存在：提示路径无效。
- 文件不是 `.txt`：提示 v0.1 只支持 txt。
- 编码无法处理：提示当前只支持 UTF-8 和 GBK。
- 元数据文件损坏：忽略旧元数据，重新创建。
- 索引文件损坏：删除或忽略旧索引，重新构建。
- 文件被修改：重建索引，并校正当前块号。

错误信息应优先使用中文，符合项目文档和用户使用语境。

## 测试范围

v0.1 至少需要这些测试：

- `ReaderEvent::Next` 在中间块能前进一块。
- `ReaderEvent::Next` 在最后一块不会越界。
- `ReaderEvent::Previous` 在中间块能后退一块。
- `ReaderEvent::Previous` 在第一块不会变成负数。
- `ReaderEvent::Quit` 会将 `running` 设为 `false`。
- `BookMetadata` 可以序列化和反序列化。
- 元数据文件损坏时可以回退重建。
- `TextSource` 可以从 UTF-8 txt 中读取指定块。
- `TextSource` 可以从 GBK txt 中读取指定块。
- 缓存命中时不需要重新读取对应块。

其中状态转移测试最适合先写，因为它们不依赖真实文件，能帮助我们快速验证状态机模型。

## 后续演进

v0.2 可以加入书架管理：

```text
reading add novel.txt
reading list
reading open <title 或 book-id>
```

v0.3 可以加入 TUI：

```text
reading
```

TUI 阶段会引入：

- 全屏阅读界面。
- 根据终端宽高动态排版。
- `j/k`、方向键、空格、PageUp/PageDown。
- 状态栏。
- 滚动或屏幕级翻页。

这些能力应复用 v0.1 的核心边界：状态机不拥有正文，正文仍由文本源提供。
