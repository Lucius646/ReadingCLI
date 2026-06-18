# genact 项目源码分析

本文分析本地参考项目：

```text
E:\LuciusProject\genact-reference
```

上游项目：

```text
https://github.com/svenstaro/genact
```

`genact` 的定位是一个 “nonsense activity generator”，也就是生成看起来像电脑正在忙碌的终端输出。它不会真的执行这些任务，而是通过多个伪装模块模拟下载、编译、Web 日志、Docker 构建等场景。

## 1. 项目目标

`genact` 的核心目标不是完成真实工作，而是制造可信的终端活动：

```text
打开终端
运行 genact
随机进入一个伪装模块
输出看起来像真实命令的日志
模块结束后继续随机运行下一个模块
```

它和 ReadingCLI 的 Cover Mode 目标非常接近。区别是：

- `genact` 是独立 CLI，直接控制 stdout。
- ReadingCLI 是 ratatui TUI，需要在自己的界面里渲染假日志。

所以 ReadingCLI 不适合直接照搬 `genact` 的输出层，但非常适合借鉴它的模块结构和日志生成思路。

## 2. 顶层结构

主要源码位于：

```text
src/
├── args.rs
├── data.rs
├── generators.rs
├── io.rs
├── lib.rs
├── main.rs
└── modules/
    ├── mod.rs
    ├── cargo.rs
    ├── weblog.rs
    ├── download.rs
    └── ...
```

各文件职责：

- `main.rs`：程序入口，解析命令行参数，处理补全/manpage/list modules，然后调用 `genact::run(...)`。
- `args.rs`：定义 `AppConfig`，使用 clap 解析 CLI 参数。
- `lib.rs`：主运行循环，随机选择模块并执行。
- `modules/mod.rs`：定义 `Module` trait，并注册所有模块。
- `modules/*.rs`：每个伪装场景的具体实现。
- `io.rs`：封装打印、延迟、逐字打印、清行、移动光标等终端输出行为。
- `generators.rs`：封装随机字符串、随机路径、随机版本号等工具。
- `data.rs`：加载 `data/` 里的静态素材，例如包名、文件名、系统日志片段等。

## 3. CLI 配置层

`src/args.rs` 里定义了 `AppConfig`。

主要字段：

```rust
pub struct AppConfig {
    pub list_modules_and_exit: bool,
    pub modules: Vec<String>,
    pub speed_factor: f32,
    pub instant_print_lines: u32,
    pub inhibit: bool,
    pub exit_after_time: Option<instant::Duration>,
    pub exit_after_modules: Option<u32>,
    pub print_completions: Option<clap_complete::Shell>,
    pub print_manpage: bool,
}
```

几个重要设计：

- `modules`：指定只运行哪些模块。如果为空，默认启用全部模块。
- `speed_factor`：全局速度倍率，影响 sleep 时长。
- `instant_print_lines`：前 N 行不等待，适合快速启动输出。
- `exit_after_time` / `exit_after_modules`：允许限制运行时长或模块数量。
- wasm 和非 wasm 平台使用不同配置结构，说明项目兼容 WebAssembly。

对 ReadingCLI 的启发：

- Cover Mode 可以先不暴露 CLI 参数，但内部也应该有类似 `CoverConfig`。
- 第一版可以只包含 `scene`、`speed_factor`、`max_lines`。

## 4. 主循环

`src/lib.rs` 的 `run(appconfig)` 是核心调度器。

逻辑简化如下：

```text
根据 appconfig.modules 从 ALL_MODULES 里取出可运行模块
loop:
    随机选择一个模块
    可选：保持系统不休眠
    执行 choice.run(&appconfig).await
    模块运行数 +1
    如果达到退出条件，则退出
```

关键点：

- 它不是每一行都随机生成，而是先随机选择一个完整模块。
- 一个模块内部有自己的节奏、循环和输出格式。
- 模块跑完以后，主循环再选择下一个模块。

对 ReadingCLI 的启发：

```text
CoverState 不应该只保存 Vec<String>
还应该保存当前 Scene
Scene 自己决定下一次输出什么
```

也就是说，Cover Mode 更像一个小型状态机，而不是静态字符串列表。

## 5. 模块抽象

`src/modules/mod.rs` 定义了统一的 `Module` trait：

```rust
#[async_trait(?Send)]
pub trait Module: Sync + Send {
    fn name(&self) -> &'static str;
    fn signature(&self) -> String;
    async fn run(&self, app_config: &AppConfig);
}
```

每个模块必须提供：

- `name()`：模块名，例如 `"cargo"`。
- `signature()`：看起来像真实命令的签名，例如 `"cargo run"`。
- `run()`：真正生成伪输出的逻辑。

模块注册表：

```rust
pub static ALL_MODULES: LazyLock<BTreeMap<&'static str, Box<dyn Module + Send + 'static>>>
```

注册了很多模块：

```text
ansible
bootlog
botnet
bruteforce
cargo
cc
composer
cryptomining
docker_build
docker_image_rm
download
julia
kernel_compile
memdump
mkinitcpio
rkhunter
simcity
terraform
uv
weblog
wpt
```

对 ReadingCLI 的启发：

第一版不一定需要 trait。因为 ReadingCLI 当前只需要一个 Cover 场景。可以先用：

```rust
enum CoverScene {
    Cargo,
}
```

但如果后续要支持 `Cargo / Weblog / Download / Npm / Server` 多模板，就可以升级为 trait 或 enum 分发。

## 6. 输出层

`src/io.rs` 是 `genact` 和 ReadingCLI 差异最大的地方。

`genact` 使用命令式输出：

```rust
print(...).await;
newline().await;
csleep(...).await;
erase_line().await;
cursor_up(...).await;
```

它直接向 stdout 写内容：

```rust
print!("{}", s.into());
stdout().flush().unwrap();
```

并用 ANSI 控制序列实现清行、移动光标：

```rust
print("\x1b[2K\x1b[0G").await;
print(format!("\x1b[{n}A")).await;
```

这套方式适合独立 CLI，但不适合 ReadingCLI 当前的 ratatui 架构。

ReadingCLI 当前是：

```text
状态变化
  -> draw(...)
  -> ratatui 根据状态重绘屏幕
```

所以 ReadingCLI 的 Cover Mode 应该这样做：

```text
CoverState 生成新的日志行
日志行保存到 Vec<String>
ratatui 每次 draw 时渲染 Vec<String>
```

不要在 Cover 模块里直接 `print!`。

## 7. 延迟机制

`genact` 的延迟函数是 `csleep(length)`。

它会考虑：

- 全局 `speed_factor`
- `instant_print_lines`
- wasm 和非 wasm 平台差异

简化逻辑：

```text
如果还在 instant_print_lines 范围内：
    不等待
否则：
    sleep(length / speed_factor)
```

对 ReadingCLI 的启发：

ReadingCLI 不应该在 Cover Mode 里直接 sleep。因为 TUI 主循环还要响应按键。

更适合的做法是：

```rust
CoverState {
    next_update_at: Instant,
}
```

主循环定时醒来：

```text
每 100ms 或 200ms poll 一次按键
如果当前是 Cover Mode
    如果 now >= next_update_at
        追加一批日志
        设置下一次更新时间
```

这样按 `c` 返回阅读时不会被 sleep 卡住。

## 8. 代表模块：cargo

`src/modules/cargo.rs` 模拟 Rust 包下载和编译。

核心逻辑：

```text
随机选择 10 到 100 个包名
为每个包生成随机版本号
第一轮输出 Downloading
第二轮输出 Compiling
每行之间随机等待 100 到 2000ms
最后输出 Finished release [optimized] target(s) in X secs
```

伪输出类似：

```text
 Downloading ratatui v0.29.0
 Downloading crossterm v0.28.1
   Compiling libc v0.2.186
   Compiling reading_cli v0.2.0
    Finished release [optimized] target(s) in 12.42 secs
```

它的真实感来自：

- 使用真实风格的 Cargo 阶段名。
- 包名来自数据文件，不是硬编码几条。
- 版本号使用随机分布生成。
- 行间延迟不固定。
- 最后的耗时来自真实 elapsed time。

对 ReadingCLI 的启发：

第一版 Cover Mode 最适合从 cargo 场景开始。因为 ReadingCLI 本身就是 Rust 项目，这个伪装最自然。

## 9. 代表模块：weblog

`src/modules/weblog.rs` 模拟 `tail -f /var/log/nginx/access.log`。

核心逻辑：

```text
生成 50 到 200 行访问日志
随机 IPv4 / IPv6
使用当前时间
生成随机路径、HTTP code、响应大小、User-Agent
支持 burst mode：偶尔进入快速输出模式
```

它的真实感来自：

- 日志格式接近 Nginx access log。
- 有正常输出和 burst 输出。
- HTTP code、响应大小、路径、User-Agent 都随机。
- 节奏不是均匀的。

对 ReadingCLI 的启发：

`weblog` 很适合作为第二个 Cover 场景。它比 cargo 更适合长时间停留，因为 server log 本来就是持续输出。

## 10. 代表模块：download

`src/modules/download.rs` 模拟下载文件。

它比 cargo/weblog 更复杂，因为它会：

- 根据终端宽度计算进度条宽度。
- 逐步更新当前行。
- 使用 `erase_line()` 清除旧进度。
- 显示百分比、下载量、速度、ETA。

伪输出大概是：

```text
archive.tar.gz  42% [===========>             ] 12MiB  8.4MiB/s eta 3s
```

对 ReadingCLI 的启发：

download 场景很有视觉效果，但第一版不建议做。原因是：

- 它需要处理单行刷新。
- ratatui 下可以实现，但会增加 CoverState 的复杂度。
- 第一版先做追加日志更稳。

## 11. 数据素材

`genact` 的真实感很大程度来自 `data/` 目录。

例如：

```text
packages.txt
cfiles.txt
bootlog.txt
docker_packages.txt
terraform_aws_resources.txt
web_apis.txt
```

这些数据文件让模块输出看起来丰富，不像只有几条硬编码模板。

对 ReadingCLI 的启发：

第一版不需要引入大型数据文件，但可以维护几个小数组：

```rust
const CARGO_PACKAGES: &[&str] = &[
    "anyhow",
    "serde",
    "ratatui",
    "crossterm",
    "unicode-width",
    "clap",
];
```

后续如果 Cover Mode 成为核心特色，再考虑把素材拆到单独文件。

## 12. 随机工具

`src/generators.rs` 封装了很多随机生成逻辑：

- 随机字符串
- 随机十六进制字符串
- 随机文件名
- 随机路径
- 随机包版本号

其中版本号不是简单均匀随机，而是用了 `rand_distr` 的分布：

```rust
ChiSquared
Exp
```

这样生成出来的版本号更像真实世界里的版本号分布。

对 ReadingCLI 的启发：

第一版不需要这么严谨。可以先用简单范围：

```text
major: 0..3
minor: 1..30
patch: 0..20
```

等功能稳定后，再考虑更真实的随机分布。

## 13. 许可证

`genact` 使用 MIT License。

许可证要求：如果复制、修改、分发其代码或 substantial portions，需要保留版权声明和许可声明。

当前 ReadingCLI 如果只是借鉴设计，不复制源码，可以在 README 或设计文档中写：

```text
Cover Mode is inspired by genact:
https://github.com/svenstaro/genact
```

如果后续复制了具体代码、数据文件或较大实现片段，建议新增：

```text
THIRD_PARTY_NOTICES.md
```

并记录：

```text
genact
https://github.com/svenstaro/genact
License: MIT
Copyright 2020 Sven-Hendrik Haase
```

## 14. ReadingCLI 应该借鉴什么

建议借鉴：

1. 模块化 Cover 场景。
2. 每个场景自己控制输出阶段。
3. 输出节奏不均匀，避免机械刷屏。
4. 使用少量随机素材提升真实感。
5. 日志缓冲区只保留最近 N 行。
6. 第一版先做 cargo 场景，后续扩展 weblog/download。

不建议照搬：

1. 不直接向 stdout 打印。
2. 不在 Cover 逻辑里 sleep 阻塞。
3. 不直接引入 async/tokio。
4. 不一开始就做全部模块。
5. 不复制大型 `data/` 文件。

## 15. 推荐的 ReadingCLI Cover Mode 架构

第一版可以新增：

```text
src/cover.rs
```

核心结构：

```rust
pub struct CoverState {
    lines: Vec<String>,
    scene: CoverScene,
    next_update_at: Instant,
}

enum CoverScene {
    Cargo,
}
```

核心接口：

```rust
impl CoverState {
    pub fn new() -> Self;
    pub fn update_if_needed(&mut self);
    pub fn lines(&self) -> &[String];
}
```

`tui.rs` 只负责：

```text
进入 Cover Mode
每轮 loop 调用 cover_state.update_if_needed()
draw_cover 读取 cover_state.lines()
c 返回 Reading
q 退出
```

第一版 Cargo 场景建议阶段：

```text
StartCommand
Downloading
Compiling
Finished
Idle
```

伪输出：

```text
$ cargo build --release
   Downloading unicode-width v0.2.2
   Downloading ratatui v0.29.0
   Compiling crossterm v0.28.1
   Compiling reading_cli v0.2.0
    Finished release [optimized] target(s) in 8.42s

$ cargo check
    Checking reading_cli v0.2.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```

## 16. 第一版实现边界

建议第一版只做：

- 一个 `Cargo` Cover 场景。
- 动态追加日志。
- 不阻塞按键。
- 只保留最近若干行。
- `c` 进入/退出 Cover。
- `q` 退出程序。
- README 或 roadmap 标注 inspired by genact。

暂时不做：

- 多 Cover 场景选择。
- 进度条原地刷新。
- async/tokio。
- 外部运行 genact。
- 复制 genact 数据文件。

这个边界比较适合当前 ReadingCLI：既能拥有 genact 风格的动态伪装，又不会把项目复杂度一下子推高。
