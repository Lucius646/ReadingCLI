# 2026-06-24 Jieba 词性高亮渲染 debug 日志

## 背景

`jieba-highlight` 分支接入了 `jieba-rs`，用词性标注生成阅读高亮：

```text
txt 原文
  -> TextSource 统一成 UTF-8 文本
  -> JiebaAnalyzer 生成 byte offset annotation
  -> AnnotationStore 持久化到 .reading/annotations/*.json
  -> render_highlighted_page 把当前页转成 ratatui Line / Span
  -> TUI 渲染正文
```

这里的高亮本身不会改变原文内容，也不会改变 UTF-8 byte offset。`Span` 只是给某段字符串附加颜色样式。

## 用户观察到的现象

用户在 `novel.txt` 中观察到：

1. 部分文字像是没有渲染出来，正文上下文不连续。
2. 第三页的文字会残留在第一页，第一页文字也可能残留到后几页。
3. 文字碎片散落在屏幕不同位置。
4. 首次启动和翻页有一定卡顿。

这些现象不像是 jieba 分词错误。分词错误通常只会导致“颜色不准”或“词被切开”，不应该导致文本跳页、散落或残留。

## 当前渲染逻辑

启动或切书时：

```text
TextSource 读取整本书
JiebaAnalyzer 对全文做词性标注
AnnotationStore 保存标注缓存
```

这个部分不是实时执行的。只要 annotation 缓存存在且源文件长度一致，就会复用缓存。

每次阅读页重绘时：

```text
当前 current_offset
  -> TextSource 从 offset 往后读候选文本
  -> layout_page 计算当前页 end_offset
  -> AnnotationStore 查询当前页范围内的 annotation
  -> render_highlighted_page 实时生成 Vec<Line>
  -> ratatui Paragraph 渲染
```

所以：

```text
jieba 分析：静态缓存
页面排版和 Span 渲染：实时执行
```

## 初始误判

一开始怀疑两个方向：

1. 每个字符都生成一个独立 `Span`，导致 ratatui 渲染压力过大。
2. `Line` 内嵌 `\n`，导致 ratatui 实际渲染行数多于分页器计算的行数，从而裁掉底部文字。

这两个判断都有一定价值：

- 单字符 `Span` 的确会带来性能问题。
- `Line` 内嵌 `\n` 的确是不合理的富文本结构。

但它们不能完全解释“文字碎片散落”和“上一页残留到下一页”的现象。

## 关键假设：控制字符进入 Span

根据截图中“文字散落”的形态，新的怀疑点是：

```text
原文里可能包含终端控制字符，尤其是 Windows 文本常见的 \r\n。
```

其中：

```text
\n = 换行
\r = carriage return，回到当前行开头
```

之前的 `render_highlighted_page` 只特殊处理了 `\n`，没有处理 `\r`。

如果 `\r` 被当成普通字符放进 ratatui `Span`，终端/ratatui 的渲染行为可能变得异常：

```text
文字覆盖
光标位置错乱
旧页残留
文本像是没有连续渲染
```

## 测试验证

为了验证这个假设，先写了两个失败测试：

```rust
highlighted_render_does_not_embed_carriage_return_inside_ratatui_lines
highlighted_render_treats_crlf_as_a_single_line_break
```

测试目标：

1. `\r` 不应该出现在 ratatui `Span` 的内容里。
2. `\r\n` 应该被视为一个普通换行，而不是渲染成 `"第一行\r"`。

修复前测试失败，关键失败信息：

```text
left: ["第一行\r", "第二行"]
right: ["第一行", "第二行"]
```

这证明了根因之一：

```text
render_highlighted_page 确实把 \r 当成普通文本塞进了 Span。
```

## 修复

最小修复是在 `render_highlighted_page` 中显式处理 `\r`：

```rust
if ch == '\r' {
    end_offset = char_end;
    continue;
}
```

含义：

```text
消费 \r 对应的 byte offset
但不渲染它
不占终端列宽
不放进 Span
```

这样 `\r\n` 会变成：

```text
\r：被消费但不显示
\n：触发正常换行
```

也就是一个正常的 Windows 换行。

## 同时修复的性能问题

为了减少翻页卡顿，还把连续相同样式的字符合并成一个 `Span`。

修复前：

```text
a -> Span
b -> Span
c -> Span
```

修复后：

```text
abc -> Span
```

新增测试：

```rust
highlighted_render_merges_adjacent_text_with_same_style
```

这个修复不会改变显示结果，但能显著减少每页的 `Span` 数量。

## 验证结果

高亮测试：

```bash
cargo test --test highlight_tests
```

结果：

```text
9 passed
```

格式和静态检查：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

均通过。

用户重新运行后确认：

```text
没有 bug 了。
```

## 学习点

这次 debug 的关键不是 NLP，而是 TUI 富文本渲染边界。

高亮链路里有几个重要不变量：

```text
1. 高亮不能改变原文 byte offset。
2. ratatui Line 里不应该嵌入换行控制字符。
3. 原文控制字符必须在进入 Span 前被规范化。
4. end_offset 要表示“已经消费到原文哪里”，即使某些控制字符不显示，也要推进 offset。
```

一个很重要的判断：

```text
分词错误通常只会影响颜色，不会让文本残留或散落。
文本残留/散落更应该怀疑渲染层、控制字符、终端 buffer 或 TUI 布局。
```

## 后续可优化项

1. 将 `\t` 规范化为固定空格宽度，避免制表符导致列宽不一致。
2. 给 `render_highlighted_page` 增加更强的不变量测试：
   - 每一行显示宽度不超过终端宽度。
   - `HighlightedPage.end_offset` 和 `layout_page.end_offset` 在同输入下保持一致。
3. 首次启动时显示 “building highlight cache...” 之类的 loading 提示。
4. 把 annotation JSON 从 pretty JSON 改成 compact JSON，减少缓存体积和写入时间。
