# world-simulator PRD 分册：启动器链上转账

## 目标
- 在启动器链路中提供可操作的链上转账能力，形成“配置-提交-结果反馈”的发行级闭环体验。
- 将转账能力拆分为可追溯条款（PRD-ID、验收、测试）并与主 PRD 同步。

## 范围
- 覆盖启动器转账 UI、`world_chain_runtime` 转账接口、runtime 主 token 账本动作的端到端产品需求。
- 覆盖输入校验、错误可观测、anti-replay（nonce）与测试口径。
- 不覆盖助记词、私钥托管、多签、跨链桥接与手续费市场。

## 接口 / 数据
- 主入口：`doc/world-simulator/prd.md`
- 项目管理：`doc/world-simulator/prd.project.md`
- 追踪主键：`PRD-WORLD_SIMULATOR-004`、`PRD-WORLD_SIMULATOR-005`
- 关键集成点：
  - `doc/world-simulator/launcher/launcher-chain-runtime-decouple-2026-02-28.md`
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world/src/runtime/world/event_processing/action_to_event_core.rs`
  - `crates/agent_world/src/runtime/state/apply_domain_event_main_token.rs`
  - `testing-manual.md`

## 里程碑
- M1：冻结转账请求/响应字段与错误码语义。
- M2：落地链运行时转账提交接口与运行时账本动作。
- M3：落地启动器转账 UI 及回归测试（`test_tier_required`）。
- M4：沉淀转账闭环测试证据并进入发布验收。

## 当前实现状态（2026-03-03）
- 已完成：`world_chain_runtime` 新增 `POST /v1/chain/transfer/submit`，覆盖请求校验、结构化响应与单元测试（对应 `TASK-WORLD_SIMULATOR-006`）。
- 已完成：runtime 主 token 转账动作/事件/状态更新已落地，包含余额约束与 nonce anti-replay（对应 `TASK-WORLD_SIMULATOR-007`）。
- 已完成：启动器新增转账窗口与提交流程（输入校验、状态提示、错误展示，`TASK-WORLD_SIMULATOR-008`）。
- 待完成：启动器-链运行时转账端到端闭环测试与证据沉淀（`TASK-WORLD_SIMULATOR-009`）。

## 风险
- nonce 与状态不同步可能导致重复提交或误拒绝。
- 启动器校验规则与 runtime 规则不一致可能出现“前端放行、后端失败”。
- 转账失败原因若不结构化会降低可诊断性。

## 1. Executive Summary
- Problem Statement: 当前启动器链路仅提供链状态/余额观测与反馈提交，不支持用户直接发起链上资产转账，发行链路缺少核心资产交互闭环。
- Proposed Solution: 在 launcher 链路新增链上转账能力，统一 UI 交互、`world_chain_runtime` 转账接口、runtime 账本动作和安全约束。
- Success Criteria:
  - SC-1: 启动器可完成“填写参数 -> 提交 -> 返回结果/错误原因”闭环。
  - SC-2: `world_chain_runtime` 提供结构化转账接口，成功与失败均可机器解析。
  - SC-3: 运行时具备余额不足、非法账户、nonce 回放拦截。
  - SC-4: 转账链路回归测试进入 `test_tier_required` 并稳定通过。
  - SC-5: 转账改动可追溯到 `PRD-WORLD_SIMULATOR-004/005` 与对应任务。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要不依赖命令行即可完成资产转账。
  - 链路开发者：需要可审计、可复现的转账规则与错误语义。
  - 测试与发布人员：需要闭环测试模板与可复核证据。
- User Stories:
  - PRD-WORLD_SIMULATOR-004: As a 启动器玩家, I want to submit a blockchain transfer in launcher, so that I can move main token balances without external tools.
  - PRD-WORLD_SIMULATOR-005: As a 链路开发者, I want transfer requests to be replay-safe and traceable, so that transfer execution is secure and auditable.
- Acceptance Criteria:
  - AC-1: 启动器提供 `from`、`to`、`amount`、`nonce` 输入与提交按钮，提交后展示成功/失败状态。
  - AC-2: `world_chain_runtime` 提供转账提交接口，返回 `ok`、交易标识/事件标识或结构化错误信息。
  - AC-3: runtime 转账动作校验账户格式、金额范围、nonce 单调与余额充足。
  - AC-4: nonce 重放请求必须被拒绝，且拒绝原因可在 UI 与日志中定位。
  - AC-5: 转账失败不影响其他链路功能（状态查询、余额查询、反馈提交）。
  - AC-6: `test_tier_required` 覆盖参数校验、成功路径、失败路径、回放拒绝。
- Non-Goals:
  - 不实现钱包托管、密钥管理、多签策略与跨链能力。
  - 不实现手续费市场、交易排序策略和复杂 mempool 策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: Playwright Web 闭环、启动器集成测试、链运行时接口契约测试、运行时单元测试。
- Evaluation Strategy:
  - 指标-1：转账提交成功率（合法请求） >= 99%（测试样本口径）。
  - 指标-2：非法请求拒绝准确率（余额不足/nonce 重放/参数非法） = 100%。
  - 指标-3：失败原因可观测率（日志 + UI） = 100%。

## 4. Technical Specifications
- Architecture Overview:
  - 客户端启动器构建转账请求并调用 `world_chain_runtime` HTTP 接口。
  - `world_chain_runtime` 完成输入校验并提交 runtime action。
  - runtime 将 action 转为 domain event 并更新 main token 账户状态。
  - 响应返回启动器 UI，UI 渲染成功信息或拒绝原因。
- Integration Points:
  - `world_game_launcher` 对链运行时配置透传能力。
  - `world_chain_runtime` 转账接口与现有 `status/balances/feedback` 接口并存。
  - runtime 主 token 动作链路：`events`、`action_to_event_core`、`apply_domain_event_main_token`。
- Security & Privacy:
  - 只允许最小必要字段进入转账接口，超出字段忽略或拒绝。
  - 对 `amount/nonce` 做严格数值边界校验并防回放。
  - 返回错误信息应可诊断但不泄露密钥等敏感信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP：文档条款冻结与任务拆解完成。
  - v1.1：链运行时接口 + runtime 账本动作落地。
  - v1.2：启动器 UI 与闭环测试落地。
  - v2.0：交易历史查询与可观测指标增强。
- Technical Risks:
  - 风险-1：请求层与运行时规则不一致导致用户体验分裂。
  - 风险-2：链状态同步延迟影响用户对转账结果的预期。
  - 风险-3：错误信息语义不稳定导致测试难以固化。
