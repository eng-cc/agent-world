# 客户端启动器区块链浏览器视觉与交互优化（2026-03-08）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 启动器区块链浏览器已覆盖查询能力，但当前信息呈现偏“日志行”，视觉层级弱、列表与详情切换成本高，导致高频排障与核查效率偏低。
- Proposed Solution: 在不改协议的前提下，优化浏览器面板 UI 结构与交互流程，补齐概览卡片、状态徽标、分页/筛选可见性、清空与跳转动作，提升 native/web 一致的读写效率。
- Success Criteria:
  - SC-1: 浏览器概览区具备可辨识的信息层级（链高度、节点信息、交易状态计数）且 native/web 行为一致。
  - SC-2: 区块与交易视图支持“列表-详情”并行浏览，点击列表后详情更新可见延迟 <= 1 次轮询周期。
  - SC-3: 交易/搜索/资产相关筛选提供“应用 + 清空”双动作，误筛选恢复步骤 <= 1 次点击。
  - SC-4: 请求中/链未就绪状态在浏览器窗口内可见，不再依赖主面板日志理解当前可操作性。
  - SC-5: `agent_world_client_launcher` native 测试与 wasm `cargo check` 回归通过。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要快速确认交易是否上链与最终状态。
  - 运维/测试人员：需要在一个窗口内高效完成筛选、定位、复核。
  - 启动器维护者：需要保持 native/web 同源 UI，同时降低后续可维护成本。
- User Scenarios & Frequency:
  - 交易核对：每次转账后 1~5 次（高频）。
  - 版本回归排障：每个候选版本至少 1 次（中频）。
  - 线上问题定位：按需触发，通常集中在失败/超时批次（中频）。
- User Stories:
  - PRD-WORLD_SIMULATOR-028: As a 启动器玩家/运维人员, I want a clearer and faster explorer UI, so that I can inspect chain state and locate problematic transactions with fewer interactions.
- Critical User Flows:
  1. Flow-LAUNCHER-EXPLORER-UX-001（快速概览）:
     `打开浏览器 -> 查看高度/状态计数卡片 -> 判断链与交易健康度`
  2. Flow-LAUNCHER-EXPLORER-UX-002（列表到详情）:
     `切到区块或交易列表 -> 点击一条记录 -> 在同页详情区域查看完整字段`
  3. Flow-LAUNCHER-EXPLORER-UX-003（筛选恢复）:
     `输入筛选条件 -> 查询 -> 发现误筛选 -> 点击清空 -> 恢复默认列表`
  4. Flow-LAUNCHER-EXPLORER-UX-004（跨视图跳转）:
     `在区块/合约/地址视图点击 tx_hash -> 自动跳转交易视图并触发详情查询`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 概览信息层级优化 | `latest/committed/network height`、`transfer counters`、`node/world` | 打开窗口即展示；点击刷新同步更新 | `loading -> ready/failed` | counters 保持后端聚合语义 | 查询只读 |
| 状态徽标可视化 | `accepted/pending/confirmed/failed/timeout` | 列表项与详情区显示状态徽标颜色 | `status` 由接口值驱动 | 不改变原状态枚举 | 查询只读 |
| 区块/交易双栏阅读 | `list items` + `selected detail` | 点击列表项更新右侧详情；保持当前筛选条件 | `none -> selected` | 列表顺序沿用接口返回（desc） | 查询只读 |
| 筛选与恢复动作 | `account/status/action/query` 等输入 | 提供 `Apply/Reset`，重置恢复默认筛选并刷新 | `filtered -> default` | 重置时 cursor 回到 0 | 查询只读 |
| 请求状态可见性 | `web_request_inflight`、`chain_ready` | 顶部显示“请求中/不可用”提示，不阻塞后续操作 | `idle <-> inflight` | 仅展示，不改变轮询节奏 | 查询只读 |
- Acceptance Criteria:
  - AC-1: 浏览器窗口展示结构化概览（至少包含高度、节点标识、交易状态计数）并具备视觉分组。
  - AC-2: Blocks/Txs 视图支持列表与详情并行展示，点击列表项后详情区域可即时更新。
  - AC-3: Txs/Search/Assets/Mempool 至少包含一处“清空筛选/恢复默认”入口。
  - AC-4: 交易状态在列表或详情中具备可区分视觉标记（不止纯文本）。
  - AC-5: 请求中与链未就绪状态在浏览器窗口内有显式提示。
  - AC-6: `PRD-WORLD_SIMULATOR-028` 可追溯到 `TASK-WORLD_SIMULATOR-067/068` 与 `test_tier_required` 回归命令。
- Non-Goals:
  - 不新增 explorer 后端 API 或字段。
  - 不在本轮重构 transfer/explorer 状态机语义。
  - 不引入新的前端技术栈（保持 egui 同源实现）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 能力。

## 4. Technical Specifications
- Architecture Overview:
  - UI 层：`agent_world_client_launcher` explorer window 调整信息结构与交互动作。
  - 接口层：继续复用 `/api/chain/explorer/*`，不改控制面与 runtime 协议。
  - 跨端：native/wasm 共用同一 Rust UI 渲染逻辑。
- Integration Points:
  - `crates/agent_world_client_launcher/src/explorer_window.rs`
  - `crates/agent_world_client_launcher/src/explorer_window_p1.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
- Edge Cases & Error Handling:
  - 链未就绪：窗口内显示不可用提示，避免“空白窗口 + 无解释”。
  - 请求进行中：显示 `inflight` 状态并保持按钮行为可预测。
  - 空列表：使用空态文案替代空白区域。
  - 非法筛选输入：沿用已有结构化错误与日志输出，不吞错。
  - 跨 tab 跳转：设置 query 输入后自动触发目标详情刷新，避免用户二次点击。
- Non-Functional Requirements:
  - NFR-1: 界面改造后不新增 explorer 请求频率（仍保持默认 1s 轮询策略）。
  - NFR-2: native 与 wasm 在 tab、筛选、详情跳转行为一致率 100%。
  - NFR-3: `explorer_window.rs`、`explorer_window_p1.rs` 单文件长度仍控制在 1200 行约束内（必要时拆分）。
  - NFR-4: `agent_world_client_launcher` 单元测试与 wasm 编译检查通过。
- Security & Privacy:
  - 仅变更展示层，不新增敏感信息暴露面。
  - 错误展示继续采用结构化 `error_code + error` 语义。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成专题 PRD 建模与任务拆解。
  - v1.1: 完成 explorer 窗口视觉层级与交互动作改造。
  - v1.2: 完成 native/wasm 回归并文档收口。
- Technical Risks:
  - 风险-1: UI 调整过大导致操作路径变化，引入回归。
  - 风险-2: 文件体量持续增长，后续维护复杂度上升。
  - 风险-3: 视觉增强若不克制，可能影响信息密度和可读性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-028 -> TASK-WORLD_SIMULATOR-067/068 -> `test_tier_required`。
  - 计划验证命令:
    - `./scripts/doc-governance-check.sh`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`
    - `env -u RUSTC_WRAPPER cargo fmt --all`
- Decision Log:
  - DEC-LAUNCHER-EXPLORER-UX-001: 采用“信息分组 + 状态徽标”而非继续纯文本行堆叠。理由：降低阅读成本并提升异常状态可见性。
  - DEC-LAUNCHER-EXPLORER-UX-002: 保持接口与状态机不变，只优化交互层。理由：降低回归风险并保证改造聚焦。
  - DEC-LAUNCHER-EXPLORER-UX-003: 优先添加“清空筛选/恢复默认”动作。理由：这是高频误操作恢复路径，投入小收益高。
