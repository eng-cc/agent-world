# 客户端启动器可用性与体验硬化（2026-03-08）

审计轮次: 1
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-availability-ux-hardening-2026-03-08.prd.project.md`

## 1. Executive Summary
- Problem Statement: 启动器在源码直接运行场景存在默认静态目录失效问题，且 Web 端仍有若干体验与可诊断性缺口（禁用态提示缺失、查询参数未编码、移动端布局拥挤、无效控制台噪声）。
- Proposed Solution: 统一修复 launcher 控制面与客户端的可用性短板，补齐路径回退、状态语义、Web 提示、参数编码与移动端可读性，并以 native/web 一致回归收口。
- Success Criteria:
  - SC-1: 源码直接运行 `world_web_launcher` 时，默认 `viewer_static_dir` 可落到有效目录或返回明确可操作的错误提示，不再出现无提示假启动。
  - SC-2: Web 端在链未就绪/已禁用时必须展示可读原因提示，用户可明确知道反馈/转账/浏览器入口禁用原因。
  - SC-3: explorer/transfer/search 相关查询参数统一完成 URL 编码，包含 `&`、空格、`:`、`?` 的输入不再破坏请求语义。
  - SC-4: 控制面 stop 空操作不覆盖既有错误态，状态机保持“未启动/已停止/失败态”语义稳定。
  - SC-5: Web 端在 390x844 移动视口下配置区可读，关键按钮与状态信息可完整查看与操作。
  - SC-6: Web 控制台默认页不再产生 `favicon.ico 404` 控制台错误噪声。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要稳定可用的启动入口与清晰状态反馈。
  - 运维人员：需要在 Web 控制台快速判断“为什么不能操作”。
  - 启动器开发者：需要 native/web 一致、可诊断、可回归的控制面行为。
- User Scenarios & Frequency:
  - 源码调试场景：开发日常高频（每日多次）直接运行 `target/debug/world_web_launcher` 验证行为。
  - Web 远程运维场景：发布前和日常值守中频（每周多次）通过浏览器执行链/游戏启停与状态检查。
  - 区块链浏览器与转账查询：链就绪后按需高频使用（每次会话 3~20 次查询）。
- User Stories:
  - PRD-WORLD_SIMULATOR-027: As a 启动器玩家/运维人员, I want launcher web control plane to provide robust defaults and explicit UX feedback, so that I can start and diagnose the stack without hidden pitfalls.
- Critical User Flows:
  1. Flow-LAUNCHER-UX-001（源码直跑可用性）:
     `启动 world_web_launcher -> 默认配置加载 -> viewer_static_dir 自动回退到有效目录 -> 成功启动游戏链路`
  2. Flow-LAUNCHER-UX-002（禁用态可解释）:
     `chain_disabled 或 chain_not_ready -> 反馈/转账/浏览器按钮禁用 -> UI 显示禁用原因文案`
  3. Flow-LAUNCHER-UX-003（查询参数鲁棒性）:
     `输入 account_id/search/contract_id（含特殊字符） -> 客户端编码查询参数 -> 控制面正确转发 -> 返回结构化结果`
  4. Flow-LAUNCHER-UX-004（状态语义稳定）:
     `未启动时重复 stop -> 控制面返回 ok 且不覆盖历史失败态 -> UI 状态保持语义一致`
  5. Flow-LAUNCHER-UX-005（移动端可读性）:
     `手机视口打开 launcher web -> 配置字段纵向可读 -> 状态/按钮/日志可正常浏览`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 默认静态目录回退 | `viewer_static_dir` 候选路径列表（bundle/web/.tmp） | 启动时自动选取首个存在目录；无有效目录时返回明确错误 | `idle -> running` 或 `idle -> invalid_config` | 按候选优先级顺序命中 | 本地运维可修改路径 |
| 链未就绪禁用提示 | `chain_status`、`chain_enabled` | 反馈/转账/浏览器按钮禁用时显示对应原因文案 | `disabled/not_started/starting/unreachable -> hint_visible` | 状态优先于按钮可见性 | 只读状态提示 |
| 查询参数编码 | `account_id/action_id/q/contract_id/tx_hash` | 构造请求前执行 URL encoding，避免拼接污染 | `input -> encoded_query -> request` | 统一编码规则（RFC3986 安全子集） | 查询接口只读 |
| stop 空操作语义 | `status/chain_status` 当前态 | stop 在未运行时返回成功但不覆盖错误态；仅记录 no-op 日志 | `* -> same_state`（no-op） | 保留最近有效状态 | 控制面可写操作 |
| 移动端配置布局 | 配置区字段渲染方式 | 字段改为纵向布局（label+input），减少小屏截断 | `form_ready -> readable_mobile_form` | 以字段顺序渲染 | 无权限变化 |
| favicon 噪声抑制 | 默认 favicon 声明 | 页面加载不再触发 `favicon.ico` 404 | `page_load -> console_clean` | 统一静态入口模板 | 无权限变化 |
- Acceptance Criteria:
  - AC-1: `world_web_launcher` 在源码直跑场景（无 bundle `web/`）下可通过默认回退路径启动，或返回可操作错误信息。
  - AC-2: Web 端在 `disabled/not_started/starting/unreachable/config_error` 任一非 `ready` 状态时，必须显示反馈不可用原因。
  - AC-3: `app_process.rs` 与 `app_process_web.rs` 所有 explorer/transfer/search 查询参数构造均使用统一编码函数。
  - AC-4: `/api/stop` 与 `/api/chain/stop` 在 no-op 场景不覆盖 `StartFailed/QueryFailed/Unreachable/ConfigError` 等错误态。
  - AC-5: Playwright 在 390x844 视口截图中，配置区字段可逐项读取且关键控制按钮可见。
  - AC-6: Playwright console 采样不再出现 `favicon.ico 404`。
- Non-Goals:
  - 不在本专题新增新的业务入口（新链功能/新操作窗）。
  - 不在本专题重构 transfer/explorer 业务语义本身。
  - 不在本专题改变 world/runtime 协议。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不涉及 AI 新能力，仅依赖现有 Playwright 与 Rust 回归测试工具链。
- Evaluation Strategy: 不适用（以功能可用性、回归结果与 UI 证据为主）。

## 4. Technical Specifications
- Architecture Overview:
  - 控制面：`world_web_launcher` 负责配置校验、进程编排、状态快照与 API 代理。
  - 客户端：`agent_world_client_launcher` native/web 共用状态模型与查询构造逻辑。
  - UI schema：继续复用 `agent_world_launcher_ui`，仅调整渲染布局与提示逻辑。
- Integration Points:
  - `crates/agent_world/src/bin/world_web_launcher/runtime_paths.rs`
  - `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
  - `crates/agent_world_client_launcher/src/platform_ops.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/launcher_core.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/index.html`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 启动路径不存在：按候选路径继续回退，全部失败时返回结构化 `invalid_config`。
  - 特殊字符查询：编码后发起请求，避免 query 截断与多参数污染。
  - stop no-op：保留当前错误态，仅增加“进程未运行”的日志记录。
  - 移动小屏：布局拥挤时优先保证字段可读与按钮可触达。
  - 浏览器静态资源：默认提供 favicon，避免无意义 404 噪声淹没真实错误。
- Non-Functional Requirements:
  - NFR-1: 源码直跑 `world_web_launcher` 后首次 `/api/start` 成功率达到 100%（前提存在至少一个有效静态目录候选）。
  - NFR-2: 查询参数编码后，含保留字符输入的 explorer/search 请求成功解析率 100%。
  - NFR-3: 390x844 视口下配置区可读性通过人工审查（字段标签与输入框可见率 100%）。
  - NFR-4: 控制面 no-op stop 不引入状态机回退，状态稳定性通过单测/集成回归验证。
- Security & Privacy:
  - 本专题不新增敏感数据采集。
  - 参数编码仅用于传输安全与语义完整性，不改变权限边界。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (UXH-1): 文档建模与任务拆解冻结。
  - v1.1 (UXH-2): 实现路径回退、状态语义修正、禁用态提示、参数编码。
  - v1.2 (UXH-3): 完成移动端布局与 favicon 噪声清理，并执行跨端回归。
- Technical Risks:
  - 风险-1: 静态目录回退策略过宽可能命中错误目录。
  - 风险-2: 参数编码修复若不统一覆盖 native/web，可能产生双端行为漂移。
  - 风险-3: UI 布局调整可能影响桌面端信息密度与操作效率。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-027 | TASK-WORLD_SIMULATOR-063/064 | `test_tier_required` | `./scripts/doc-governance-check.sh` + `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + Playwright headed（桌面 + 390x844）采证 | 启动器控制面可用性、web 体验、native/web 请求一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LAUNCHER-UX-001 | 为静态目录引入“候选路径回退 + 明确报错” | 维持单一路径并要求人工配置 | 源码直跑是高频开发场景，需降低首次失败率。 |
| DEC-LAUNCHER-UX-002 | 在 UI 层明确展示链未就绪禁用原因（含 wasm） | 保持纯禁用态无文案 | 可诊断性优先，减少“按钮灰掉但原因未知”。 |
| DEC-LAUNCHER-UX-003 | 统一 URL 编码函数并在 native/web 全覆盖 | 局部按需修补 | 统一方案可避免后续遗漏与行为漂移。 |
| DEC-LAUNCHER-UX-004 | stop no-op 保持现有错误态，不做状态覆盖 | 无条件置为 stopped/not_started | 保留状态语义有助于运维定位最后一次失败原因。 |
