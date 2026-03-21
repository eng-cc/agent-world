# 客户端启动器 Native 遗留代码清理（2026-03-06）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.project.md`

审计轮次: 6

## 1. Executive Summary
- Problem Statement: 启动器 native 已完成“客户端 + 控制面服务”迁移，但代码中仍保留部分无读写路径的历史状态字段与未被编译入口引用的旧测试文件，持续增加维护噪声。
- Proposed Solution: 在不改变 native 运行时行为的前提下，清理已失效遗留代码（状态字段、常量边界与未引用测试资产），并用现有 required 回归验证行为稳定。
- Success Criteria:
  - SC-1: 移除 native 端无读写路径的历史状态字段与初始化逻辑。
  - SC-2: 删除 `oasis7_client_launcher` 中未被编译入口引用的旧测试文件。
  - SC-3: 平台相关常量边界收敛后，不再保留明显的 native 历史残留告警。
  - SC-4: `test_tier_required` 回归全通过，启动器 native/web 功能行为不回退。

## 2. User Experience & Functionality
- User Personas:
  - 启动器开发者：需要长期维护成本可控、代码语义清晰的启动器代码库。
  - 回归测试人员：需要在清理后继续稳定执行 native + wasm 回归。
- User Scenarios & Frequency:
  - 日常维护：每次新增启动器能力前先判断并收敛历史残留（高频）。
  - 发布前回归：每次发布前执行一次清理后稳定性确认（中频）。
- User Stories:
  - PRD-WORLD_SIMULATOR-022: As a 启动器开发者, I want native legacy launcher code to be removed after control-plane unification, so that code ownership and long-term maintenance are cleaner.
- Critical User Flows:
  1. Flow-LAUNCHER-NATIVE-LEGACY-001（遗留识别）:
     `扫描 native 端状态字段与模块引用 -> 识别无读写路径与无编译入口引用资产`
  2. Flow-LAUNCHER-NATIVE-LEGACY-002（安全清理）:
     `删除遗留字段/旧测试文件 -> 保持启动器按钮行为与接口行为不变`
  3. Flow-LAUNCHER-NATIVE-LEGACY-003（回归验证）:
     `执行 launcher native + oasis7_web_launcher + wasm check -> 结果全绿 -> 回写文档状态`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Native 遗留状态清理 | `launcher_started_at/chain_started_at/last_chain_probe_at/chain_running` 等历史状态 | 不新增按钮，不改变现有启动/停止/反馈/转账入口行为 | `legacy_present -> removed -> validated` | 仅删除“无读写路径”字段，不调整行为字段 | 仅开发维护路径可改 |
| 平台常量边界收敛 | native-only 常量与 wasm-only 常量边界 | 不改变 UI 交互，仅收敛编译目标边界 | `mixed_scope -> target_scoped` | 优先按 `cfg(target_arch)` 收敛 | 与运行权限无关 |
| 旧测试资产清理 | 未被 `main.rs` 编译入口引用的历史测试文件 | 不改变现有测试命令，仅移除无效资产 | `stale_asset -> removed` | 仅删除不可达测试文件 | 仅开发维护路径可改 |
- Acceptance Criteria:
  - AC-1: `ClientLauncherApp` 中控制面迁移后无读写路径的 native 历史字段被移除。
  - AC-2: native/wasm 相关常量按目标平台收敛，不再混用无效作用域。
  - AC-3: `oasis7_client_launcher/src/tests.rs`（未被入口引用）被移除。
  - AC-4: 启动器 UI 行为不变：启动/停止、设置、反馈、转账入口保持原语义。
  - AC-5: required 回归通过并可追溯到 `TASK-WORLD_SIMULATOR-051`。
- Non-Goals:
  - 不重构启动器 UI 架构与交互布局。
  - 不改变反馈“链就绪门控”或转账业务语义。
  - 不在本轮引入新的控制面 API。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 清理范围限定在 `oasis7_client_launcher` native 遗留资产。
  - 不触碰 `oasis7_web_launcher` 与 `world_chain_runtime` 业务协议。
  - 通过现有测试矩阵证明“代码清理 != 行为变更”。
- Integration Points:
  - `crates/oasis7_client_launcher/src/main.rs`
  - `crates/oasis7_client_launcher/src/launcher_core.rs`
  - `crates/oasis7_client_launcher/src/main_tests.rs`
  - `crates/oasis7_client_launcher/src/tests.rs`（删除）
  - `doc/world-simulator/prd.md`
  - `doc/world-simulator/project.md`
- Edge Cases & Error Handling:
  - 字段误删风险：仅删除经引用扫描确认不可达字段，避免运行路径字段被误删。
  - 条件编译偏差：收敛 `cfg` 时需确保 native 与 wasm 均可编译。
  - 测试漂移：移除旧测试文件后，保留被入口编译的 `main_tests.rs` 与模块内单测覆盖。
  - 行为回归：若回归失败，优先恢复语义，再细化清理粒度。
- Non-Functional Requirements:
  - NFR-1: 代码清理后不得引入启动器行为退化（按 required 回归定义）。
  - NFR-2: 新增/修改 Rust 文件仍满足单文件 < 1200 行约束。
  - NFR-3: 清理变更应保持可读性，避免引入复杂条件分支。
- Security & Privacy:
  - 本专题不新增数据采集、传输或存储行为。
  - 保持现有控制面与反馈/转账鉴权边界不变。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成 native 遗留字段与旧测试资产清理，并通过 required 回归。
  - v1.1: 评估并继续收敛 launcher 其余低价值历史代码片段。
  - v2.0: 形成 launcher 迁移后“遗留检测 + 清理”常态化规则。
- Technical Risks:
  - 风险-1: 错误判定字段为“遗留”可能导致运行路径回归。
  - 风险-2: `cfg` 收敛不完整导致单目标编译通过、跨目标编译失败。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-022 -> TASK-WORLD_SIMULATOR-050/051 -> `test_tier_required`。
  - 计划验证：
    - `env -u RUSTC_WRAPPER cargo test -p oasis7_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p oasis7_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-NATIVE-LEGACY-001: 先清理“无读写路径 + 无入口引用”的确定性遗留，再做更大重构。理由：风险最低且可快速去噪。
  - DEC-LAUNCHER-NATIVE-LEGACY-002: 复用现有 required 回归，不新增独立测试框架。理由：保证变更可追溯并减少额外维护面。
