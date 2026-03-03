# Agent World: 基于 world_chain_runtime 的长跑脚本可用化（2026-02-28）

## 1. Executive Summary
- Problem Statement: 长跑脚本依赖已下线的 `world_viewer_live --node-*` 链路，导致脚本无法稳定执行，自动化门禁与产物输出被阻断。
- Proposed Solution: 将 S10 与 P2P 长跑脚本统一迁移到 `world_chain_runtime` 多节点编排，改用 `/v1/chain/status` 与 `/v1/chain/balances` 采样，并保留既有产物与失败定位契约。
- Success Criteria:
  - SC-1: `scripts/s10-five-node-game-soak.sh` 与 `scripts/p2p-longrun-soak.sh` 均可基于 `world_chain_runtime` 正常运行。
  - SC-2: 脚本持续输出 `run_config.json`、`timeline.csv`、`summary.json`、`summary.md`（失败时含 `failures.md`）。
  - SC-3: 核心门禁覆盖共识推进、连接健康、reward mint 记录与高度滞后。
  - SC-4: P2P chaos 注入能力在新链路下继续可用。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：需要可重复执行的长跑脚本与可比较产物。
  - 发布负责人：需要脚本作为发布前稳定性证据来源。
  - 基础设施维护者：需要统一启动口径并降低旧参数兼容负担。
- User Scenarios & Frequency:
  - 每周/每日 soak 回归：批量执行 S10/P2P 长跑任务。
  - 发布候选验证：执行长跑脚本并审阅 summary 与失败报告。
  - 故障复盘：根据 timeline 与 failures 快速定位异常节点。
- User Stories:
  - PRD-TESTING-LONGRUN-SOAK-001: As a 测试维护者, I want longrun scripts to run on `world_chain_runtime`, so that soak regression is executable again.
  - PRD-TESTING-LONGRUN-SOAK-002: As a 发布负责人, I want stable status/balances sampling and summary artifacts, so that gate decisions remain auditable.
  - PRD-TESTING-LONGRUN-SOAK-003: As a 脚本维护者, I want chaos injection and runtime controls preserved, so that legacy automation contracts keep working.
- Critical User Flows:
  1. Flow-SOAK-001: `启动多实例 world_chain_runtime -> 建立 validator/gossip 拓扑 -> 进入采样循环`
  2. Flow-SOAK-002: `定时调用 /v1/chain/status 与 /v1/chain/balances -> 汇总 timeline/summary -> 生成门禁结论`
  3. Flow-SOAK-003: `P2P 脚本注入 chaos -> 记录恢复与失败信息 -> 输出 failures.md`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 多节点 runtime 编排 | `--node-id --world-id --status-bind --node-role --node-tick-ms`、`--node-validator`、`--node-gossip-*` | 脚本按节点配置启动并维持运行时长 | `booting -> running -> stopping` | 先启动主节点后拉起 peers，保留 startup grace | 脚本维护者可改默认参数 |
| HTTP 采样 | status: `running/tick_count/consensus.committed_height/network_committed_height/known_peer_heads/last_error`；balances: `reward_mint_record_count/node_power_credit_balance/node_main_token_liquid_balance` | 周期抓取并写入 timeline/summary | `sampling -> aggregated -> reported` | 高度滞后与错误字段优先标注 | QA/发布可审阅 |
| 产物契约保持 | `run_config.json/timeline.csv/summary.json/summary.md/failures.md` | 每次 run 输出固定结构产物 | `generated -> archived` | 失败时必须附 `failures.md` 诊断 | 自动化流水线直接消费 |
| chaos 复用 | 故障注入配置与恢复窗口 | 在 P2P longrun 中执行注入并观测恢复 | `stable -> injected -> recovered/degraded` | 注入后需经过观测窗口再判定门禁 | 测试维护者可开关/调参 |
- Acceptance Criteria:
  - AC-1: 两个脚本均完成 `world_chain_runtime` 链路改造并可执行。
  - AC-2: status/balances 采样字段满足 summary 生成与门禁判断最小集合。
  - AC-3: 产物文件名与核心语义兼容既有消费方。
  - AC-4: 旧链路弃用说明与兼容行为在脚本帮助/文档中可追溯。
  - AC-5: 回归验证、项目状态与任务日志完成收口。
- Non-Goals:
  - 不恢复 `world_viewer_live` 内嵌节点路径。
  - 不追求旧脚本全部细分字段的 1:1 恢复。
  - 不在本轮新增新的长跑质量指标体系。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为脚本运行链路迁移与可观测性恢复，不涉及 AI 推理系统改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `world_chain_runtime` 作为统一节点进程，脚本通过 HTTP 状态端点采样运行健康并汇总产物，替代旧 viewer 内嵌节点路径。
- Integration Points:
  - `scripts/s10-five-node-game-soak.sh`
  - `scripts/p2p-longrun-soak.sh`
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `testing-manual.md`
  - `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.project.md`
- Edge Cases & Error Handling:
  - 字段缺失：旧 epoch 报表特有字段缺失时标记 `unavailable`，避免伪造对齐。
  - 启动抖动：引入 startup grace 与采样重试窗口，降低初期误判。
  - 参数兼容偏差：help 中显式区分兼容字段与弃用字段。
  - 节点异常退出：保留失败节点日志与 `failures.md` 诊断信息。
- Non-Functional Requirements:
  - NFR-SOAK-1: 脚本具备稳定可执行性，支持时长控制与优雅收口。
  - NFR-SOAK-2: 产物结构在自动化消费侧保持兼容。
  - NFR-SOAK-3: 门禁判定可在缺失部分旧字段时仍输出可解释结论。
  - NFR-SOAK-4: 脚本失败可在单次 run 内定位根因（节点/阶段/错误签名）。
- Security & Privacy: 脚本日志不应泄露私钥等敏感参数；节点绑定与对外暴露范围按现有测试环境最小化。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (SOAKREACT-1): 完成专题设计与项目文档建档。
  - v1.1 (SOAKREACT-2): S10 脚本迁移到 `world_chain_runtime` 五节点启动链路。
  - v2.0 (SOAKREACT-3): P2P 脚本迁移并恢复 chaos 注入能力。
  - v2.1 (SOAKREACT-4): 完成回归、手册口径对齐与文档收口。
  - v2.2 (SOAKREACT-5): 按 strict schema 人工迁移并统一 `.prd` 命名。
- Technical Risks:
  - 风险-1: 新链路缺失旧 epoch 细粒度字段，可能影响历史对比口径。
  - 风险-2: 多节点启动顺序与网络抖动导致短时误报。
  - 风险-3: 参数兼容策略不清晰会引发旧自动化调用偏差。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-SOAK-001 | SOAKREACT-1/2 | `test_tier_required` | S10 脚本启动与运行时长控制验证 | longrun 基础可执行性 |
| PRD-TESTING-LONGRUN-SOAK-002 | SOAKREACT-2/3/4 | `test_tier_required` | status/balances 采样字段与 summary 产物核验 | 门禁可观测性与发布证据 |
| PRD-TESTING-LONGRUN-SOAK-003 | SOAKREACT-3/4/5 | `test_tier_required` | P2P chaos 回归 + 文档治理/引用检查 | 自动化兼容性与追溯一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-SOAK-001 | 统一切换到 `world_chain_runtime` 启动链路 | 延续 `world_viewer_live --node-*` 旧路径 | 旧路径已下线，无法支撑可执行长跑。 |
| DEC-SOAK-002 | 以 status/balances HTTP 采样重建门禁与报表 | 依赖旧 epoch 文件接口 | 统一运行时接口更稳定且可维护。 |
| DEC-SOAK-003 | 优先保证核心门禁可用，缺失字段标 `unavailable` | 强制恢复全部历史字段 | 控制变更复杂度，先恢复可执行与可判定能力。 |

## 原文约束点映射（内容保真）
- 原“目标（恢复脚本可执行/可产物/可门禁）” -> 第 1 章 Problem/Solution/SC。
- 原“范围（S10/P2P 脚本、status/balances 采样、chaos 复用）” -> 第 2 章规格矩阵 + 第 4 章 Integration。
- 原“非目标（不回滚 viewer 节点路径、不强求 1:1 字段）” -> 第 2 章 Non-Goals。
- 原“接口/数据（节点参数、采样字段、产物约定）” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M1~M4” -> 第 5 章 Phased Rollout（SOAKREACT-1~5）。
- 原“风险（字段缺失、兼容偏差、启动抖动）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
