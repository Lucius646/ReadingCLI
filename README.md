# ReadingCLI

ReadingCLI 是一个用 Rust 编写的终端小说阅读器。当前 v0.1 版本聚焦本地 `.txt` 阅读：打开文本文件后进入全屏终端界面，按终端大小动态分页，并保存阅读进度。

## 功能

- 支持本地 `.txt` 文件
- UTF-8 优先解码，失败后使用 GBK 解码
- 全屏终端阅读模式
- 根据当前终端宽高动态分页
- 终端大小变化后重建分页索引
- `n` / `p` / `q` 按键操作
- 保存当前阅读 offset，下次打开同一本书时恢复进度
- 状态栏显示当前页、总页数、offset 和阅读百分比

## 使用

准备 Rust 环境后，在项目目录运行：

```bash
cargo run -- open novel.txt
```

进入阅读界面后：

```text
n  下一页
p  上一页
q  退出并保存进度
```

## 状态文件

程序会在项目目录下创建 `.reading/`，用于保存本地运行状态：

```text
.reading/current-book.json
.reading/current-text.utf8.txt
```

其中：

- `current-book.json` 保存当前书籍路径和阅读 offset。
- `current-text.utf8.txt` 是内部 UTF-8 缓存文件，用于让 GBK 文本也能按 UTF-8 offset 读取。

这些文件是本地运行数据，不需要提交到仓库。

## 当前限制

- 暂时只支持单本书进度，不提供书架管理。
- 暂时只支持 `.txt`，不支持 EPUB。
- 分页索引在每次进入阅读器时运行时构建，暂不持久化。
- 终端显示宽度使用 `unicode-width` 估算，真实终端渲染仍可能存在少量差异。

## 开发验证

常用检查命令：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

如果只是快速确认能否编译：

```bash
cargo check
```
