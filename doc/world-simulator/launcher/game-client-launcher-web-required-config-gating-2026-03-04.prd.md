# 客户端启动器 Web 必填配置校验分流修复（2026-03-04）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: 启动器 Web 端当前沿用 native 必填校验，错误要求 `launcher_bin` 与 `chain_runtime_bin`，导致浏览器场景出现误报并阻断启动提示。
- Proposed Solution: 将 launcher 必填项与 UI 字段渲染按目标平台分流：native 保持二进制路径必填，wasm/web 排除 native-only 必填与字段展示。
- Success Criteria:
  - SC-1: Web 端不再出现“启动器二进制路径（launcher bin）是必填项”。
  - SC-2: Web 端不再出现“链运行时二进制路径（chain runtime bin）是必填项”。
  - SC-3: Native 端仍保持上述二进制路径必填校验。
  - SC-4: `agent-browser --headed` 闭环页面可加载且状态轮询/启动停止流程可用。

## 2. User Experience & Functionality
- User Personas:
  - 运维人员：通过浏览器管理无 GUI 服务器上的启动器。
  - 启动器开发者：维护同层 UI，但要求平台差异校验准确。
- User Scenarios & Frequency:
  - 每次 Web 启动器打开首页时（高频）不应被 native-only 必填项误阻断。
  - 每次配置校验逻辑变更后（每次发布）执行一次 Web 闭环回归。
- User Stories:
  - As an 运维人员, I want web launcher validation to ignore native-only binary fields, so that browser workflow is not blocked by irrelevant required checks.
  - As a 启动器开发者, I want native validation unchanged, so that desktop binary path safety checks remain enforced.
- Critical User Flows:
  1. Flow-LAUNCHER-WEB-REQ-001（Web 校验）:
     `打开 web launcher -> 加载配置 -> 执行必填校验 -> 不出现 native-only 必填错误`
  2. Flow-LAUNCHER-WEB-REQ-002（Native 校验）:
     `打开 native launcher -> 清空二进制路径 -> 显示必填错误`
  3. Flow-LAUNCHER-WEB-REQ-003（闭环回归）:
     `agent-browser --headed open -> snapshot/screenshot/console -> /api/start + /api/stop`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Web 必填校验分流 | `launcher_bin`、`chain_runtime_bin`（native-only） | Web 端不将其计入必填错误 | `config_loaded -> validated` | 按 `target_arch` 分流 | 浏览器会话只读配置字段 |
| UI 字段可见性分流 | schema `web_visible/native_visible` | 渲染时按目标平台选取字段集合 | `schema -> form_ready` | 保持 schema 顺序渲染 | 与平台角色一致 |
| Native 必填保持 | binary path 字段 | Native 端仍提示二进制路径必填 | `invalid -> blocked` | 必填优先级不变 | 仅桌面端可编辑 |
- Acceptance Criteria:
  - AC-1: wasm/web 编译产物中必填校验不包含 `LauncherBinRequired/ChainRuntimeBinRequired`。
  - AC-2: native 编译产物中上述必填校验保持有效。
  - AC-3: Web 端 UI 字段渲染不展示 binary path 字段。
  - AC-4: `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` 通过。
  - AC-5: `agent-browser --headed` 闭环与 `/api/start` `/api/stop` 回归通过并归档证据。
- Non-Goals:
  - 不重构 `world_web_launcher` API 协议。
  - 不新增启动器配置字段。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不涉及新增 AI 能力。

## 4. Technical Specifications
- Architecture Overview:
  - `agent_world_client_launcher` 以 `cfg(target_arch)` 分流配置校验与字段迭代源。
  - `agent_world_launcher_ui` schema 继续作为字段可见性单一来源。
- Integration Points:
  - `crates/agent_world_client_launcher/src/launcher_core.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_launcher_ui/src/lib.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
- Edge Cases & Error Handling:
  - Web API 返回配置中无 binary path 字段时，不得触发 native-only 必填错误。
  - native 若 binary path 为空，必须继续阻断并提示必填。
  - chain disabled 时链路必填项保持短路逻辑。
- Non-Functional Requirements:
  - NFR-1: Web 校验分流不引入额外请求开销。
  - NFR-2: Native 与 Web 配置校验分支需可编译且不产生行为漂移。
  - NFR-3: Web 闭环回归证据存档于 `output/playwright/`。
- Security & Privacy:
  - 不新增敏感字段曝光，日志仅输出配置问题类型。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: PRD 建模与任务拆解。
  - M2: 校验与渲染分流代码修复。
  - M3: agent-browser 闭环回归与文档收口。
- Technical Risks:
  - 风险-1: 分流条件遗漏导致某端校验退化。
  - 风险-2: 字段渲染源切换后引入 section 空渲染副作用。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-014 -> TASK-WORLD_SIMULATOR-031/032 -> `test_tier_required`。
- Decision Log:
  - DEC-LAUNCHER-WEB-REQ-001: 采用“平台分流校验 + 平台分流字段渲染”方案，而非在 Web 端补默认二进制路径占位值。理由：字段语义属于 native-only，分流更符合职责边界且避免伪配置输入。
