# world-simulator PRD

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
- 追踪主键: `PRD-WORLD_SIMULATOR-xxx`
- 测试与发布参考: `testing-manual.md`
- 分册索引:
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`（PRD-WORLD_SIMULATOR-001/002）
  - `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`（PRD-WORLD_SIMULATOR-002/003）
  - `doc/world-simulator/prd/quality/experience-trend-tracking.md`（PRD-WORLD_SIMULATOR-003）
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`（PRD-WORLD_SIMULATOR-004/005）

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2 (2026-03-03): 完成启动器链上转账需求建模与任务拆解。
- M3: 完成 PRD 分册结构落地，主入口仅保留总览与导航。
- M4: 完成链运行时转账 API、运行时账本动作、启动器交互与闭环验收。

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
- Acceptance Criteria:
  - AC-1: world-simulator PRD 覆盖场景、Viewer、LLM、启动器四条主线。
  - AC-2: world-simulator project 文档维护任务拆解与状态。
  - AC-3: 与 `doc/world-simulator/scenario/scenario-files.md`、`doc/world-simulator/viewer/viewer-web-closure-testing-policy.md` 等专题文档一致。
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
  - `doc/world-simulator/scenario/scenario-files.md`
  - `doc/world-simulator/viewer/viewer-web-closure-testing-policy.md`
  - `doc/world-simulator/launcher/game-unified-launcher-2026-02-27.md`
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`
  - `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`
  - `doc/world-simulator/prd/quality/experience-trend-tracking.md`
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`
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
- Non-Functional Requirements:
  - 性能目标:
    - NFR-1: 启动器链状态探针刷新周期 <= 1s，状态可见延迟 <= 2s。
    - NFR-2: 本地链路下转账提交 API `p95` 响应时间 <= 500ms。
  - 兼容性目标:
    - NFR-3: Launcher/Web 闭环流程在 Linux/macOS 开发环境可执行并产出一致证据结构。
    - NFR-8: Viewer 2D/3D 默认渲染在 Linux native + Web 环境可稳定启动，且不出现粉紫回退屏。
    - NFR-9: 复用同一 `--out-dir` 连续执行 `build-game-launcher-bundle.sh`（含“上一次 bundle 仍运行”场景）时，打包成功率需为 100%（`test_tier_required`）。
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

- Decision Log:

| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WS-001 | 采用 Web-first 作为默认 UI 闭环链路，native 抓帧仅 fallback | 以 native 图形链路为默认 | 与 `testing-manual.md` 的 S6 约束一致，能提升复现稳定性与自动化覆盖。 |
| DEC-WS-002 | 主 PRD 保持总览，专题细节下沉到 `doc/world-simulator/prd/*` 分册 | 所有条款继续堆叠在主 PRD | 主入口可读性和变更追踪效率更高，且满足单文档长度约束。 |
| DEC-WS-003 | 反馈分布式提交在 `Connection refused` 时强制本地回落并保留错误签名 | 远端失败直接报错终止，不落盘 | 可确保证据不丢失，便于回归和线上诊断；已被 `TASK-WORLD_SIMULATOR-012` 验证。 |
| DEC-WS-004 | 保留 `TonyMcMapFace` 默认色调映射并显式启用 `bevy/tonemapping_luts` 依赖 | 改默认 tonemapping 或运行时静默降级 | 保持既有视觉基线，同时消除 native 粉紫回退与不可交互回归；已由 `TASK-WORLD_SIMULATOR-020` 的 native 抓帧与 viewer 回归验证。 |
| DEC-WS-005 | `build-game-launcher-bundle.sh` 在 copy 前删除目标二进制，规避运行中覆盖 `Text file busy` | 保持直接 `cp` 覆写（遇占用即失败） | Linux 上删除运行中可执行文件不会中断现有进程，可确保同一路径重建 bundle 不进入半更新态；已由 `TASK-WORLD_SIMULATOR-022` 通过“运行中重复打包”回归验证。 |
