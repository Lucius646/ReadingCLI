# ReadingCLI Roadmap

## 总方向

ReadingCLI 的长期方向不是做一个普通终端阅读器，而是做成一个有开发者终端场景特色的阅读工具：

```text
基础阅读器
  ↓
ratatui TUI 重构
  ↓
Cover Mode
  ↓
TUI / 体验打磨
  ↓
NLP 语法分析高亮
  ↓
未来语义标注 / LLM 扩展
```

一句话目标：

> ReadingCLI 先做成一个有 Cover Mode 的开发终端阅读器，再引入 NLP 做自然语言语法高亮，最后预留 LLM 语义标注扩展。

## v0.1：基础阅读器

状态：已完成。

当前能力：

- 本地 `.txt` 阅读
- UTF-8 优先，GBK fallback
- 全屏终端阅读
- 根据终端大小动态分页
- resize 后重建分页索引
- `n` / `p` / `q` 翻页和退出
- 保存 offset 阅读进度
- 状态栏显示页数、offset 和百分比
- 基础 README 和 CI

## v0.2：ratatui TUI 重构

优先级：最高，Cover Mode 前置任务。

原因：

Cover Mode 本质上是多界面渲染。如果继续用 crossterm 手写坐标、清屏和 `write!`，后续 `Reading / Cover / Search / Config` 会让 `tui.rs` 快速变复杂。ratatui 可以把界面拆成布局和 widget，为后续模式切换做准备。

目标：

- 保持现有阅读功能不变。
- 保持 `n` / `p` / `q` 交互不变。
- 保持 PageIndex、动态分页、resize 重建逻辑不变。
- 只把渲染层从手写 crossterm 迁移到 ratatui。

建议边界：

- 新增 `ratatui` 依赖。
- 保留 `crossterm` 负责输入事件和终端 backend。
- 使用 `Terminal<CrosstermBackend<Stdout>>`。
- 使用 `Layout` 垂直拆分正文区域和状态栏。
- 正文使用 `Paragraph` 渲染。
- 状态栏使用 `Paragraph` 渲染。

暂时不做：

- Cover Mode
- 搜索
- 高亮
- 主题配置
- 弹窗

## v0.3：Cover Mode

目标：

让 ReadingCLI 具备开发终端场景特色：在 VS Code / Git Bash 中阅读时，可以按老板键立刻切换成类似开发输出的伪装界面，再按一次恢复阅读。

核心功能：

- `Reading Mode / Cover Mode` 状态切换
- 老板键绑定
- Fake cargo test 输出
- Fake cargo build 输出
- Fake server log 输出
- 恢复阅读页和 offset

架构方向：

```rust
enum AppMode {
    Reading,
    Cover,
}
```

后续可以扩展为：

```rust
enum AppMode {
    Reading,
    Cover,
    Search,
    HighlightConfig,
}
```

目标不是恶搞，而是形成项目特色，同时推动状态模型升级：

```text
按键 -> 更新 AppState -> 根据 AppMode 渲染不同界面
```

## v0.4：TUI / 体验打磨

目标：

在 Cover Mode 基础上整理 TUI 状态、按键和界面体验。

可能内容：

- 统一 `AppState`
- 统一按键分发
- 状态栏更清晰
- 帮助提示
- 错误信息展示
- 简单配置入口
- 更好的退出和恢复体验

这一阶段不追求大功能，而是让后续搜索和高亮不会继续堆在 `tui.rs` 里。

## v0.5：NLP 语法分析高亮

目标：

把 ReadingCLI 做成“自然语言版代码高亮”阅读器。

第一版不要做深层语义理解，先做浅层结构高亮：

- 分词
- 词性
- 代词
- 连接词
- 名词 / 动词
- 括号 / 引号
- 章节标题
- 搜索词

可能技术：

- `jieba-rs`
- 词典规则
- 正则规则
- 高亮配置
- ratatui `Line` / `Span` / `Style`

重点：

不要一开始就做“理解整本书”。先让文本更容易扫读。

## v0.6：语义标注与 LLM 扩展预留

目标：

引入结构化标注层，为未来 NLP / LLM 扩展预留接口。

可能文件：

```text
.reading/annotations.json
```

可能标注：

- 人物
- 概念
- 情绪
- 转折
- 重要句
- 意象
- 章节摘要

这一阶段不是当前主战场。当前主战场是：

```text
ratatui TUI 重构
Cover Mode
NLP 语法高亮
```

## 当前下一步

下一步从 `v0.2 ratatui TUI 重构` 开始。

第一版 ratatui 迁移只做等价替换：

```text
现有功能不变
渲染方式换成 ratatui
不同时加入 Cover Mode
```
