# world-simulator PRD

审计轮次: 5

## 目标
- 建立 world-simulator 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 world-simulator 模块后续改动可追溯到 PRD-ID、任务和测试。
- 为专题能力提供分册挂载机制，保持主 PRD 可导航、可审计。

## 范围
- 覆盖 world-simulator 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/world-simulator/prd.project.md` 的任务映射。
- 覆盖启动器链路中的链上转账能力（通过分册维护详细条款）。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/world-simulator/prd.md`
- 项目管理入口: `doc/world-simulator/prd.project.md`
- 文件级索引: doc/world-simulator/prd.index.md
- 追踪主键: `PRD-WORLD_SIMULATOR-xxx`
- 测试与发布参考: `testing-manual.md`
- UI 视觉评审列表: `doc/ui_review_result/ui_review_list.md`
- 分册索引:
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`（PRD-WORLD_SIMULATOR-001/002）
  - `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`（PRD-WORLD_SIMULATOR-002/003）
  - `doc/world-simulator/prd/acceptance/visual-review-score-card.md`（PRD-WORLD_SIMULATOR-003）
  - `doc/world-simulator/prd/quality/experience-trend-tracking.md`（PRD-WORLD_SIMULATOR-003）
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`（PRD-WORLD_SIMULATOR-004/005）
  - `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-010）
  - `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-011）
  - `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-012）
  - `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-013）
  - `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-014）
  - `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-015）
  - `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.prd.md`（PRD-WORLD_SIMULATOR-020）
  - `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.prd.md`（PRD-WORLD_SIMULATOR-021）
  - `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.prd.md`（PRD-WORLD_SIMULATOR-022）
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.md`（PRD-WORLD_SIMULATOR-016）
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.md`（PRD-WORLD_SIMULATOR-017）
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.md`（PRD-WORLD_SIMULATOR-018）
  - `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.md`（PRD-WORLD_SIMULATOR-019）
  - `doc/world-simulator/kernel/power-storage-complete-removal-2026-03-06.prd.md`（PRD-WORLD_SIMULATOR-001/002/003）

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2 (2026-03-03): 完成启动器链上转账需求建模与任务拆解。
- M3: 完成 PRD 分册结构落地，主入口仅保留总览与导航。
- M4: 完成链运行时转账 API、运行时账本动作、启动器交互与闭环验收。
- M5 (2026-03-04): 完成无 GUI 服务器场景的启动器 Web 控制台能力建模与落地。
- M6 (2026-03-04): 完成启动器 native 客户端服务端分离，native/web 统一控制面链路与功能对齐。
- M7 (2026-03-06): 完成启动器 Web 端链上转账闭环补齐（控制面代理 + wasm 提交）。
- M8 (2026-03-06): 完成启动器 Web 端设置/反馈入口对齐（设置中心可用化 + 反馈代理提交）。
- M9 (2026-03-06): 完成启动器 native 遗留代码清理与测试资产收敛（移除失效状态字段与未引用旧测试文件）。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
- 分册与主入口不同步会导致需求追踪断裂。

## 1. Executive Summary
- Problem Statement: world-simulator 涵盖场景、viewer、LLM 与 launcher 多条链路，需求持续增长时若全部堆叠在单文档，会降低可维护性并影响变更追踪效率。
- Proposed Solution: 主 PRD 保持模块级边界与验收总口径，专题能力迁移到分册；启动器链上转账作为首个分册能力，维护完整细节。
- Success Criteria:
  - SC-1: simulator/viewer/launcher 变更全部映射 PRD-WORLD_SIMULATOR-ID。
  - SC-2: Web 闭环路径作为默认链路并保持可复现测试证据。
  - SC-3: 场景系统关键基线（初始化、资源、交互）具备稳定回归。
  - SC-4: LLM 交互链路具备可观察性与降级策略记录。
  - SC-5: 启动器链上转账需求在分册中完整定义，并与主 PRD / 项目任务保持一一映射。
  - SC-6: 分册变更后主 PRD 仍可作为唯一入口完成导航与验收追踪。
  - SC-7: 场景系统、Viewer、启动器验收口径统一到同一 checklist，并可直接映射到测试证据。
  - SC-8: Web-first 闭环与 LLM 链路具备统一证据模板，支持发布前快速复核。
  - SC-9: simulator 体验质量指标形成趋势跟踪，支持按周判定退化风险。
  - SC-10: 启动器支持“游戏进程 / 区块链进程”独立编排，且反馈入口严格受链就绪状态门控。
  - SC-11: 启动器“设置”入口升级为完整设置中心，覆盖游戏/区块链/LLM 配置并提供统一可见性。
  - SC-12: Viewer 在 Linux native + Web 双链路下打开 2D/3D 视角时不得出现粉紫屏，且关键交互必须可操作。
  - SC-13: 启动器发行包在可执行文件仍被运行时，重复打包不得出现 `Text file busy` 且不得产生“半更新”产物。
  - SC-14: 启动器支持无 GUI 服务器 Web 控制台，允许远程启动/停止并查看状态日志。
  - SC-15: 启动器 native 与 web 表单字段由同一份 UI schema 驱动，避免配置项漂移。
  - SC-16: 启动器 web 端改为复用同一套 egui UI 代码并以静态资源方式托管，消除独立 HTML 控制台分叉。
  - SC-17: 启动器 wasm 页面初始化不得触发 `time not implemented` 崩溃，Playwright headed 闭环必须可稳定采证。
  - SC-18: 启动器 Web 端不得要求 native-only 二进制路径必填，且 native 端仍保持对应必填校验。
  - SC-19: 启动器 native 与 web 必须通过同一控制面 API 编排游戏/区块链进程，并保持状态与按钮行为一致。
  - SC-20: `world_viewer_live` 必须支持 runtime/world 驱动模式，并在不改 viewer 协议前提下输出可消费的 live 快照与事件。
  - SC-21: runtime 模式必须支持 `LLM/chat/prompt` 控制链路（含鉴权与错误语义），避免与 simulator 模式形成双套体验断裂。
  - SC-22: runtime live 必须补齐高频动作映射与等价回归，并移除 `world_viewer_live` simulator 启动分支，统一 runtime-only 体验。
  - SC-23: runtime live 必须使用真实 LLM 决策链路（AgentRunner/LlmAgentBehavior），且 LLM 失败时硬失败，不得回退启发式。
  - SC-24: runtime live 必须达到 runtime 事件/快照 100% 覆盖，并允许通过协议扩展输出 DecisionTrace。
  - SC-25: 启动器 Web 端必须支持链上转账提交并保持与 native 一致的成功/拒绝/失败语义。
  - SC-26: 启动器 Web 端必须支持设置中心与反馈入口可用，并保持与 native 一致的操作语义（不再为禁用占位）。
  - SC-27: 启动器 native 端已迁移后的历史遗留状态字段/测试资产必须收敛，避免继续引入无效维护成本与告警噪声。

## 2. User Experience & Functionality
- User Personas:
  - 模拟层开发者：需要统一场景与运行语义。
  - Viewer 体验开发者：需要明确 Web/Native 行为边界与验收标准。
  - 发布与测试人员：需要可执行的闭环测试与证据产物。
  - 启动器玩家：需要在同一入口内完成资产查询与转账，无需命令行。
- User Scenarios & Frequency:
  - 场景开发与调试（模拟层开发者）：日常开发日均多次，变更后即时验证场景初始化、事件推进与资源变更。
  - Viewer Web 闭环验收（Viewer 体验开发者/测试人员）：每个功能分支至少 1 次，发布前按 `test_tier_required` 执行。
  - LLM 链路核验（测试人员）：每周例行 + 发布前专项，聚焦可用性、回退策略与错误签名。
  - 启动器转账操作（启动器玩家）：按需触发，典型为启动后 1~3 次余额查询与转账提交。
  - 发布复核（发布负责人）：每个版本候选至少 1 次，汇总 checklist、证据模板与回归结论。
- User Stories:
  - PRD-WORLD_SIMULATOR-001: As a 模拟层开发者, I want unified world-simulator contracts, so that scenario evolution is stable.
  - PRD-WORLD_SIMULATOR-002: As a Viewer 开发者, I want consistent web-first UX rules, so that user paths remain predictable.
  - PRD-WORLD_SIMULATOR-003: As a 发布人员, I want reproducible simulator closure tests, so that releases are verifiable.
  - PRD-WORLD_SIMULATOR-004: As a 启动器玩家, I want to submit a blockchain transfer in launcher, so that I can move main token balances without external tools.（详见分册）
  - PRD-WORLD_SIMULATOR-005: As a 链路开发者, I want transfer requests to be replay-safe and traceable, so that transfer execution is secure and auditable.（详见分册）
  - PRD-WORLD_SIMULATOR-006: As a 启动器玩家, I want separate controls for blockchain/game startup and feedback gated by chain readiness, so that startup behavior is predictable and feedback availability is explicit.
  - PRD-WORLD_SIMULATOR-007: As a 启动器玩家, I want a complete settings center for game/blockchain/LLM, so that all launch-related configuration can be managed from one place.
  - PRD-WORLD_SIMULATOR-008: As a Viewer 开发者, I want native/web rendering defaults to avoid tonemapping fallback regressions, so that 2D/3D rendering remains stable and operable.
  - PRD-WORLD_SIMULATOR-009: As a 发布工程师, I want launcher bundle rebuild to safely replace running binaries, so that repeated packaging does not fail or leave mixed-version artifacts.
  - PRD-WORLD_SIMULATOR-010: As an 运维人员, I want a web-based launcher control plane, so that headless servers can be operated through network browsers.
  - PRD-WORLD_SIMULATOR-011: As a 启动器开发者, I want shared launcher UI schema across native/web, so that form fields and labels stay consistent.
  - PRD-WORLD_SIMULATOR-012: As a 启动器开发者, I want launcher egui UI to be reused across native/wasm with static asset serving, so that we no longer maintain a separate HTML console.
  - PRD-WORLD_SIMULATOR-013: As a 启动器开发者, I want launcher wasm to use web-compatible time primitives, so that web UI startup does not panic in browser runtime.
  - PRD-WORLD_SIMULATOR-014: As an 运维人员, I want web launcher required checks to ignore native-only binaries, so that browser startup is not blocked by irrelevant fields.
  - PRD-WORLD_SIMULATOR-015: As a 启动器玩家, I want native/web launcher to share the same control-plane API, so that game/blockchain control behavior remains fully aligned across platforms.
  - PRD-WORLD_SIMULATOR-016: As a 玩法架构开发者, I want world_viewer_live to run on runtime/world with protocol compatibility, so that live behavior aligns with runtime semantics without immediate viewer model rewrite.
  - PRD-WORLD_SIMULATOR-017: As a 玩法架构开发者, I want runtime live to support llm/chat/prompt controls, so that runtime 联调不再依赖 simulator 模式兜底。
  - PRD-WORLD_SIMULATOR-018: As a 玩法架构开发者, I want runtime live action mapping coverage with parity regression and runtime-only launch path, so that world_viewer_live no longer keeps a simulator fallback branch.
  - PRD-WORLD_SIMULATOR-019: As a 玩法架构开发者, I want runtime live to use true LLM decisions with 100% runtime event/snapshot coverage, so that runtime 行为与观测不再被启发式上限限制。
  - PRD-WORLD_SIMULATOR-020: As a Web 启动器玩家, I want to submit blockchain transfers in browser, so that I can complete asset interaction without native tools.
  - PRD-WORLD_SIMULATOR-021: As a Web 启动器玩家, I want settings and feedback entries to work in browser, so that I can complete the same control loop as native.
  - PRD-WORLD_SIMULATOR-022: As a 启动器开发者, I want native legacy launcher code to be removed after control-plane unification, so that code ownership and long-term maintenance are cleaner.
- Critical User Flows:
  1. Flow-WS-001（Web-first 闭环）:
     `选择场景 -> 启动 Viewer Web -> 执行关键交互 -> 采集日志/截图/指标 -> 产出 test_tier_required 结论`
  2. Flow-WS-002（LLM 链路验证）:
     `启动链路 -> 触发 LLM 交互 -> 验证响应时延/错误处理 -> 故障时走降级路径 -> 归档证据`
  3. Flow-WS-003（启动器转账成功）:
     `启动器加载 -> 填写 from/to/amount/nonce -> 提交 -> runtime 接收并生成事件 -> UI 展示成功`
  4. Flow-WS-004（启动器转账失败）:
     `提交转账 -> 余额不足/nonce 重放/参数非法 -> API 返回结构化错误 -> UI 保留可诊断错误签名`
  5. Flow-WS-005（反馈分布式提交回退）:
     `提交反馈 -> 链状态服务 Connection refused -> 回落本地落盘 -> 展示失败原因并保留远端错误`
  6. Flow-WS-006（Viewer 粉紫屏回归）:
     `启动 native viewer -> 切换 2D/3D -> 观察渲染与交互 -> 若异常则抓取日志/截图 -> 修复后执行 Web+native 双链路回归`
  7. Flow-WS-007（Launcher 重复打包容错）:
     `已有 bundle 运行中 -> 再次执行 build-game-launcher-bundle -> 二进制替换成功 -> 产物完整可启动`
  8. Flow-WS-008（Launcher Web 控制台远程运维）:
     `SSH 启动 world_web_launcher -> 浏览器访问控制台 -> 提交配置并启动 -> 观察状态/日志 -> 远程停止`
  9. Flow-WS-009（Launcher UI Schema 共享）:
     `更新共享 schema -> native 配置区渲染同步变更 -> web 控制台通过 /api/ui/schema 动态渲染同字段`
  10. Flow-WS-010（Launcher egui Web 同层复用）:
     `构建 launcher wasm 静态资源 -> world_web_launcher 托管静态目录 -> 浏览器加载同一 egui UI -> 调用 /api/state|start|stop`
  11. Flow-WS-011（Launcher wasm 时间兼容闭环）:
     `Playwright headed 打开 launcher Web 页面 -> wasm 初始化 -> 无 time panic -> snapshot/console/screenshot 采证`
  12. Flow-WS-012（Launcher Web 必填校验分流）:
     `浏览器加载配置 -> 必填校验 -> 不再提示 launcher/chain runtime bin 必填 -> 继续启动流程`
  13. Flow-WS-013（Launcher native/web 同控制面）:
     `启动 world_web_launcher -> native/wasm 客户端轮询 /api/state -> 分别触发 /api/start|stop 与 /api/chain/start|stop -> 状态一致反馈`
  14. Flow-WS-014（Viewer live runtime 接管 Phase 1）:
     `world_viewer_live --runtime-world -> runtime::World 驱动 step -> 适配为 simulator 协议快照/事件 -> agent_world_viewer 消费`
  15. Flow-WS-015（Viewer live runtime 接管 Phase 2）:
     `world_viewer_live --runtime-world --llm -> PromptControl/AgentChat 鉴权通过 -> LLM 决策动作桥接 runtime 执行 -> 输出兼容事件/快照`
  16. Flow-WS-016（Viewer live runtime 接管 Phase 3）:
     `补齐 action 映射 -> 增加等价回归 -> 删除 world_viewer_live simulator 启动分支 -> 统一 runtime-only 启动与错误语义`
  17. Flow-WS-017（Viewer live runtime 真 LLM 全量接管）:
     `真实 LLM 决策 -> runtime action 执行 -> 100% 事件/快照映射 -> DecisionTrace 输出 -> 无启发式 fallback`
  18. Flow-WS-018（Launcher Web 链上转账闭环）:
     `链状态就绪 -> 打开转账窗口 -> 填写 from/to/amount/nonce -> 通过 /api/chain/transfer 提交 -> 返回 action_id 或结构化错误`
  19. Flow-WS-019（Launcher Web 设置/反馈对齐）:
     `链状态就绪 -> 打开设置窗口编辑参数 -> 打开反馈窗口提交 kind/title/description -> 通过 /api/chain/feedback 返回 feedback_id/event_id 或结构化错误`
  20. Flow-WS-020（Launcher native 遗留清理）:
     `确认 native 已走统一控制面 -> 删除失效状态字段/未引用旧测试文件 -> 运行 native+wasm 回归 -> 状态与交互行为保持不变`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 启动器转账提交 | `from_account`、`to_account`、`amount`、`nonce`；`amount` > 0；`nonce` 为非负整数 | 点击“提交转账”触发请求；非法输入阻止提交并显示错误 | `idle -> validating -> submitting -> success/failed` | 余额与 nonce 校验在 runtime 执行；`nonce` 必须严格递增 | 仅在区块链功能可用且配置合法时允许提交 |
| 顶部区块链状态指示 | `disabled/not_started/booting/ready/unreachable/config_error` | 启动后自动探针；状态变化实时刷新；错误支持 hover 详情 | `booting -> ready/unreachable/config_error` | 默认 1s 周期探测；短超时避免阻塞 UI | 状态读取只读，无写权限 |
| Viewer Web 闭环入口 | 场景标识、测试命令、产物路径、结论 | 执行 Web-first 流程并输出证据模板 | `planned -> running -> evidence_ready -> reviewed` | 按场景优先级与风险等级执行 | required gate 为默认必经路径 |
| LLM 链路验证 | 模型配置、提示模板、超时阈值、降级开关 | 触发交互与降级；记录成功/失败信号 | `ready -> invoking -> fallback(optional) -> completed` | 统计成功率、超时率、回退率 | 仅测试环境允许注入调试配置 |
| 反馈分布式提交 | 反馈文本、会话标识、链状态地址 | 远端失败时自动写本地文件，不丢失原始错误 | `submitting -> remote_failed -> local_saved -> reported` | 本地落盘路径按日期归档；错误签名保留用于回归 | 仅本地授权实例可写入反馈归档目录 |
| Viewer 色调映射兼容 | `render_profile`、`tonemapping`、`bevy feature`（含 `tonemapping_luts`） | 启动时加载后处理组件；缺失依赖时不得进入粉紫回退态 | `boot -> render_ready -> interactive` | 默认 profile 保持稳定可视；2D/3D 切换不重置错误态 | 对外仅保留配置入口；底层 feature 由构建配置管控 |
| Launcher bundle 二进制替换容错 | `OUT_DIR/bin/*`、目标文件占用状态、`--profile`、`--web-dist` | 打包时先删除目标二进制再 copy；若旧进程占用也需完成替换 | `build_start -> binaries_replaced -> web_prepared -> bundle_ready` | 同一 `OUT_DIR` 多次执行后产物版本需一致，不得残留半更新状态 | 仅本地构建者可写 bundle 输出目录 |
| Launcher Web 控制台 | `scenario/live_bind/web_bind/viewer_host/viewer_port/viewer_static_dir/llm/chain` | 浏览器点击“启动/停止”触发子进程编排与状态刷新 | `idle -> running -> stopped/exited` | bind/端口/目录先校验，失败时拒绝启动并返回错误详情 | 默认部署在受信网络，具备远程访问能力 |
| Launcher UI schema 共享 | `id/section/kind/label_zh/label_en/web_visible/native_visible` | UI 按 schema 动态渲染字段，新增字段无需双端重复定义 | `schema_loaded -> form_ready` | section 内按 schema 顺序渲染 | schema 只读；不含敏感数据 |
| Launcher egui Web 复用与静态托管 | `launcher wasm dist`、`console_static_dir`、`/api/state|start|stop` | `world_web_launcher` 托管静态资源并由浏览器运行同一 egui UI | `boot -> static_ready -> interactive` | 静态请求走目录白名单，API 路径优先级高于静态路径 | 受信网络部署，禁止目录穿越 |
| Launcher wasm 时间兼容 | `WEB_POLL_INTERVAL_MS`、`last_web_poll_at`、web time primitive | 页面轮询 `/api/state` 且不触发 wasm 时间平台 panic | `interactive -> polling -> synced` | 轮询基于单调时钟差值，防并发请求堆积 | 浏览器会话可读，接口由受信网络控制 |
| Launcher Web 必填分流 | `launcher_bin`、`chain_runtime_bin`（native-only） | Web 端校验排除 native-only 必填，native 端保持阻断 | `config_loaded -> validated` | 按 `target_arch` 分流 | 平台边界一致 |
| Launcher native/web 同控制面 | `/api/state`、`/api/start`、`/api/stop`、`/api/chain/start`、`/api/chain/stop` | native 与 web 端统一走 API 触发游戏/区块链独立启停 | `service_ready -> game_running/stopped + chain_starting/ready/stopped` | 状态以服务端快照为准，客户端仅做展示与轮询 | 控制面部署在受信网络，客户端仅消费授权 API |
| Launcher Web 转账闭环 | `/api/chain/transfer`、`from/to/amount/nonce` | 浏览器端提交转账请求并展示成功/拒绝/失败 | `idle -> validating -> submitting -> success/failed` | `amount/nonce > 0`、`from != to`，账本规则以 runtime 为准 | 链就绪才允许提交 |
| Launcher Web 设置/反馈对齐 | 设置窗口字段 + `/api/chain/feedback` + `kind/title/description` | 浏览器端可打开设置窗口与反馈窗口；反馈提交通过控制面代理返回结构化结果 | `settings: closed/open/saved` + `feedback: idle/validating/submitting/success/failed` | 反馈标题/描述必填；单请求 in-flight 门控 | 反馈提交仅链就绪可用；设置仅当前会话可编辑 |
| Launcher native 遗留清理 | native 失效状态字段、无效常量 `cfg` 边界、未引用旧测试文件 | 保持现有 UI/API 行为不变前提下清理历史残留 | `legacy_present -> removed -> regression_passed` | 优先删除“无读写路径/无编译入口引用”的资产 | 仅开发维护路径可修改，运行时玩家能力不变 |
| Viewer live runtime 接管 | `--runtime-world`、runtime `DomainEvent`、兼容 `WorldSnapshot/WorldEvent` | 启动 runtime 模式后按 Play/Step 推进 runtime，并推送兼容快照/事件 | `simulator_mode/runtime_mode`（启动参数决定） | 事件序列保持单调；至少映射注册/移动/转移/拒绝四类事件 | 本地开发链路，默认不开放远程写接口 |
| Viewer live runtime LLM/chat/prompt 接管 | `--runtime-world --llm`、`PromptControl`、`AgentChat`、auth proof、nonce | 运行时允许 prompt 预览/应用/回滚与 agent chat，驱动 LLM 决策并桥接可映射动作 | `runtime_script/runtime_llm` + `profile[vN]->profile[vN+1]` | 版本单调递增；nonce 必须递增；不可映射动作输出可诊断拒绝 | 仅本地受控链路可写，鉴权签名与绑定校验必经 |
| Viewer live runtime action 覆盖与分支收敛 | `simulator_action_to_runtime`、`ActionRejected::RuleDenied`、`world_viewer_live` runtime-only CLI | 补齐 runtime 可执行映射并对不可映射动作保持结构化拒绝；启动链路移除 simulator 分支 | `runtime_bridge_partial -> runtime_bridge_hardened` | 映射成功动作优先执行；不可映射动作拒绝语义稳定可回归 | 不新增远程写入口，仅本地受控链路 |
- Acceptance Criteria:
  - AC-1: world-simulator PRD 覆盖场景、Viewer、LLM、启动器四条主线。
  - AC-2: world-simulator project 文档维护任务拆解与状态。
  - AC-3: 与 `doc/world-simulator/scenario/scenario-files.prd.md`、`doc/world-simulator/viewer/viewer-web-closure-testing-policy.prd.md` 等专题文档一致。
  - AC-4: 关键交互变更同步更新 testing 手册与测试记录。
  - AC-5: 分册内专题条款（接口/安全/测试）在主 PRD 中可定位、在项目文档中可执行。
  - AC-6: 统一验收清单覆盖场景、Viewer Web 闭环、启动器入口与证据模板，并与 `testing-manual.md` 一致。
  - AC-7: Web-first 与 LLM 证据模板可直接用于 S6/S8，且必填字段包含命令、产物路径、指标、结论。
  - AC-8: 体验质量趋势文档定义指标、采集节奏、归档路径与记录模板，并可落地执行。
  - AC-9: 启动器反馈分布式提交流程在链状态服务 `Connection refused` 时必须回落本地落盘，并保留远端连接失败错误签名用于诊断与回归测试。
  - AC-10: 启动器顶部必须可视化区块链启动状态（含禁用/未启动/启动中/已就绪/不可达），用于玩家快速判定链能力是否可用。
  - AC-11: 客户端启动器必须将“区块链启动/停止”与“游戏启动/停止”拆分为独立按钮；打开启动器后默认自动拉起区块链进程；游戏启动链路不再隐式托管区块链进程。
  - AC-12: 仅当区块链处于“已就绪”状态时，反馈按钮可用并允许打开反馈窗口；区块链未启动/启动中/不可达时反馈入口需明确禁用。
  - AC-13: 设置窗口必须提供完整配置分区（游戏、区块链、LLM），并覆盖启动器运行所需的核心参数编辑入口。
  - AC-14: 设置中心内的 LLM 配置（`llm.api_key/base_url/model`）必须支持文件重载与保存；游戏/区块链配置变更应即时作用于启动器内存配置。
  - AC-15: `agent_world_viewer` native 链路在默认渲染配置下不得出现 `TonyMcMapFace tonemapping requires tonemapping_luts feature` 错误；2D/3D 均可正常渲染并可交互。
  - AC-16: `scripts/build-game-launcher-bundle.sh` 在 `OUT_DIR/bin` 目标文件已存在且正被运行时，重复执行仍可成功产出完整 bundle，不得出现 `Text file busy` 或“二进制部分更新”状态。
  - AC-17: `world_web_launcher` 支持在无 GUI 服务器上通过浏览器完成启动/停止、状态查询与日志查看，且打包产物提供独立入口脚本。
  - AC-18: `agent_world_client_launcher` 与 `world_web_launcher` 必须消费同一份 launcher UI schema；web 控制台表单字段通过 `/api/ui/schema` 动态渲染。
  - AC-19: 启动器 web UI 必须由 `agent_world_client_launcher` 的 egui wasm 产物提供；`world_web_launcher` 默认托管该静态目录且保持 API 闭环可用。
  - AC-20: 启动器 wasm 页面在 Playwright headed 打开后不得出现 `time not implemented on this platform` 或 `RuntimeError: unreachable`，并需输出 snapshot/console/screenshot 证据。
  - AC-21: 启动器 Web 端不得再提示 `launcher bin` 与 `chain runtime bin` 必填；native 端保留对应必填校验。
  - AC-22: 启动器 native 与 web 必须统一消费 `world_web_launcher` 控制面 API，并支持链/游戏独立启停及一致状态展示。
  - AC-23: `world_viewer_live --runtime-world` 可启动 runtime 驱动 live server，并保持现有 viewer 协议兼容（`WorldSnapshot/WorldEvent`）。
  - AC-24: `world_viewer_live --runtime-world --llm` 必须支持 prompt/chat 鉴权与控制闭环，runtime script 模式对 prompt/chat 返回 `llm_mode_required`。
  - AC-25: runtime live 补齐动作映射覆盖并新增等价回归；`world_viewer_live` 移除 simulator 启动分支并统一 runtime-only 路径。
  - AC-26: runtime live 使用真实 LLM 决策链路且 LLM 失败时硬失败，不得回退启发式。
  - AC-27: runtime 事件/快照映射覆盖率 100%，DecisionTrace 可被 viewer 订阅并包含错误上下文。
  - AC-28: 启动器 Web 端支持转账提交（`/api/chain/transfer`），并可展示结构化 `action_id/error_code/error`。
  - AC-29: 启动器 Web 端 `设置` 与 `反馈` 入口可用，反馈提交流程通过 `/api/chain/feedback` 返回结构化成功/失败结果。
  - AC-30: 启动器 native 已失效遗留代码（状态字段/测试资产）完成清理后，`agent_world_client_launcher` 与 `world_web_launcher` required 回归通过且行为无回归。
- Non-Goals:
  - 不在本 PRD 中详细列出每个 UI 像素级规范。
  - 不替代 world-runtime/p2p 的底层协议设计。
  - 不在主 PRD 中展开专题实现细节（转账细节迁移至分册）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: LLM 回归脚本、Playwright 闭环、场景矩阵测试、启动器集成测试。
- Evaluation Strategy: 以场景启动成功率、关键交互完成率、LLM 链路稳定性、闭环缺陷收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: world-simulator 连接 runtime 与 viewer，负责把世界状态转化为可交互体验，并通过场景系统与启动器提供可复现实验环境。专题能力通过分册文档按域维护。
- Integration Points:
  - `doc/world-simulator/scenario/scenario-files.prd.md`
  - `doc/world-simulator/viewer/viewer-web-closure-testing-policy.prd.md`
  - `doc/ui_review_result/ui_review_list.md`
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`
  - `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`
  - `doc/world-simulator/prd/acceptance/visual-review-score-card.md`
  - `doc/world-simulator/prd/quality/experience-trend-tracking.md`
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`
  - `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.prd.md`
  - `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.prd.md`
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.md`
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.md`
  - `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.md`
  - `crates/agent_world_launcher_ui/src/lib.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/transfer_window_web.rs`
  - `crates/agent_world_client_launcher/src/launcher_core.rs`
  - `crates/agent_world_client_launcher/Cargo.toml`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world/src/viewer/runtime_live.rs`
  - `crates/agent_world_client_launcher/index.html`
  - `scripts/build-game-launcher-bundle.sh`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 网络异常：链状态探针或转账提交失败时，返回结构化错误并在 UI 展示可诊断提示。
  - 接口超时：请求超时后不得阻塞主线程，状态回落 `unreachable` 并保留超时上下文。
  - 空数据：余额/场景列表为空时展示空态，不允许进入依赖数据的后续动作。
  - 权限不足：未启用链能力或配置不合法时，禁用转账入口并提示原因。
  - 并发冲突：同账户并发转账以 nonce 作为幂等与反重放边界，不满足递增规则即拒绝。
  - 数据异常：收到非预期响应结构时转为“失败且可重试”状态，并写入诊断日志。
  - 远端不可达回退：反馈提交流程在 `Connection refused` 时必须本地落盘，保证证据不丢失。
  - 渲染能力缺口：当色调映射依赖缺失时，viewer 必须避免进入粉紫回退屏，并保留可诊断日志用于回归。
  - 产物覆写冲突：当 bundle 目标二进制正在运行时，打包脚本必须通过“删除后复制”避免 `Text file busy` 并确保输出目录完整可启动。
  - 无 GUI 环境：桌面 GUI 不可用时需通过 Web 控制台操作启动器，且必须支持远程状态可见与错误可诊断。
  - wasm 时间兼容：浏览器运行路径不得使用不支持的平台时间实现，避免页面初始化阶段 panic 直接阻断闭环。
  - Web 必填误判：Web API 配置不含 native-only 字段时，必填校验必须按平台分流，防止误报阻断。
  - 控制面分离回归：native 若无法拉起本地 `world_web_launcher`，必须回传可诊断错误且禁止误导性“运行中”状态。
  - Web 设置存储失败：浏览器禁用本地存储时，设置窗口需返回明确失败提示，不得 silent fail。
  - Web 反馈代理失败：`/api/chain/feedback` 不可达或上游拒绝时，前端需展示结构化错误并保留重试能力。
  - runtime 映射覆盖不足：runtime `DomainEvent` 未全量映射时，需降级输出可诊断事件并保留序列一致性。
  - runtime llm 桥接缺口：LLM 决策动作若无 runtime 映射实现，需返回结构化拒绝并继续服务循环，禁止 panic/卡死。
- Non-Functional Requirements:
  - 性能目标:
    - NFR-1: 启动器链状态探针刷新周期 <= 1s，状态可见延迟 <= 2s。
    - NFR-2: 本地链路下转账提交 API `p95` 响应时间 <= 500ms。
  - 兼容性目标:
    - NFR-3: Launcher/Web 闭环流程在 Linux/macOS 开发环境可执行并产出一致证据结构。
    - NFR-8: Viewer 2D/3D 默认渲染在 Linux native + Web 环境可稳定启动，且不出现粉紫回退屏。
    - NFR-9: 复用同一 `--out-dir` 连续执行 `build-game-launcher-bundle.sh`（含“上一次 bundle 仍运行”场景）时，打包成功率需为 100%（`test_tier_required`）。
    - NFR-10: `world_web_launcher` 在受信网络下可绑定 `0.0.0.0`，并在浏览器端稳定轮询状态接口（`p95 <= 200ms`，本地网络）。
    - NFR-11: `/api/ui/schema` 响应 `p95 <= 100ms`（本地网络），且 schema 新增字段不破坏既有渲染逻辑。
    - NFR-12: launcher wasm 静态资源由 `world_web_launcher` 托管时，首屏可交互时间（本地网络）`p95 <= 2s`。
    - NFR-13: launcher wasm 在 headed 浏览器启动后 `console error` 不得包含 `time not implemented on this platform`，且不出现 `RuntimeError: unreachable`。
    - NFR-14: Web 必填校验分流后不得新增 native 校验退化，`launcher_bin`/`chain_runtime_bin` 在 native 仍为必填。
    - NFR-15: native 与 web 客户端状态刷新节奏一致（默认 1s），不得出现持续状态漂移（>2 个轮询周期）。
    - NFR-16: runtime live 模式下，`Step` 控制请求本地执行延迟 `p95 <= 100ms`（默认场景、无外部依赖）。
    - NFR-17: runtime live llm 模式下，prompt/chat 鉴权失败请求返回延迟 `p95 <= 100ms`（本地环境）。
    - NFR-18: runtime live action 映射不可覆盖项必须稳定返回结构化拒绝，且 `world_viewer_live` runtime-only 启动路径在 required 回归中通过率为 100%。
    - NFR-19: Web 转账提交（控制面代理 + runtime 受理）本地链路 `p95 <= 500ms`，失败路径 `p95 <= 1s` 返回结构化结果。
    - NFR-20: Web 反馈提交（控制面代理 + runtime 受理）本地链路 `p95 <= 500ms`，失败路径 `p95 <= 1s` 返回结构化结果。
  - 安全与隐私目标:
    - NFR-4: 日志与证据中不得输出私钥、口令、完整凭据；敏感字段需脱敏。
    - NFR-5: 转账请求必须经过 nonce anti-replay 与余额约束校验。
  - 数据规模目标:
    - NFR-6: 场景清单规模 200 条时，场景选择与启动流程仍可在可接受等待时间内完成（< 3s 首屏可操作）。
  - 可扩展性目标:
    - NFR-7: 新增 launcher 链上动作时，不破坏既有转账请求结构与验收模板字段。
- Security & Privacy:
  - Viewer/launcher 链路涉及配置与鉴权注入时必须最小暴露。
  - 调试接口需受限并可审计。
  - 专题安全规则在对应分册中维护（含转账安全约束）。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 world-simulator 主 PRD 边界。
  - v1.1: 完成分册化治理与启动器链上转账专题建模。
  - v2.0: 建立体验质量趋势指标（启动、交互、性能、稳定性）。
- Technical Risks:
  - 风险-1: 前端体验迭代快导致行为回归频发。
  - 风险-2: LLM 外部依赖波动影响端到端稳定性。
  - 风险-3: 分册索引维护缺失导致需求追踪断链。

## 6. Validation & Decision Record
- Test Plan & Traceability:

| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-001 | TASK-WORLD_SIMULATOR-001/002/010/011/018 | `test_tier_required` | 文档结构校验、统一验收清单覆盖检查、关键入口引用可达检查 | 场景系统基线、模块导航与入口一致性 |
| PRD-WORLD_SIMULATOR-002 | TASK-WORLD_SIMULATOR-002/003/012/013/018 | `test_tier_required` | Playwright Web 闭环、反馈回退回归、链状态探针单测 | Viewer 交互、Launcher 状态提示、故障诊断可见性 |
| PRD-WORLD_SIMULATOR-003 | TASK-WORLD_SIMULATOR-003/004/010/018 | `test_tier_required` + `test_tier_full` | LLM 证据模板审查、趋势指标周报抽样、长稳回归 | LLM 链路稳定性、发布证据完整性 |
| PRD-WORLD_SIMULATOR-004 | TASK-WORLD_SIMULATOR-005/006/008/009/018 | `test_tier_required` | 转账 API 单测、启动器转账提交流程测试、闭环证据归档 | Launcher 转账入口、runtime API 契约 |
| PRD-WORLD_SIMULATOR-005 | TASK-WORLD_SIMULATOR-005/007/009/018 | `test_tier_required` + `test_tier_full` | runtime main token 转账语义测试（余额/nonce anti-replay）、多轮回归 | 账本一致性、反重放策略、发布前链路风险 |
| PRD-WORLD_SIMULATOR-006 | TASK-WORLD_SIMULATOR-014/015 | `test_tier_required` | 启动器链/游戏独立启动与反馈门控回归测试 | 启动链路可预测性与反馈可用性 |
| PRD-WORLD_SIMULATOR-007 | TASK-WORLD_SIMULATOR-016/017 | `test_tier_required` | 设置中心分区配置读写与生效验证 | 启动器配置可用性与一致性 |
| PRD-WORLD_SIMULATOR-008 | TASK-WORLD_SIMULATOR-019/020 | `test_tier_required` + `test_tier_full` | native 抓帧脚本复现/回归、`agent_world_viewer` 单测与构建检查 | Viewer 2D/3D 渲染稳定性、native 交互可用性 |
| PRD-WORLD_SIMULATOR-009 | TASK-WORLD_SIMULATOR-021/022 | `test_tier_required` | 启动 `run-game.sh` 占用二进制后重复执行 bundle 脚本，验证无 `Text file busy` 且新产物可启动 | 启动器发行打包稳定性、重复发布可靠性 |
| PRD-WORLD_SIMULATOR-010 | TASK-WORLD_SIMULATOR-023/024 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher` + 启动 `world_web_launcher` 后通过 `/api/start`/`/api/stop`/`/api/state` 回归 + `bash -n scripts/build-game-launcher-bundle.sh` 校验打包入口脚本 | 无 GUI 服务器远程运维、launcher Web 控制能力 |
| PRD-WORLD_SIMULATOR-011 | TASK-WORLD_SIMULATOR-025/026 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world_launcher_ui` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher` + `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher`，验证 shared schema 定义、native/web 同源渲染与接口输出 | 启动器 UI 一致性、跨端配置项治理能力 |
| PRD-WORLD_SIMULATOR-012 | TASK-WORLD_SIMULATOR-027/028 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher` + `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher` + `bash -n scripts/build-game-launcher-bundle.sh`，验证同层 egui 复用、静态托管与打包入口 | 启动器 UI 统一维护能力、headless 运维体验一致性 |
| PRD-WORLD_SIMULATOR-013 | TASK-WORLD_SIMULATOR-029/030 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + headed Playwright 打开 `world_web_launcher` 控制台并校验 `console error` 无 `time not implemented` + 归档 screenshot/console/snapshot 证据 | 启动器 Web 端可用性、wasm 运行时兼容稳定性 |
| PRD-WORLD_SIMULATOR-014 | TASK-WORLD_SIMULATOR-031/032 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + headed Playwright 打开 launcher web 并回归 `/api/start` `/api/stop`，验证 Web 端不再受 native-only 必填项阻断 | 启动器 Web 配置校验准确性、跨端校验边界一致性 |
| PRD-WORLD_SIMULATOR-015 | TASK-WORLD_SIMULATOR-033/034/035 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + headed Playwright 覆盖链/游戏独立启停 | 启动器 native/web 控制面一致性、链路维护成本与回归稳定性 |
| PRD-WORLD_SIMULATOR-016 | TASK-WORLD_SIMULATOR-036/037 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_viewer_live` + `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_viewer_live`，验证 runtime 驱动 live 链路与协议兼容适配 | viewer live runtime/simulator 双模式一致性与迁移风险可控 |
| PRD-WORLD_SIMULATOR-017 | TASK-WORLD_SIMULATOR-038/039 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_viewer_live` + `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_viewer_live`，验证 runtime llm/chat/prompt 控制链路打通与脚本模式边界错误码 | viewer live runtime llm/script 体验连续性、鉴权与桥接稳定性 |
| PRD-WORLD_SIMULATOR-018 | TASK-WORLD_SIMULATOR-040/041 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_viewer_live` + `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_viewer_live`，验证 action 映射覆盖扩展、等价回归与 runtime-only 启动分支收敛 | viewer live runtime 映射覆盖稳定性、旧分支移除风险与体验一致性 |
| PRD-WORLD_SIMULATOR-019 | TASK-WORLD_SIMULATOR-042/043/044/045 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_viewer_live` + `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_viewer_live`，验证真实 LLM 决策链路、100% 映射覆盖与硬失败语义 | runtime live LLM 行为真实性与观测完整性 |
| PRD-WORLD_SIMULATOR-020 | TASK-WORLD_SIMULATOR-046/047 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`，验证 Web 转账代理与 wasm 提交流程 | 启动器 Web 转账闭环可用性与跨端语义一致性 |
| PRD-WORLD_SIMULATOR-021 | TASK-WORLD_SIMULATOR-048/049 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`，验证 Web 设置中心可用化与反馈代理提交闭环 | 启动器 Web 设置/反馈跨端一致性与功能可达性 |
| PRD-WORLD_SIMULATOR-022 | TASK-WORLD_SIMULATOR-050/051 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`，验证 native 遗留代码清理后行为稳定 | 启动器 native 维护面收敛与跨端行为稳定性 |

- Decision Log:

| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WS-001 | 采用 Web-first 作为默认 UI 闭环链路，native 抓帧仅 fallback | 以 native 图形链路为默认 | 与 `testing-manual.md` 的 S6 约束一致，能提升复现稳定性与自动化覆盖。 |
| DEC-WS-002 | 主 PRD 保持总览，专题细节下沉到 `doc/world-simulator/prd/*` 分册 | 所有条款继续堆叠在主 PRD | 主入口可读性和变更追踪效率更高，且满足单文档长度约束。 |
| DEC-WS-003 | 反馈分布式提交在 `Connection refused` 时强制本地回落并保留错误签名 | 远端失败直接报错终止，不落盘 | 可确保证据不丢失，便于回归和线上诊断；已被 `TASK-WORLD_SIMULATOR-012` 验证。 |
| DEC-WS-004 | 保留 `TonyMcMapFace` 默认色调映射并显式启用 `bevy/tonemapping_luts` 依赖 | 改默认 tonemapping 或运行时静默降级 | 保持既有视觉基线，同时消除 native 粉紫回退与不可交互回归；已由 `TASK-WORLD_SIMULATOR-020` 的 native 抓帧与 viewer 回归验证。 |
| DEC-WS-005 | `build-game-launcher-bundle.sh` 在 copy 前删除目标二进制，规避运行中覆盖 `Text file busy` | 保持直接 `cp` 覆写（遇占用即失败） | Linux 上删除运行中可执行文件不会中断现有进程，可确保同一路径重建 bundle 不进入半更新态；已由 `TASK-WORLD_SIMULATOR-022` 通过“运行中重复打包”回归验证。 |
| DEC-WS-006 | 新增 `world_web_launcher` 作为 headless 场景控制平面 | 仅保留桌面 GUI / 仅依赖 CLI 手工操作 | headless 服务器无图形会话，Web 控制台可在浏览器统一操作并保留日志可观察性；由 `TASK-WORLD_SIMULATOR-024` 落地。 |
| DEC-WS-007 | 采用共享 launcher UI schema，由 native 与 web 双端适配渲染 | 继续维持 native/web 双端字段硬编码 | 单点维护字段与文案可显著降低配置漂移风险，且可保持 UI 行为一致性；由 `TASK-WORLD_SIMULATOR-026` 落地。 |
| DEC-WS-008 | 采用 `agent_world_client_launcher` 同一套 egui UI 跨 native/wasm 双目标，并由 `world_web_launcher` 托管 launcher 静态资源 | 继续维护独立 HTML 控制台并与 native 并行演进 | 可彻底消除 UI 双栈分叉，降低维护与验收成本；由 `TASK-WORLD_SIMULATOR-028` 落地。 |
| DEC-WS-009 | launcher wasm 轮询计时切换到 Web 兼容时间实现，并将 Playwright headed 闭环作为回归门禁 | 接受已知 panic 并仅做文档告警 | 该问题会直接导致 Web UI 不可用，必须通过代码修复 + 自动化采证闭环防止回归；由 `TASK-WORLD_SIMULATOR-030` 落地。 |
| DEC-WS-010 | 启动器必填校验按平台分流（Web 排除 native-only binary 必填；native 保持阻断） | 在 Web 端注入伪二进制路径默认值 | 分流更符合字段语义边界，避免伪配置污染与误导；由 `TASK-WORLD_SIMULATOR-032` 落地。 |
| DEC-WS-011 | native 客户端改为“客户端 + 本地 world_web_launcher 服务端”，与 web 客户端共用同一控制面 API | 继续维护 native 直连本地进程 + web API 双路径 | 单一控制面可保证行为一致并降低并行回归成本；由 `TASK-WORLD_SIMULATOR-035` 落地。 |
| DEC-WS-012 | viewer live 采用“runtime 驱动 + simulator 协议兼容适配”的 Phase 1 迁移 | 一次性替换 viewer 协议与前端模型 | 先切 runtime 主驱动可快速降低双轨风险，同时控制改动面与回归成本；由 `TASK-WORLD_SIMULATOR-037` 落地。 |
| DEC-WS-013 | viewer live runtime Phase 2 采用“LLM sidecar + prompt/chat/auth + 动作桥接子集”渐进方案 | 等待 runtime action 全量 1:1 映射后再开放控制面 | 先打通 runtime 的 LLM/chat/prompt 体验可立即消除双套控制断裂，并将动作映射风险限制在可诊断范围内；由 `TASK-WORLD_SIMULATOR-039` 落地。 |
| DEC-WS-014 | viewer live runtime Phase 3 采用“补齐高频动作映射 + 等价回归 + runtime-only 启动”方案 | 保留 simulator fallback 分支与部分映射缺口 | fallback 分支会持续制造双轨行为与回归成本，统一 runtime-only 并补齐映射可把风险收敛到单一可诊断链路；由 `TASK-WORLD_SIMULATOR-041` 落地。 |
| DEC-WS-015 | 在不改变 native 功能语义前提下，优先清理“无读写路径 + 无编译入口引用”的启动器遗留代码 | 维持遗留字段/旧测试文件并通过忽略告警维持现状 | 迁移到统一控制面后遗留代码会持续制造维护歧义；结构化清理可降低后续需求迭代成本；由 `TASK-WORLD_SIMULATOR-051` 落地。 |
