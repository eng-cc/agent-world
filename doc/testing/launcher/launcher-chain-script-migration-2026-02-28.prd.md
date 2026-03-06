# Agent World: 启动链路脚本迁移（2026-02-28）

审计轮次: 4

- 对应项目管理文档: doc/testing/launcher/launcher-chain-script-migration-2026-02-28.prd.project.md

## 1. Executive Summary
- Problem Statement: 多个运行脚本仍依赖 `world_viewer_live` 旧节点参数链路（`--node-*`/`--topology`/`--reward-runtime-*`），在新启动架构下已失效且错误提示不清晰。
- Proposed Solution: 将日常脚本迁移到 `world_game_launcher`，并对尚未重构完成的长跑脚本提供显式阻断与迁移指引，统一启动口径。
- Success Criteria:
  - SC-1: `run-game-test.sh` 与 `viewer-release-qa-loop.sh` 迁移到 `world_game_launcher` 调用链路。
  - SC-2: `s10-five-node-game-soak.sh` 与 `p2p-longrun-soak.sh` 在旧参数路径下启动前明确失败并给出迁移方向。
  - SC-3: 文档与手册口径同步到“单机优先 `world_game_launcher`、多节点建议 `world_chain_runtime`”。
  - SC-4: 迁移后脚本失败信息可定位，不再出现隐式参数失效。

## 2. User Experience & Functionality
- User Personas:
  - 测试执行者：需要可直接运行且报错清晰的脚本入口。
  - 脚本维护者：需要统一参数口径降低维护分叉。
  - 发布负责人：需要知道哪些流程可执行、哪些被有意阻断。
- User Scenarios & Frequency:
  - 日常回归：每次本地或 CI 手动触发脚本时执行。
  - 版本发布前验证：按手册调用标准化脚本链路。
  - 长跑链路排查：当脚本被阻断时依据提示切换到替代入口。
- User Stories:
  - PRD-TESTING-LAUNCHER-SCRIPT-001: As a 测试执行者, I want everyday scripts to use `world_game_launcher`, so that launch commands match the current runtime architecture.
  - PRD-TESTING-LAUNCHER-SCRIPT-002: As a 脚本维护者, I want legacy longrun scripts to fail fast with actionable guidance, so that broken parameter paths are not silently used.
  - PRD-TESTING-LAUNCHER-SCRIPT-003: As a 发布负责人, I want manual/docs aligned to the new launch entrypoints, so that release checks remain operable.
- Critical User Flows:
  1. Flow-LAUNCH-001: `执行 run-game-test/viewer-release-qa-loop -> world_game_launcher 启动 -> 输出兼容日志`
  2. Flow-LAUNCH-002: `执行长跑脚本 -> 命中旧参数链路 -> 显式阻断并输出迁移提示`
  3. Flow-LAUNCH-003: `查阅 testing-manual -> 按新口径选择 world_game_launcher 或 world_chain_runtime`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 日常脚本迁移 | `--scenario`、`--live-bind`、`--web-bind`、`--viewer-*`、`--chain-*` | 调用 `world_game_launcher` 统一入口 | `legacy -> migrated -> verified` | 单机链路优先迁移 | 脚本维护者可修改，执行者可运行 |
| 长跑脚本阻断 | 阻断条件、提示文案、替代入口 | 启动前检测旧参数并失败退出 | `legacy -> blocked-with-guidance` | 先阻断错误路径，再给替代建议 | 维护者定义阻断策略 |
| 文档口径收口 | 手册条目、脚本调用示例、状态说明 | 同步更新文档并标明可执行边界 | `draft -> aligned` | 手册与脚本口径必须一致 | 文档维护者审批 |
| 兼容日志/产物 | 日志路径、关键产物命名 | 尽量保持命名兼容避免下游破坏 | `unchanged -> validated` | 优先保持现有产物路径 | 测试维护者核验 |
- Acceptance Criteria:
  - AC-1: 两条日常脚本完成 `world_game_launcher` 迁移并可执行。
  - AC-2: 两条长跑脚本对旧链路显式阻断，提示包含迁移方向。
  - AC-3: 手册与专题文档反映新启动口径与暂不可执行边界。
  - AC-4: 未重构的长跑多节点闭环不在本轮硬实现范围。
  - AC-5: 文档与任务状态可在项目文档/devlog 追溯。
- Non-Goals:
  - 不在本轮重写 S10/P2P 为完整 `world_chain_runtime` 多节点闭环。
  - 不恢复 `world_viewer_live` 内嵌节点参数兼容。
  - 不扩展本任务之外的新脚本编排框架。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题聚焦脚本与启动链路迁移，不涉及 AI 推理策略改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 启动入口分层为“单机场景 `world_game_launcher` + 多节点场景 `world_chain_runtime`”，脚本按场景选择入口并对失效链路 fail-fast。
- Integration Points:
  - `scripts/run-game-test.sh`
  - `scripts/viewer-release-qa-loop.sh`
  - `scripts/s10-five-node-game-soak.sh`
  - `scripts/p2p-longrun-soak.sh`
  - `crates/agent_world/src/bin/world_game_launcher.rs`
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 脚本仍传入 `world_viewer_live --node-*`：立即失败并提示替代入口。
  - 自动化依赖旧日志进程名：尽量保留日志产物命名兼容并在文档标注差异。
  - 长跑流程临时不可执行：阻断信息必须给出下一步迁移方向（`world_chain_runtime` 多节点编排）。
  - 参数组合错误：由脚本前置校验输出可定位错误文本。
- Non-Functional Requirements:
  - NFR-LAUNCH-1: 失效参数路径失败提示需在一次执行中可理解且可操作。
  - NFR-LAUNCH-2: 日常脚本迁移不增加额外人工步骤。
  - NFR-LAUNCH-3: 手册与脚本实际行为保持 0 冲突。
  - NFR-LAUNCH-4: 兼容日志命名尽量维持下游消费稳定性。
- Security & Privacy: 仅调整启动参数与脚本路径，不新增敏感数据采集。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (LAUNCHMIG-1): 完成设计与项目文档建档。
  - v1.1 (LAUNCHMIG-2): 日常脚本迁移到 `world_game_launcher`。
  - v2.0 (LAUNCHMIG-3): 长跑脚本加入显式阻断与迁移提示。
  - v2.1 (LAUNCHMIG-4): 手册与状态文档收口。
- Technical Risks:
  - 风险-1: 旧自动化依赖日志/进程名，迁移后可能出现解析偏差。
  - 风险-2: 长跑脚本阻断会造成短期“不可执行”感知，需要清晰迁移路线。
  - 风险-3: 文档口径更新不完整会导致使用者误走旧参数链路。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LAUNCHER-SCRIPT-001 | LAUNCHMIG-1/2 | `test_tier_required` | 日常脚本启动参数与执行路径检查 | 本地/发布前日常脚本链路 |
| PRD-TESTING-LAUNCHER-SCRIPT-002 | LAUNCHMIG-2/3 | `test_tier_required` | 长跑脚本阻断与提示文案校验 | 长跑任务触发安全性 |
| PRD-TESTING-LAUNCHER-SCRIPT-003 | LAUNCHMIG-3/4 | `test_tier_required` | 手册/项目文档/devlog 一致性检查 | 运行指引可用性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LAUNCH-001 | 日常脚本统一迁移 `world_game_launcher` | 继续分散在 `world_viewer_live` 旧参数链路 | 统一入口更易维护且与当前架构一致。 |
| DEC-LAUNCH-002 | 长跑脚本先显式阻断并提示迁移 | 继续尝试兼容失效参数 | fail-fast 能避免误用和隐式错误。 |
| DEC-LAUNCH-003 | 手册同步标注替代入口 | 仅改脚本不改文档 | 文档不更新会放大执行偏差风险。 |

## 原文约束点映射（内容保真）
- 原“目标（迁移旧参数链路、保持可执行、显式阻断）” -> 第 1 章 Problem/Solution/SC。
- 原“范围/非目标” -> 第 2 章 AC 与 Non-Goals。
- 原“接口/数据（新调用口径与阻断口径）” -> 第 2 章规格矩阵 + 第 4 章 Integration。
- 原“里程碑 M1~M3” -> 第 5 章 Phased Rollout（LAUNCHMIG-1~4）。
- 原“风险与缓解” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
