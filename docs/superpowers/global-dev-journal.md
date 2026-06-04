# Development Learning Journal

## 2026-04-28 - Brainstorming starts: define the project before coding

Phase: brainstorming.

Current problem: the project directory `E:\LuciusProject\ReadingCLI` has no source code and is not yet a git repository. The user wants to build a terminal reading CLI while also learning engineering practice, and explicitly does not want a large dump of code or framework choices.

What happened: I inspected the current directory with `Get-ChildItem -Force`, checked `git status --short`, and checked recent commits with `git log --oneline -5`. The git commands confirmed there is no repository yet. This means the right first engineering move is not to implement features, but to define a small product goal and learning path.

Decision: use a design-first workflow. We will clarify the first useful version of the CLI, then choose a small architecture and learning path before writing code.

What to learn from this step: engineering starts by reducing uncertainty. Before files and commands matter, a developer should know what problem the first version solves, how small the first milestone can be, and how success will be verified.

Likely next step: ask one clarifying question about what kind of reading workflow the CLI should support first.

## 2026-04-28 - Product direction: start with reading, then add library management

Phase: brainstorming.

Current problem: the final goal is a full-featured terminal novel reader, but building that directly would mix too many unknowns at once: terminal UI, file parsing, state persistence, library management, search, configuration, and packaging.

What happened: the user chose a route that starts with option A, a local text reading experience, then grows toward option B, reading list and library management.

Decision: treat the project as a staged product. The first useful version should prove that a user can read a local novel in the terminal and resume progress. Later versions can organize many books, metadata, notes, and advanced reading behavior.

What to learn from this step: a high final goal is fine, but engineering progress depends on milestones. A good milestone is not "a fragment of code"; it is a small working product slice that can be used and tested.

Likely next step: compare possible implementation routes and choose the first architecture small enough for a beginner but strong enough to grow.

## 2026-04-28 - Clarifying engineering terms before choosing a route

Phase: brainstorming.

Current problem: terms like "minimal CLI", "TUI", and "library-first" are meaningful to experienced developers, but they are not useful to a beginner unless translated into concrete user experience and engineering cost.

What happened: the user clarified that they want the final product to feel like a TUI novel reader, but also recognizes they do not currently have the engineering ability to build that directly. The user is interested in Rust, Go, and Python, and needs guidance on language choice.

Decision: explain the routes using concrete examples instead of abstract labels, then recommend a staged path: build a simple command-line reader first, while choosing tools that can later grow into a full-screen TUI.

What to learn from this step: engineering vocabulary is only useful when it helps choose the next action. A developer should be able to translate terms like "CLI" or "TUI" into what the user sees, what code must exist, and what risks the team takes on.

Likely next step: choose the implementation language based on learning value, ecosystem fit, and how smoothly it supports the future TUI goal.

## 2026-04-28 - Language choice: use Rust despite higher learning cost

Phase: brainstorming.

Current problem: Go would be the lower-friction choice for a beginner-friendly CLI/TUI, but the user is more interested in learning Rust. Choosing a language is not only a technical decision; it also affects motivation and what kind of engineering lessons the project teaches.

What happened: the user accepted the staged route but expressed a clear preference for Rust.

Decision: use Rust as the project language, while deliberately keeping the first milestone small. The first version should avoid full TUI complexity and focus on a simple reader loop, so the user can learn Rust project structure, ownership basics, error handling, file I/O, and tests without being overwhelmed by terminal UI rendering.

What to learn from this step: the "best" technology is not always the easiest one. A good engineering choice balances product fit, learning goal, ecosystem support, and risk. Since the user's goal includes learning Rust, choosing Rust is reasonable if the project scope is reduced accordingly.

Likely next step: present a Rust-based v0.1 design and ask the user to approve or adjust it before writing any code.

## 2026-04-28 - Input files: local txt novels, likely GBK

Phase: brainstorming.

Current problem: the first version needs to know what kind of text files it should handle. Chinese novels downloaded from the internet are often `.txt` files encoded as GBK rather than UTF-8, so a naive "read file as UTF-8 string" approach may fail or display garbled text.

What happened: the user confirmed v0.1 only needs to support `.txt`, and the likely source is downloaded Chinese novel files that may use GBK.

Decision: keep the file format scope narrow, but include character encoding handling in the design. The reader should first support `.txt`, and its file loading layer should try UTF-8 and GBK-compatible decoding instead of assuming every file is UTF-8.

What to learn from this step: real software has messy input. Narrowing file type to `.txt` simplifies the product, but encoding remains a real-world compatibility issue. Good design isolates this problem in a file-loading module so the rest of the program can work with normal Rust `String` values.

Likely next step: decide whether v0.1 pagination should be based on lines, paragraphs, or character count.

## 2026-04-28 - Page model: fixed text blocks with page-style navigation

Phase: brainstorming.

Current problem: a terminal novel reader ultimately needs dynamic layout based on terminal width and height, especially when the user changes font size. That is TUI-level behavior. For v0.1, implementing it directly would combine too many difficult pieces at once.

What happened: the user asked how terminal font scaling affects line-based pagination, then clarified and accepted a simpler v0.1 model: display a fixed-size block of text and navigate by page-like commands.

Decision: v0.1 will use fixed text blocks and page navigation. The program stores progress as a character position in the decoded text. `n` moves forward by one block, `p` moves backward by one block, and `q` saves and exits. Terminal font scaling is allowed, but v0.1 will not dynamically recalculate layout when the font or terminal size changes.

What to learn from this step: a product can intentionally accept a rough edge when it is named clearly. The important engineering move is to store progress in a form that can survive future improvements. Character position is a better long-term progress model than raw terminal page number.

Likely next step: present the v0.1 design for approval, including architecture, data flow, error handling, and testing scope.

## 2026-04-28 - v0.1 product boundary approved

Phase: brainstorming.

Current problem: before writing code, the first version needs a stable boundary. Without one, it is easy to keep adding features such as TUI, scrolling, book library management, search, bookmarks, and chapter detection before the simplest reader works.

What happened: the user approved the v0.1 boundary: Rust, local `.txt` files, UTF-8/GBK decoding, fixed-size text blocks, page-style navigation with `n`/`p`/`q`, progress saved by character position, and no full TUI yet.

Decision: freeze v0.1 product scope and move to internal structure design. The next design step is to divide the program into small responsibilities: command handling, text loading, text block navigation, progress storage, and interactive loop.

What to learn from this step: scope control is an engineering skill. A good first version is not the final product with missing polish; it is a complete small slice that can run, fail clearly, and teach the next design decision.

Likely next step: confirm the internal module structure before writing the design document.

## 2026-04-28 - Core model refinement: use reader state and events

Phase: brainstorming.

Current problem: the first internal structure described command handling, text loading, position, progress, and read loop as separate parts, but it did not explicitly name the central state model. Without a central state, navigation commands can become scattered across the program.

What happened: the user proposed a state-machine-style model: keep per-book metadata, load it when opening a book, then treat `n`, `p`, and `q` as state changes.

Decision: revise the design around a `ReaderState` plus explicit events. For v0.1, the state can remain simple: current book metadata, decoded text length, current character position, block size, and running/exiting status. Inputs such as `n`, `p`, and `q` become events that transition the state.

What to learn from this step: a state machine does not have to be complicated. At this scale, the important idea is to put "what can change" in one model, and make user actions update that model through clear rules. This improves testability and prepares the program for future TUI behavior.

Likely next step: present the revised v0.1 structure using metadata, reader state, and events.

## 2026-04-28 - 元数据策略：先保存当前文本，给未来书架留位置

阶段：brainstorming。

当前问题：v0.1 只打开一本本地 `.txt`，但最终产品会有书架管理。如果现在完全不考虑元数据目录，后面做书架时可能要迁移存储结构；如果现在直接做多书籍交互，又会扩大第一版范围。

发生了什么：用户提出在工作区里创建一个专门存储书籍元数据的文件夹，但 v0.1 暂时不处理多书籍交互。当前版本只维护“当前文本”的元数据；程序运行时把这些数据放入状态机，退出时写回元数据文件。

决策：v0.1 采用“单书元数据文件 + 可扩展元数据目录”的策略。运行时维护当前文本的数据结构，例如路径、标题、编码、总字符数、当前位置、块大小等；退出时把当前位置等信息保存到元数据文件。目录结构为后续书架管理预留空间，但不在 v0.1 实现书架命令。

这一学习点：工程里的“可扩展”不是提前实现所有功能，而是在不增加当前复杂度的前提下，避免把未来明显需要的边界封死。这里的边界是：阅读状态在内存里，持久化数据在元数据文件里，二者通过明确的加载和保存动作连接。

下一步：把 v0.1 设计正式收束为 `BookMetadata`、`ReaderState`、事件转移、元数据文件和阅读循环。

## 2026-04-28 - 调研结论：状态机不持有整本正文

阶段：brainstorming。

当前问题：在设计 `ReadingSession` 时，不能把解码后的完整 txt 文本直接放进状态机。小说文件可能很大，而状态机应该描述“当前读到哪里、是否继续运行、当前配置是什么”，不应该拥有大体积正文数据。

发生了什么：用户要求联网搜索开源方案如何处理小说/电子书文件。调研发现，成熟工具通常把阅读状态、元数据和正文访问分开：`less` 这类 pager 不需要先读完整文件；`epr` 保存每本书的阅读状态到配置文件，并在阅读时按章节读取 EPUB 内容；KOReader 使用 sidecar 元数据文件保存进度、设置、书签和笔记；Bookokrat 把书签、评论、缓存、日志和设置放在 XDG 风格的数据目录中。

决策：v0.1 设计应改成三层边界：`BookMetadata` 保存可持久化元数据；`ReadingSession` 只维护轻量运行状态；`TextSource` 负责从 txt 文件中提供当前要显示的文本块。即使 v0.1 为简单实现可能先采用较粗糙的文件读取策略，设计上也不能让状态机拥有整本正文。

这一学习点：工程设计里，“对象里能放什么”和“对象应该负责什么”不是一回事。把大文件正文放进状态机会让状态模型膨胀，也会让后续按块读取、缓存、编码处理和 TUI 分页变难。正确边界是让状态机只保存状态，让文本源负责 I/O。

下一步：在 v0.1 设计中明确 `TextSource`，并把当前页显示定义为 `TextSource.read_block(position, block_size)` 的结果。

## 2026-04-28 - 架构边界确认：CLI 不一次性拥有整本正文

阶段：brainstorming。

当前问题：v0.1 需要读取小说文本块，但不应该让 CLI 的状态机一次性持有整个 txt 内容。这样会让状态对象变重，也会把文件 I/O、编码处理和阅读状态混在一起。

发生了什么：用户确认分层方向正确，并明确“不应该把 content 一次加载进 cli”。

决策：冻结这个边界：`ReadingSession` 只保存轻量状态，例如当前元数据、当前位置、块大小和运行状态；`TextSource` 负责根据当前位置读取或提供当前文本块。后续实现可以先用简单方案，但接口和设计必须保持“状态机不拥有正文”的原则。

这一学习点：工程里真正重要的不是一开始就实现最高性能，而是把职责边界放对。只要边界正确，内部实现可以从简单版本演进到流式读取、缓存、索引或按窗口分页，而不需要推翻整个程序结构。

下一步：把 v0.1 设计写成中文设计文档，并在文档里明确这个边界。

## 2026-04-29 - TextSource 缓存：区分文件访问和界面滚动

阶段：brainstorming。

当前问题：用户已经接受 `TextSource` 使用临近文本块作为缓存区，但进一步追问：为什么记事本这类软件可以直接滚动，看起来不需要显式缓存。

发生了什么：讨论从“切页时是否读文件”推进到更底层的模型：文件访问、内存缓冲、操作系统缓存和界面渲染是不同层次。普通编辑器或记事本可能会把小文件整份加载到内存，也可能依赖操作系统页缓存、文本布局缓存和渲染窗口，只是这些缓存对用户不可见。

决策：v0.1 设计保留 `TextSource` 内部缓存概念。对外接口仍然是 `get_block(block_index)`；内部先查临近块缓存，未命中再通过索引和文件读取补充缓存。这样既不要求状态机持有全文，也不要求每次切页都实际访问磁盘。

这一学习点：用户看到的“滚动很顺滑”不等于程序没有缓存。很多缓存发生在应用内部、运行库、操作系统文件系统缓存、磁盘控制器甚至硬件层。工程设计需要把这些层次拆开，明确哪些缓存由我们控制，哪些由系统隐式提供。

下一步：在设计文档中把 `TextSource` 定义为“按需读取 + 小窗口缓存 + 可选索引”的组件。

## 2026-04-29 - TextSource 缓存窗口：当前块上下各十块

阶段：brainstorming。

当前问题：`TextSource` 需要一个具体缓存策略。缓存太小会导致连续翻页频繁补读；缓存太大又接近把正文整体放入内存，模糊了文本源和状态机的边界。

发生了什么：用户提出一本 26 万字左右的小说通常也只有几百 KB，可以把缓存块设计大一点，缓存当前块上下各十块。

决策：v0.1 采用 21 块缓存窗口：当前块、前 10 块、后 10 块。这个窗口属于 `TextSource` 内部缓存，不进入 `ReadingSession` 状态机。若每块约 1200 字，则缓存约 2.5 万字；对中文 GBK/UTF-8 文本来说仍然是很小的内存占用，同时能覆盖连续翻页的常见行为。

这一学习点：缓存大小是工程取舍，不是越小越“正确”。合理缓存应该服务真实使用场景，同时保持职责边界清楚。这里的边界是：缓存是 `TextSource` 的性能细节，不是阅读状态。

下一步：把该策略写入 v0.1 设计文档，并说明未来可以把缓存窗口做成配置项。

## 2026-04-29 - v0.1 设计文档落地

阶段：brainstorming。

当前问题：经过多轮讨论，v0.1 的产品边界和核心架构已经稳定，需要从对话沉淀为可 review、可执行的设计文档。

发生了什么：创建了 `docs/superpowers/specs/2026-04-29-reading-cli-v0.1-design.md`。文档记录了 Rust CLI、txt 支持、UTF-8/GBK、固定文本块、`n/p/q`、`BookMetadata`、`ReadingSession`、`TextSource`、块索引、上下各 10 块缓存、元数据文件和测试范围。

决策：设计文档作为下一阶段实现计划的输入。在用户 review 并确认前，不进入 Rust 项目初始化或代码实现。

这一学习点：设计文档不是形式主义，它把口头讨论变成稳定契约。对初学者尤其重要，因为后续写代码时可以不断回到文档，判断某个实现是否偏离了目标。

下一步：用户 review 设计文档；确认后进入 implementation plan 阶段，把设计拆成可执行任务。

## 2026-04-29 - 进入实现计划阶段

阶段：writing-plans。

当前问题：v0.1 设计已经被确认，但还不能直接开始写代码。设计说明“要做什么”，实现计划需要说明“按什么顺序做、每一步验证什么、哪些文件负责什么”。

发生了什么：创建了 `docs/superpowers/plans/2026-04-29-reading-cli-v0.1.md`。计划把实现拆成 Rust 项目初始化、CLI 解析、状态机、元数据、UTF-8 TextSource、GBK TextSource、编码检测、阅读循环、错误处理、README 等任务。

决策：采用测试驱动的顺序，先实现不依赖文件 I/O 的状态机，再处理元数据，最后进入最复杂的 `TextSource`。用户明确要求暂不 commit，因此计划中不包含提交步骤。

这一学习点：实现计划的价值是降低同时处理的问题数量。初学工程时，最常见的失败方式是把项目结构、业务逻辑、文件 I/O、编码、缓存、CLI 交互一次性混在一起。计划把这些问题拆开，每一步都能单独验证。

下一步：用户确认计划后，进入执行阶段，开始初始化 Rust 项目并按任务逐步实现。

## 2026-04-29 - 协作模式调整：用户亲手编码

阶段：writing-plans。

当前问题：原实现计划更像给代理或熟练工程师执行的任务清单，不够适合用户亲手编码和学习。用户明确表示希望自己动手实现，而不是由代理直接执行。

发生了什么：重写了 `docs/superpowers/plans/2026-04-29-reading-cli-v0.1.md`。新版计划改为教学式小步计划，从环境确认开始，逐步进入 Cargo 项目、模块结构、CLI、状态机、元数据、TextSource、缓存、GBK 和阅读循环。每个阶段都说明要创建什么文件、写什么内容、运行什么命令、期望看到什么输出，以及完成后应该把什么发给我。

决策：后续采用结对编程式协作。用户负责编写代码和运行命令；我负责拆任务、解释概念、review 代码、分析错误和决定下一步。不自动实现代码。

这一学习点：工程学习不能只看最终代码。亲手创建文件、运行命令、观察失败、理解错误并修正，才会建立真实的工程经验。计划需要适配学习者，而不是只追求最快完成。

下一步：用户从 Phase 0 开始，运行 Rust 和 Cargo 环境检查命令，并把输出发给我。

## 2026-04-29 - 终端约定改为 Git Bash

阶段：writing-plans。

当前问题：原计划使用了 PowerShell 命令，但用户实际习惯是在 VS Code 内置终端中使用 Git Bash。命令风格不一致会增加初学者的额外负担。

发生了什么：用户说明“终端”指 Git Bash。实现计划已重写为 Git Bash 版本，默认项目路径改为 `/e/LuciusProject/ReadingCLI`，命令改为 `pwd`、`ls`、`mkdir -p`、`touch`、`cargo run`、`cargo test`、`cargo check`。

决策：后续教学和操作说明默认使用 Git Bash 命令。只有明确需要 Windows/PowerShell 特性时才单独说明。

这一学习点：工程文档要贴合真实工作环境。对初学者来说，命令行环境差异本身就是认知成本；统一环境能减少无关错误。

下一步：用户从 Phase 0 开始，在 Git Bash 中运行环境检查命令。
