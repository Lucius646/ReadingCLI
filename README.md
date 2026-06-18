# ReadingCLI

ReadingCLI 是一个用 Rust 编写的终端小说阅读器。v0.3 在动态分页、多书目进度和全屏 TUI 的基础上，加入了动态 Cover Mode，可在阅读界面与伪终端活动之间快速切换。

## v0.3 功能简述

- 支持无参数启动，默认进入全屏首页。
- 保留 `cargo run -- open <txt-path>` 直接打开本地小说。
- 首页提供 Continue、Open New Book、Select 和 Quit。
- 支持 UTF-8，并在解码失败时回退到 GBK。
- 根据终端宽高动态分页，resize 后自动重建分页索引。
- 使用书架保存多本书的绝对路径、阅读 offset 和最近打开时间。
- Cover Mode 动态模拟 Cargo、Web 访问日志和文件下载活动。
- Cover 输出支持 Unicode 宽度换行、窗口 resize 重排和历史滚动。

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

Cover Mode 不显示操作提示，以保持终端伪装效果：

```text
c             返回阅读界面
Up/Down       滚动一行
PageUp/Down   滚动一屏
End           返回最新输出
鼠标滚轮      浏览输出历史
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
```

- `library.json` 保存当前书籍、书目路径、阅读 offset、文件长度和最近打开时间。
- `current-text.utf8.txt` 是内部 UTF-8 缓存，使 GBK 文本也能按照 UTF-8 offset 读取。

这些文件是本地运行数据，不提交到仓库。

## 当前限制

- 只支持 `.txt`，暂不支持 EPUB。
- 分页索引在运行时构建，尚未持久化。
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
