# Viewer Live runtime/world 接管 Phase 1（2026-03-04）

审计轮次: 5
- 对应项目管理文档: doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.project.md

## 1. Executive Summary
- Problem Statement: `world_viewer_live` 当前主执行链路仍使用 `simulator::WorldKernel`，而 `runtime::World` 已承载核心玩法规则，导致 live 体验与规则层长期双轨。
- Proposed Solution: 在不改 Viewer 协议的前提下新增 runtime 驱动 live server（Phase 1），让 `world_viewer_live` 可通过开关切换到 `runtime::World`，并通过兼容适配输出现有 `WorldSnapshot/WorldEvent`。
- Success Criteria:
  - SC-1: `world_viewer_live` 支持 `--runtime-world` 启动 runtime 驱动链路。
  - SC-2: runtime 模式下 Viewer 仍消费现有协议（`WorldSnapshot/WorldEvent`）并保持基础交互可用。
  - SC-3: 至少完成 `AgentRegistered/AgentMoved/ResourceTransferred/ActionRejected` 四类事件适配。
  - SC-4: `test_tier_required` 命令通过并可追溯到 `PRD-WORLD_SIMULATOR-016`。

## 2. User Experience & Functionality
- User Personas:
  - 玩法架构开发者：希望 live 行为更接近 runtime 真实规则执行语义。
  - Viewer 体验开发者：希望不修改前端协议即可观察 runtime 驱动世界。
  - 回归测试人员：希望新链路可通过现有 required 测试口径验证。
- User Scenarios & Frequency:
  - 日常本地联调：开发者以 `--runtime-world` 启动 live viewer 观察规则层行为。
  - 发布前回归：测试人员验证 simulator/runtime 两模式都可启动且基础控制可用。
- User Stories:
  - As a 玩法架构开发者, I want world_viewer_live to run on runtime/world, so that live behavior is aligned with production rule semantics.
  - As a Viewer 开发者, I want runtime mode to keep protocol compatibility, so that front-end code does not need immediate rewrite.
- Critical User Flows:
  1. Flow-VIEWER-RUNTIME-001（runtime 模式启动）:
     `world_viewer_live --runtime-world -> 启动 ViewerLive RuntimeServer -> 客户端 Hello/Subscribe/RequestSnapshot`
  2. Flow-VIEWER-RUNTIME-002（播放推进）:
     `Play/Step 控制 -> runtime::World 提交脚本动作并 step -> 产生日志事件 -> 适配为 simulator event -> 推送 Event/Snapshot`
  3. Flow-VIEWER-RUNTIME-003（兼容回退）:
     `未传 --runtime-world -> 保持原 simulator live server 链路`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| runtime live 启动开关 | `--runtime-world` | 启用 runtime 驱动 server；未启用保持旧链路 | `simulator_mode <-> runtime_mode`（由启动参数确定） | 启动时一次性判定 | 本地启动参数控制 |
| 事件兼容适配 | runtime `DomainEvent` -> simulator `WorldEventKind` | 播放/单步后推送映射事件 | `runtime_event -> mapped_event -> client_render` | 保持事件序列单调递增 | 只读推送 |
| 快照兼容适配 | runtime `WorldState` -> simulator `WorldSnapshot` | `RequestSnapshot` 或事件后更新快照 | `state_change -> snapshot_emit` | 快照时间戳跟随 runtime `state.time` | 只读推送 |
- Acceptance Criteria:
  - AC-1: 新增 `crates/agent_world/src/viewer/runtime_live.rs`，实现 runtime 驱动 live server。
  - AC-2: `world_viewer_live` 新增 `--runtime-world` 参数并正确接线。
  - AC-3: runtime 模式下 `ViewerRequest::Hello/Subscribe/RequestSnapshot/Control` 基础闭环可用。
  - AC-4: runtime 事件至少映射四类：`AgentRegistered`、`AgentMoved`、`ResourceTransferred`、`ActionRejected`。
  - AC-5: simulator 默认模式行为保持不变（未传 `--runtime-world`）。
  - AC-6: 命令通过：`env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_viewer_live` 与 `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_viewer_live`。
- Non-Goals:
  - 不在 Phase 1 完成 runtime 全量事件 1:1 映射。
  - 不在 Phase 1 改造 `agent_world_viewer` 前端数据结构。
  - 不在 Phase 1 移除 simulator live server 路径。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 新增 runtime live server，内部维护 `runtime::World` 与脚本驱动器。
  - 通过兼容适配层将 runtime 状态/事件转换为 simulator 协议对象。
  - `world_viewer_live` 通过 CLI 选择 runtime 或 simulator server 实例。
- Integration Points:
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world/src/viewer/mod.rs`
  - `crates/agent_world/src/viewer/runtime_live.rs`
  - `crates/agent_world/src/viewer/protocol.rs`
  - `crates/agent_world_viewer/src/main.rs`
- Edge Cases & Error Handling:
  - runtime 模式收到暂未支持的请求（如 prompt control/chat）时返回结构化错误，不 panic。
  - runtime 模式事件未覆盖映射时降级为 `ActionRejected::RuleDenied` 或忽略（按可诊断优先）。
  - 仅 1 个位置或无可移动目标时，脚本驱动不应产生非法动作。
  - `--runtime-world` 与 `--llm` 同时启用时（Phase 1 未支持）需显式报错并退出。
- Non-Functional Requirements:
  - NFR-1: runtime 模式启动可监听端口时间 `p95 <= 2s`（本地环境）。
  - NFR-2: `Step` 单次请求在无外部依赖时 `p95 <= 100ms`（本地环境、默认场景）。
  - NFR-3: 新增 Rust 文件行数 <= 1200。
- Security & Privacy:
  - 仅本地控制端口暴露，不新增远程鉴权面。
  - 日志输出不得包含凭据或密钥字段。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: 文档建模与任务拆解。
  - M2: runtime live server + CLI 接线 + 核心映射落地（Phase 1）。
  - M3: 完成 required 测试与文档收口。
- Technical Risks:
  - 风险-1: runtime 到 simulator 的兼容映射不完整导致部分 UI 文案退化。
  - 风险-2: 双模式并存期间可能引入行为认知偏差，需要手册明确开关语义。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-016 -> TASK-WORLD_SIMULATOR-036/037 -> `test_tier_required`。
- Decision Log:
  - DEC-WS-012: 采用“先协议兼容适配、后语义全量替换”的分阶段迁移方案；否决“直接一次性替换 viewer 协议与前端模型”。依据：可在当前迭代内快速把 live 主驱动切到 runtime，同时把回归面控制在可执行范围。
