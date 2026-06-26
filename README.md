# ReadingCLI

ReadingCLI 是一个用 Rust 编写的终端小说阅读器。

当前版本定位为 **v0.4.0：自然语言高亮实验版**。它在 v0.3 的动态分页、多书目、全屏 TUI 和 Cover Mode 基础上，加入了基于 analyzer 的高亮架构，并尝试接入 `jieba-rs` 与 LTP legacy 模型。

## v0.4.0 功能简述

- 支持无参数启动，默认进入全屏首页。
- 保留 `cargo run -- open <txt-path>` 直接打开本地小说。
- 首页提供 Continue、Open New Book、Select、Analyzer 和 Quit。
- 支持 UTF-8，并在解码失败时回退到 GBK。
- 根据终端宽高动态分页，resize 后自动重建分页索引。
- 使用书架保存多本书的绝对路径、阅读 offset 和最近打开时间。
- 使用 ratatui/crossterm 绘制全屏 TUI。
- Cover Mode 动态模拟 Cargo、Web 访问日志和文件下载活动。
- 新增 analyzer 抽象，支持不同文本分析器生成高亮 annotation。
- 默认支持 `jieba` 词性高亮。
- 实验性支持 LTP legacy POS/NER 高亮。
- annotation cache 按书籍和 analyzer 分离，避免不同分析器结果互相污染。
- 提供 `examples/analyzer_probe.rs` 和 `examples/ltp_news_probe.rs` 用于比较分析器效果。

## 使用

准备 Rust 环境后，在项目目录运行：

```bash
cargo run
```

也可以直接打开一本本地小说：

```bash
cargo run -- open novel.txt
```

阅读界面按键：

```text
n      下一页
p      上一页
h/Esc  返回首页
c      进入 Cover Mode
q      退出并保存进度
```

首页中的 Analyzer 选项可以在当前支持的分析器之间切换：

```text
jieba       词性高亮，当前最适合作为默认阅读高亮
ltp-pos     LTP legacy 词性高亮，实验性功能
ltp-ner     LTP legacy 实体高亮，实验性功能，小说文本效果不稳定
```

Cover Mode 不显示操作提示，以保持终端伪装效果：

```text
c             返回阅读界面
Up/Down       滚动一行
PageUp/Down   滚动一屏
End           返回最新输出
鼠标滚轮      浏览输出历史
```

## 高亮系统

v0.4 引入了统一的 `TextAnalyzer` 接口。不同 analyzer 负责把原始文本转换为 byte-offset annotation，TUI 只负责查询缓存并渲染颜色。

当前内置：

- `jieba`：基于 `jieba-rs` 的词性高亮，轻量、稳定、启动快。
- `ltp-pos`：基于 LTP legacy 的词性高亮。
- `ltp-ner`：基于 LTP legacy 的命名实体识别高亮。

LTP legacy 对新闻文本表现相对正常，但对小说文本中的译名、引号、书名号、换行和口语动作不稳定，可能出现错误实体。当前保留它作为实验和对照，不建议作为默认阅读模式。

调试分析器：

```bash
cargo run --example analyzer_probe
cargo run --example ltp_news_probe
```

## Cover Mode

Cover Mode 使用模块化引擎持续生成终端活动。当前内置：

- `cargo`：模拟依赖下载和编译。
- `weblog`：模拟 Nginx 访问日志。
- `download`：模拟带动态进度条的文件下载。

引擎将模块输出转换为虚拟终端指令，再由 ratatui 绘制。日志保留逻辑行，显示时根据当前终端宽度重新排版，因此窗口和字体变化后仍能正确换行。

Cover Mode 的模块化伪活动设计参考了 [genact](https://github.com/svenstaro/genact)。项目使用和适配的素材、生成器与模块逻辑遵循 genact 的 MIT 许可证，详情见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) 和 `assets/genact/LICENSE`。

## 状态文件

程序会在项目目录下创建 `.reading/`：

```text
.reading/library.json
.reading/current-text.utf8.txt
.reading/annotations/
.reading/models/
```

- `library.json` 保存当前书籍、书目路径、阅读 offset、文件长度和最近打开时间。
- `current-text.utf8.txt` 是内部 UTF-8 缓存，使 GBK 文本也能按照 UTF-8 offset 读取。
- `annotations/` 保存不同书籍、不同 analyzer 的高亮缓存。
- `models/` 可放置本地 LTP legacy 模型文件；这些模型文件不提交到仓库。

这些文件是本地运行数据，不提交到仓库。

## 当前限制

- 只支持 `.txt`，暂不支持 EPUB。
- 分页索引在运行时构建，尚未持久化。
- LTP legacy NER 是实验功能，对小说文本效果不稳定。
- LTP 模型文件需要用户自行放在 `.reading/models/ltp/legacy/`。
- Cover Mode 当前只有三个活动模块。
- Cover 视觉行每次绘制时重新排版，尚未增加布局缓存。

## 开发验证

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

快速检查：

```bash
cargo check
```
