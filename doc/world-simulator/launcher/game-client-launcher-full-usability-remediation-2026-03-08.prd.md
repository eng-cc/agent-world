# 客户端启动器全量可用性收口修复（2026-03-08）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 启动器在最近一轮 UI/UX 优化后仍存在残余可用性风险：配置可能被轮询状态回写、全局单 in-flight 造成高频交互串行、反馈窗口会被链状态切换强制关闭、顶栏在窄屏下拥挤、转账历史过滤缺少一键重置。
- Proposed Solution: 保持现有功能边界不变，在客户端状态管理与交互层完成收口修复：配置防回写、请求并发域拆分、反馈草稿保护、顶栏响应式布局、过滤重置入口。
- Success Criteria:
  - SC-1: 本地已编辑但未提交的配置不会被 `/api/state` 快照覆盖，直到用户显式提交或与远端配置一致。
  - SC-2: 状态轮询、控制动作、反馈、转账、浏览器查询采用独立 in-flight 域，互不阻塞。
  - SC-3: 反馈窗口在链未就绪时不被强制关闭，已输入草稿保持。
  - SC-4: 顶栏在 390x844 视口下自动换行，状态/语言控件可读可操作。
  - SC-5: 转账历史过滤支持“应用 + 清空”，一键恢复默认查询。
  - SC-6: 主 PRD 启动器条款保持唯一且可追溯（AC 编号无重复、集成点无重复路径）。
  - SC-7: `agent_world_client_launcher` 源码满足单文件 <=1200 行治理约束，不引入功能回归。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要连续操作时界面稳定、可预期。
  - 运维人员：需要在窄屏和高频排障场景下快速读取状态并操作。
  - 启动器维护者：需要 native/web 一致的请求并发语义与可回归行为。
- User Scenarios & Frequency:
  - 高频状态核查与查询：每次会话多次（10~50 次）。
  - 参数编辑与启动：每次会话 1~5 次。
  - 反馈提交：低频但对草稿安全性敏感。
- User Stories:
  - PRD-WORLD_SIMULATOR-029: As a 启动器玩家/运维人员, I want launcher interactions to remain stable under polling and continuous operations, so that edits and high-frequency actions are not interrupted or silently dropped.
- Critical User Flows:
  1. Flow-LAUNCHER-USABILITY-001（配置防回写）:
     `打开高级配置 -> 编辑字段 -> 后台轮询 /api/state -> 本地草稿保持不被覆盖 -> 点击启动后与远端配置收敛`
  2. Flow-LAUNCHER-USABILITY-002（并发域拆分）:
     `触发 explorer/transfer 查询 -> 同时保持状态轮询 -> 查询结果返回后各自更新，不互相跳过`
  3. Flow-LAUNCHER-USABILITY-003（反馈草稿保护）:
     `打开反馈窗口并输入草稿 -> 链状态从 ready 变为 starting/not_started -> 窗口保持打开且草稿不丢失`
  4. Flow-LAUNCHER-USABILITY-004（窄屏顶栏）:
     `390x844 打开 launcher -> 顶栏自动换行展示标题/状态/语言 -> 无关键字段截断`
  5. Flow-LAUNCHER-USABILITY-005（过滤恢复）:
     `转账历史输入过滤并应用 -> 点击清空过滤 -> 恢复默认历史列表请求`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 配置防回写 | `config`、`config_dirty`、`remote_config_pending` | 本地编辑后标记 dirty；快照只更新状态/日志，不覆盖本地配置 | `clean -> dirty -> synced` | 当快照配置与本地配置一致时自动回到 `synced` | 本地会话可编辑 |
| 请求并发域拆分 | `state/control/feedback/transfer_submit/transfer_query/explorer` in-flight | 各请求仅阻塞同域请求，允许跨域并发 | `idle <-> inflight(domain)` | 轮询仅受 `state` 域控制 | 控制操作可写，其余查询只读 |
| 反馈窗口草稿保护 | `feedback_window_open`、`feedback_draft` | 链未就绪时仅禁用提交，不强制关窗 | `open + disabled_submit` | 草稿按会话内存保持 | 链就绪前禁止提交 |
| 顶栏响应式 | 顶栏标题/状态/语言控件 | 使用 wrapped 布局自动换行 | `single_row/multi_row` | 窄屏优先保证状态可见 | 无权限变化 |
| 转账过滤重置 | `history_account_filter`、`history_action_filter` | 新增“清空过滤”并触发刷新 | `filtered -> default` | 清空后按默认 limit 与排序加载 | 查询只读 |
- Acceptance Criteria:
  - AC-1: `apply_web_snapshot` 在本地配置 dirty 且快照配置不同的情况下，不得覆盖 `self.config`。
  - AC-2: `poll_process` 的状态轮询不得因为 transfer/explorer/feedback 请求而停止。
  - AC-3: 反馈窗口在 `ChainRuntimeStatus != Ready` 时保持打开，提交按钮禁用且提示原因。
  - AC-4: 顶栏渲染在窄视口下不出现核心状态信息丢失。
  - AC-5: 转账历史区提供“清空过滤”并在点击后触发刷新。
  - AC-6: native 测试与 wasm `cargo check` 通过。
  - AC-7: 主 PRD 中与启动器相关 AC 编号保持连续唯一，且集成点列表无重复路径。
  - AC-8: `crates/agent_world_client_launcher/src/main.rs` 与 `crates/agent_world_client_launcher/src/explorer_window.rs` 行数均 <=1200。
- Non-Goals:
  - 不新增链协议字段与后端 API。
  - 不重构 transfer/explorer 业务状态机。
  - 不调整 launcher 功能信息架构（仅修复交互与状态语义）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 能力。

## 4. Technical Specifications
- Architecture Overview:
  - 客户端维护“配置编辑态”与“服务端快照态”的最小分离，避免轮询回写。
  - 请求并发控制由“全局布尔”改为“按域布尔”，降低无关请求互斥。
  - UI 层保留现有入口，仅调整窗口开关策略与窄屏布局。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/feedback_window.rs`
  - `crates/agent_world_client_launcher/src/feedback_window_web.rs`
  - `crates/agent_world_client_launcher/src/transfer_window.rs`
  - `crates/agent_world_client_launcher/src/explorer_window.rs`
  - `crates/agent_world_client_launcher/src/main_tests.rs`
- Edge Cases & Error Handling:
  - 快照与本地配置冲突：保留本地 dirty 配置并持续更新状态/日志。
  - 同域重复请求：继续拒绝并提供明确日志，不跨域拒绝。
  - 反馈窗口打开时链掉线：不关闭窗口，提交失败给出结构化提示。
  - 转账过滤清空后无数据：显示空态文案，不报错。
- Non-Functional Requirements:
  - NFR-1: 状态轮询周期维持既有 1s，不新增轮询风暴。
  - NFR-2: 请求并发域拆分后，不允许出现“跨域操作被无关请求阻塞”。
  - NFR-3: 改造后 `agent_world_client_launcher` 单文件行数仍满足既有约束。
  - NFR-4: native 与 wasm 行为一致性保持 100%。
- Security & Privacy:
  - 本专题仅改客户端状态管理与 UI，不新增敏感数据采集或外发。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (USRM-1): 文档建模、任务拆解与主文档树挂载。
  - v1.1 (USRM-2): 配置防回写 + 请求并发域拆分。
  - v1.2 (USRM-3): 反馈窗口草稿保护 + 顶栏响应式 + 转账过滤重置。
- Technical Risks:
  - 风险-1: 并发域拆分若清理标志位不完整，可能导致“永久 busy”。
  - 风险-2: 配置 dirty 语义若边界不清，可能引入“远端变更不可见”。
  - 风险-3: 顶栏布局调整可能影响桌面端信息密度。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-029 | TASK-WORLD_SIMULATOR-069/070/071/089/090 | `test_tier_required` | `./scripts/doc-governance-check.sh` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + `wc -l crates/agent_world_client_launcher/src/main.rs crates/agent_world_client_launcher/src/explorer_window.rs` | 启动器配置稳定性、高频交互并发性、反馈与转账窗口可用性、文档追溯一致性与代码维护可持续性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LAUNCHER-USRM-001 | 本地配置 dirty 时禁止快照覆盖配置对象 | 每次轮询直接全量替换配置 | 避免用户编辑期间被回写导致“改不动”体验。 |
| DEC-LAUNCHER-USRM-002 | 请求并发按功能域拆分 | 保持全局单 in-flight | 避免无关请求互相阻塞并降低“跳过操作”概率。 |
| DEC-LAUNCHER-USRM-003 | 反馈窗口保持打开，仅禁用提交 | 链未就绪时强制关窗 | 保护低频但高成本输入，降低草稿丢失风险。 |
