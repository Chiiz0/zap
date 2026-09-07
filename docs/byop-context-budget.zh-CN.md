# BYOP 工具输出、上下文预算与自动摘要兜底

本次故障由一条无输出数量上限的 `docker logs --since 2h` 触发：对话导出中该结果有 25,214 行、约 4.36 MB。上一轮成功请求的 10,610 个输入 token 不包含这批新结果，不能用来判断下一轮是否安全。

## 发送前保护

入口为 `app/src/ai/agent_providers/request_budget.rs`，在 `build_chat_request` 完成历史、当前输入和工具定义的组装后执行。

- 单个工具结果最多 32 KiB，所有工具结果合计最多 128 KiB；预算按序列化后的字符串计数，包含 JSON 转义开销。
- 模型窗口与 profile 上限取较小的有效值；未知窗口暂按 200,000 token 计算，但仍执行工具结果硬上限。
- 文本按每 token 两个序列化 UTF-8 字节估算，预留输出及协议空间后，用剩余预算进一步减少工具结果。优先保留最近结果，为旧结果保留省略提示。
- 通常保留结果的头尾。JSON 的 `output` 字段可单独裁剪时，保留状态、退出码、命令 ID 等其他字段；其他结构返回明确标注为部分内容的 JSON 预览。
- 不删除或重排工具调用和结果，不修改工具参数、调用者元数据及本地原始历史。完整输出仍可在原会话中查看。
- 省略提示要求模型按范围读取或搜索缺失内容，不建议重新执行有副作用的命令。
- 非工具正文、附件和模型窗口的估算只建议提前摘要，不直接拒绝请求；没有可行摘要范围时继续交由提供商判断。工具结果仍须在独立字节硬上限内保留最小控制信息，无法满足时明确停止，不静默删除用户输入。

该入口同时覆盖历史重放、当前批次结果、摘要请求、Chat Completions、Responses 和其他 BYOP adapter。Responses 首次裁剪或裁剪内容改变时，会放弃旧状态并发送有界完整历史；相同的裁剪投影用稳定指纹继续复用已接收该版本的状态。支持 `count_input_tokens` 时，按实际窗口和请求显式输出上限检查，不把固定预留比例冒充实际超限；已配置 `context_management` 的服务继续自行压缩。

## 错误处理

明确的 `context_window_exceeded`、`context_length_exceeded` 或“input exceeds the context window”错误会进入上下文超限展示，不受外层 HTTP 502 影响，不再原样自动重试或自动续接。普通 502 仍保留原有重试策略，输出 token 用尽和限流不归类为上下文超限。Responses HTTP、SSE 和 WebSocket 的结构化错误码会保留到分类阶段。

中英文错误提示同步更新。裁剪诊断仅记录结果数量和裁剪前后的字节数，不记录命令、日志正文或凭据。

## 适用边界

本地字节估算不是提供商的精确 tokenizer，未知模型窗口也不是实际能力声明。图片和文件保留原内容并预留估算空间，base64 长度不直接视作文本 token 数；提供商计数接口不可用时，附件和远端状态的精确占用仍无法由客户端保证。实际服务限制更低时，明确超限会停止并给出可操作提示。

## BYOP 自动摘要修复

原有五种 BYOP 协议共用的完成事件没有填入顶层 `token_usage`，控制器因此一直读到 0，基于用量的本地自动摘要未正常触发。已有的工具结果清理 `prune_now` 仍会运行；Responses 的 `context_management` 属于另一条提供商侧压缩路径，只有配置非零 `compact_threshold` 时才发送，默认关闭。

现在使用两条触发路径，均遵守 `byop_compaction_auto` 开关：

- 请求成功后，上报提供商实际输入、输出及缓存明细。输入总量已包含缓存，触发判断只加输入总量和输出，不重复加缓存。模型窗口与 profile 限制取较小值，达到建议输入预算的 80% 后，在原流清理完毕且没有待执行工具时做摘要。正常任务已回答完毕时，不额外再生成一轮回答。
- 发送下一轮之前，先持久化已完成工具结果，再用实际请求构造和工具结果裁剪后的大小做保守估算。达到建议输入预算的 80% 且存在可行摘要范围时，暂存原待发请求，先发摘要。这样不返回 usage 的兼容服务也能触发，工具结果突然变大也不依赖上一轮用量。该估算不写入计费用量，也不因估算偏大直接阻断原请求。

显式配置了 Responses 服务端压缩时，不再叠加本地自动摘要，手动摘要仍可用。

摘要使用当前请求的模型和固定 `CompactionPlan`，包含确切原始消息快照、完整工具配对边界、待保留区起点及前次摘要。优先按预算向前收缩；切点落在单轮内部且没有更早完整轮次时，尝试向后扩展到最近完整且工具闭合的轮次，仍须通过摘要请求预算。手动摘要不存在可发送范围时明确停止，不截断用户正文后声称已完整摘要。摘要不带执行工具，输出上限取 8,000 token、有效窗口的四分之一和模型配置输出上限中的最小值。Anthropic 固定思考预算与输出上限冲突时，缩减思考预算并给正文保留空间；正常可容纳的 High 请求不变。

仅在本次请求返回明确完成的非空正文、最终文本与流式增量一致且源历史快照仍有效时提交。工具调用、输出截断、缺失明确完成标志、正文超过 64 KiB、旧回答或并发修改均不能覆盖历史。摘要失败或取消时保留原历史及待发输入，停止本次自动恢复。

摘要成功后，先清理旧流，再在原会话续发原请求一次，本次续发禁止再次自动摘要。已持久化的工具结果不会变成再次执行命令。内部摘要成功时会话保持进行中，避免排队提示抢在原请求之前发送。失败/取消时在原摘要位置补回待发输入，不把旧取消记录追加到新回答之后，并保留实际失败原因。输入视图校验原提交内容（包括 `/plan`、`/init` 的既有展开规则）后保存真实草稿和上下文；只有本次恢复实际开启新流且草稿未变才清理，等待工具或摘要期间写入的新草稿和附件继续保留。

Responses 的状态指纹包含最近一次本地摘要标识和已清理工具结果的消息 ID，摘要请求也不复用旧服务端链。压缩后必须从本地有效摘要与保留区建立新链，避免旧 `previous_response_id` 或 conversation 重新带入被压缩的原文。

这些保护不能验证摘要的语义是否准确，也不能保证兼容网关声明的模型窗口正确。实际服务拒绝仍超限时，不原样循环重试。自动摘要阶段的待发请求在启用历史保存时写入可恢复检查点；重载后只恢复为停止状态，由用户手动继续。关闭历史保存时仍沿用内存摘要，不写入恢复数据。专用摘要模型设置的历史行为未在本次扩展，摘要复用当前请求模型。

## 回归验证入口

- `request_budget_tests.rs`：数 MB 输出、累计限额、头尾与控制字段、UTF-8/转义、短结果、小模型、未知模型、附件不变。
- `chat_stream_budget_tests.rs`：真实请求组装中的历史结果、当前执行结果、协议切换及 Responses 增量结果。
- `chat_stream_compaction_tests.rs`：五种协议无 usage 的预检、真实 SSE 用量、摘要输出上限、完整终止及禁止工具调用。
- `controller_compaction_tests.rs` 与 `response_stream_compaction_tests.rs`：真实应用模型中的失败/取消、会话目标、提示队列、新请求接管、草稿附件保护与实际窗口阈值。
- `byop_compaction/plan_tests.rs`：固定快照、安全提交、双 task 的 DFS 和用户消息去重、增量摘要及预算收缩后的保留区。
- `api_error_tests.rs` 和 Responses 测试：502 包装、结构化错误码、普通网络错误及输出上限负例。
- `test_context_error_text_layout_in_english_and_chinese`：两种语言、220/360 px 文字宽度、新提示及外层错误消息的原生字体排版。

2026-09-07 第一阶段（工具预算与错误分类）的 macOS 本地验证：

- 受影响模块的 nextest 测试：374/374 通过，包含本次新增的预算、请求组装与错误分类回归。
- `cargo test -p warp --lib i18n::tests`：11/11 通过。
- 双语原生字体排版：手动运行通过，nextest 1/1 通过（运行 ID `2264fca5-2542-44b1-b242-e9bfc44453ec`）。
- 最终源码的 `cargo check` 与 `cargo check -p warp` 均通过。
- 修改文件的 rustfmt 检查及 `git diff --check` 通过。

验证对象是基于 `0a1a2d3e8df1d6386be83008e0d8e1b12bcc9c3a` 的未提交工作区，以上结果不表示该基线提交包含本次修复。自动摘要阶段的最终验证结果另列于下方，不能用第一阶段结果代替。

初版提交 `6d604397b` 已完成跨平台验证；下表仅对应初版，不能代替后续复盘修订的验证。

| 平台 | 结果 | 说明 |
| --- | --- | --- |
| macOS | 通过 | 上述本地构建、单测和双语原生排版检查 |
| Linux x64 | 通过 | Cross-platform preflight：524 项聚焦测试、80 项 rust-genai 测试 |
| Windows x64 | 通过 | 同上，另通过 SSH worker 构建和 PowerShell 参数检查 |

## 自动摘要阶段的最终本地验证

- `cargo nextest run --no-fail-fast -p warp --lib`（筛选 provider、API 错误、compaction、controller 和 conversation 测试）：499/499 通过。
- 自动摘要请求采用专用协议提示词，新增失败提示已同步英文与简体中文；原上下文超限提示也已复核，仍适用于自动恢复无法完成后的停止状态。
- 最终源码的 `cargo check` 和 `cargo check -p warp` 均通过。
- 最终源码的 `cargo test -p warp --lib i18n::tests`：11/11 通过。
- 双语原生字体排版测试通过：覆盖上下文超限及无效摘要提示，在英文/简体中文、220/360 px 下无截断、溢出或缺失字形。
- 36 个修改或新增 Rust 文件的 rustfmt 检查及 `git diff --check` 通过。
- 提交前按扩展后的 CI 筛选在 macOS 运行：524/524 通过，其中既有 `webfetch_description_matches_opencode_verbatim` 被 nextest 标记为输出句柄关闭延迟（leaky），未判失败。工作流 YAML 解析及两端命令结构检查通过。
- 验证日志：`/tmp/infinishell-auto-compaction-nextest-final.log`、`/tmp/infinishell-auto-compaction-workspace-check.log`、`/tmp/infinishell-auto-compaction-warp-check.log`、`/tmp/infinishell-auto-compaction-i18n-final.log`、`/tmp/infinishell-auto-compaction-layout.log`。
- Linux/Windows CI 使用 `Cross-platform preflight`，默认聚焦测试现已包含 Responses、BYOP provider、预算与摘要、控制器恢复、conversation 和本地化回归；本次不启用全工作区测试。

## 对既有功能的复盘与修订

`6d604397b` 的 Linux/Windows CI 各有 524 项聚焦回归和 80 项 rust-genai 测试通过（[运行记录](https://github.com/Infinimesh-ai/InfiniShell-Desktop/actions/runs/34120577609)）。随后从已有功能的兼容性出发复盘，仍发现了此前未覆盖的组合，不能用原先的通过结果代替这些场景的验证。

| 场景 | 原修复的问题 | 修订后的保护 |
| --- | --- | --- |
| 小窗口图片、实际 token 较少的长文本 | 保守字节估算直接拒绝，早于提供商精确计数 | 估算只建议摘要，正文和附件不据此硬拒绝；工具字节上限保持 |
| Claude 旧版固定 thinking + 短摘要 | Medium/High 思考预算不小于摘要 max_tokens，参数无效 | 无工具摘要仅在冲突时限制 thinking；带工具的交错思考允许预算大于 max_tokens，保持原值；覆盖模型后缀推断 |
| 单轮较长输入、尾部切点落在轮内 | 找不到更早完整轮次，即使整轮可容纳也拒绝手动摘要 | 向后尝试最近的完整闭合轮次，仍需预算检查 |
| Responses 远端状态和服务端压缩 | 本地旧全文估算抢先阻断，固定预留比例挡住服务端压缩 | 保留远端精确计数和显式服务端压缩路径 |
| Responses 历史每轮出现同一条大日志 | 每次都重建链，不能复用已接收裁剪版本的状态 | 对实际裁剪投影计算稳定指纹；相同内容复用、同长度但内容变化仍重建 |
| 摘要取消后已开始新回答 | 补记旧取消问题追加到新回答之后，干扰队列判断 | 在原摘要位置补回，保留真实错误，新回答完成后再处理队列 |
| `/plan`、`/init`、等待期间的新草稿及附件 | 用协议展开文本比较编辑框，或误把后来编辑的内容当作原提交 | 按原提交内容校验实际草稿、上下文和对应流，只在真正续发且内容未变时清理 |

以上修订不改变正常 502、限流、输出额度耗尽的既有分类，不删除本地原文，不重执行已完成工具，也不放宽摘要成功提交的校验。此次无需本地化变更；原错误提示继续使用英文与简体中文已有消息键。

协议依据：[OpenAI 服务端压缩](https://developers.openai.com/api/docs/guides/compaction#server-side-compaction)明确支持以 `previous_response_id` 继续已压缩的状态；[Anthropic 思考预算规则](https://platform.claude.com/docs/en/build-with-claude/extended-thinking#budget-rules-and-tuning)规定普通固定预算须小于输出上限，并为工具交错思考保留例外。相关反例同时覆盖实际请求序列化与 HTTP/SSE 流。

本轮修订的最终 macOS 本地验证：

- 按跨平台 CI 同样的筛选运行 nextest：546/546 通过，未出现失败、重试或 leaky 标记。
- `cargo test --manifest-path lib/rust-genai/Cargo.toml --lib`：81/81 通过，包含短输出与工具交错思考的反例。
- `cargo test -p warp --lib i18n::tests`：11/11 通过；`cargo check` 与 `cargo check -p warp` 均通过。
- 应用侧修改文件及新增 adapter 测试文件的 rustfmt 检查、`git diff --check` 均通过。
- 日志保存在 `/tmp/infinishell-byop-review-release-*.log` 与 `/tmp/infinishell-byop-review-genai-final.log`。
- 此处记录的是提交前工作区结果；Linux/Windows 必须再对包含本节修订的远端提交运行 CI，不能沿用初版的结果。


## 自动摘要期间的崩溃恢复补强

启用历史保存的本地会话，在发出内部摘要前，将原问题、用户模式、选中文字、图片、文件正文和引用附件与完整会话快照一起保存。SQLite 事务提交成功才允许开始请求；摘要提交后、原请求续发前再保存一次，避免摘要已完成但待发输入尚未落盘的空档。首次摘要没有 completed 记录时也保存恢复 sidecar，无需数据库迁移。

快照在创建时获得进程内单调版本号。writer 拒绝迟到的旧快照覆盖较新的保存结果；确认通道关闭、队列不可用、事务失败、任务超过既有存储上限或快照被淘汰，都不能当成保存成功。错误使用新增中英文提示，不作为网络故障自动重试。用户在等待保存时取消优先于同时到达的确认，迟到回调不能启动旧请求或影响新请求。

重载后，原输入在原有位置显示为停止状态，不自动运行工具或发送模型请求。点击继续时使用保存的输入和附件，不消费后来编辑的草稿。仅当前末尾的恢复输入具有继续优先权；后来的新问题即使文字相同，也不会误发旧问题。连续多次摘要失败或取消时，尚未发送的旧问题保留在仅供历史展示的恢复列表中，新检查点不覆盖旧问题和附件。续发请求 ID 与真实生成事件保持一致，已落入历史的原输入不会再次恢复或重复发送。工具续接只保留继续所需上下文，不重放已完成的工具结果。

用户主动关闭历史保存、共享会话或运行模式不允许保存时，继续使用既有内存摘要流程。该选择不提供跨进程恢复；不会因为新增检查点强制保存数据。启用保存但数据库故障时则明确停止并保留当前输入。

回归入口新增：

- `persistence::sqlite::checkpoint_tests`：真实 SQLite 事务确认、失败回滚、存储上限、暂停 writer 和快照乱序。
- `byop_compaction/recovery_tests.rs` 与 `conversation_recovery_tests.rs`：附件完整性、旧版本兼容、输入顺序、继续目标和请求去重。
- `controller_compaction_crash_tests.rs`：独立进程运行真实控制器与临时 SQLite，在摘要流尚未结束、以及摘要已提交但续发确认尚未送达两个阶段强杀，再通过正式读取与转换逻辑重载并手动续发。
- `chat_stream_live_tests.rs`：手动运行的真实 Responses 网关冒烟，仅发送合成历史，验证数 MB 日志经过工具预算后请求成功、真实摘要提交、摘要后的约束续接；默认忽略，CI 不读取凭据。

真实冒烟需要显式设置 `INFINISHELL_BYOP_SMOKE_PROVIDER_ID`、`INFINISHELL_BYOP_SMOKE_MODEL_ID` 和 `INFINISHELL_BYOP_SMOKE_API_KEY`，然后运行 `cargo test -p warp --lib live_byop_large_log_summary_and_resume -- --ignored --nocapture`。密钥仅通过测试子进程环境传入，不写入仓库或测试夹具。该用例固定三次生成，Responses 适配器还可能执行精确 token 计数请求。

这些验证不等于覆盖所有兼容网关，也不证明摘要语义无损。缺失 usage 的情形继续由已有五种协议的模拟回归覆盖；语义质量保留为后续有具体案例时分析的边界。


本轮 macOS 定向回归：`cargo nextest run --no-fail-fast -p warp --lib` 选择提供商、上下文摘要、控制器、会话、历史模型、SQLite 检查点和 i18n，共 651/651 通过，包含连续中断的恢复记录保留回归。两个强杀用例均包含重载和实际手动续发；测试使用临时数据库与模拟 HTTP 服务，不依赖开发者数据。日志：`/tmp/infinishell-recovery-nextest-release.log`。


本轮真实网关冒烟：使用本机已配置的 Infinimesh Responses 端点 `https://lapi.infinimesh.cloud/v1/`、模型 `gpt-5.6`，三次合成生成全部完成，约 20.6 秒。数 MB 合成日志的出站工具结果限制到 32 KiB 以内，原历史不变；真实摘要保留固定测试约束，摘要后继续回答准确返回该约束。三次都观察到提供商输出 usage；输入统计为提供商或精确预检计数。未读取真实会话、未执行模型工具，也未实测其他未配置网关。日志：`/tmp/infinishell-recovery-live-smoke.log`。


中英文审计：新增保存失败提示使用同一 Fluent 消息键，无变量差异；`cargo test -p warp --lib i18n::tests` 11/11 通过。原生字体布局手动运行及 nextest 1/1 通过，两种语言在 220/360 px 文字区域均完整换行、无缺字或截断（运行 ID `0f13aac9-2f95-45d8-b11b-8dc01953730d`）。连续中断记录沿用既有停止状态和输入展示，无需额外本地化变更。

最终源码的 `cargo check` 与 `cargo check -p warp` 均通过；修改文件的 rustfmt 检查与 `git diff --check` 通过。提交前门禁日志为 `/tmp/infinishell-recovery-{nextest,i18n,workspace,warp}-release.log`。Linux/Windows CI 对推送后的同一个提交执行，结果以对应 Actions run 为准。
