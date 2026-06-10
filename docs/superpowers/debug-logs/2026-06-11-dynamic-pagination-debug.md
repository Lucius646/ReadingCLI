# 2026-06-11 动态分页 debug 日志

## 背景

全屏 TUI 已经可以根据终端尺寸动态显示正文，但冷启动后向前翻页不可靠。最早的问题出现在“上一页”计算：程序只有当前页 offset，没有上一页 offset，因此需要从当前 offset 反推出上一页的起点。

## 初始方案：二分反推上一页

最早的思路是：

1. 从当前 offset 往前读取一段候选文本。
2. 找出候选文本中的合法 UTF-8 byte offset。
3. 对这些 offset 做二分。
4. 对每个候选 start 调用 `layout_page`。
5. 寻找一个 start，使得 `layout_page(start).end_offset` 尽量接近当前 offset。

这个方案的问题是它依赖一个隐含假设：

```text
start_offset 越靠后，layout_page(start_offset).end_offset 也越靠后
```

这个假设在真实文本中不够稳。自动换行、手动换行、中文宽字符、拉丁扩展字符、终端宽度变化都会影响一页能容纳多少内容。继续修二分会让逻辑越来越复杂，而且很难保证 `p` 和 `n` 互逆。

## 设计调整：运行时 PageIndex

后来决定放弃二分反推，改成运行时构建页面索引。

核心思路：

```text
进入 TUI 时读取当前终端 columns / rows
从 offset 0 开始调用 layout_page
记录每一页的起始 byte offset
n / p 只移动 current_page_index
current_offset 从 page_starts[current_page_index] 取得
```

这样 `p` 和 `n` 不再临时计算文本边界，而是使用同一份 `PageIndex`。

## 新问题：看起来像 PageIndex 跳页

接入 `PageIndex` 后，用户观察到 offset 序列类似：

```text
0 -> 2853 -> 13618
```

这看起来像分页算法跳过了大量内容。

最初排查过两个方向：

1. `PageIndex::build()` 和 `draw()` 使用的候选区大小不一致。
2. UTF-8、`\r\n`、终端显示宽度和 `char_display_width` 的估算不一致。

这些方向都没有解释主要现象。

## 关键证据

临时写出 `.reading/page-index-debug.log` 后，发现索引前几项是：

```text
[0, 1015, 2853, 8064, 13618, ...]
```

这说明 `PageIndex` 本身没有直接生成 `0 -> 2853 -> 13618`。真实索引中间还有：

```text
1015
8064
```

也就是说，问题不是页面索引直接跳页，而是按键处理时跳过了索引项。

## 根因

`crossterm` 在当前终端环境中可能为一次按键产生多个 key event，例如：

```text
Press
Release
```

旧代码只检查：

```rust
key_event.code
```

没有检查：

```rust
key_event.kind
```

所以一次按 `n` 可能被处理两次：

```text
page 0 -> page 1 -> page 2
```

用户看到的 offset 就变成：

```text
0 -> 2853
```

而不是：

```text
0 -> 1015 -> 2853
```

## 修正

只处理 `KeyEventKind::Press`：

```rust
if key_event.kind != KeyEventKind::Press {
    continue;
}
```

这样 `Release` 等事件不会触发翻页。

## 结果

加入 `KeyEventKind::Press` 过滤后，`n/p` 翻页恢复稳定。PageIndex 的方向被验证为可行。

## 学习点

这次 debug 的关键不是算法本身，而是分层验证：

```text
TextSource 负责文件 offset 读取
layout_page 负责单页布局
PageIndex 负责页起点索引
TUI 负责绘制和输入事件
```

现象像“分页算法错了”，但根因实际在 TUI 输入事件被重复消费。以后遇到类似问题，应先暴露状态，例如显示 `offset/file_len`、写出 `page_starts`，再逐层排除。

## 后续

1. 清理废弃的二分反推代码。
2. 删除临时 debug 输出。
3. 在状态栏中显示当前页和总页数。
