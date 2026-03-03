# Agent World: 发布门禁指标策略对齐（2026-02-28）

- 对应项目管理文档: doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.prd.project.md

## 1. Executive Summary
- Problem Statement: S9/S10 在 `world_chain_runtime` 路径下存在“闭环已运行但指标缺失”的门禁误判，导致发布风险评估失真。
- Proposed Solution: 让发布门禁直接消费 reward runtime 真实指标（mint/distfs/settlement/invariant），并同步脚本与手册口径，避免兼容字段导致的偏差。
- Success Criteria:
  - SC-1: `world_chain_runtime` 暴露 `reward_runtime` 指标快照并可被门禁脚本稳定读取。
  - SC-2: S9/S10 脚本以真实 reward runtime 指标完成比例计算并保持产物结构兼容。
  - SC-3: chaos 场景下 `reward_asset_invariant_violation` 仅由 `reward_runtime.invariant_ok=false` 触发。
  - SC-4: 发布前回归包含 runtime 构建、S9/S10 复跑与文档收口，结果可追溯。

## 2. User Experience & Functionality
- User Personas:
  - 发布负责人：需要可信的门禁指标避免误放行或误阻断。
  - 测试维护者：需要脚本口径与运行时指标一致。
  - Runtime 维护者：需要可诊断的状态接口定位异常。
- User Scenarios & Frequency:
  - 发布候选验证：每个候选版本执行一次 S9/S10 门禁。
  - 门禁异常排查：出现误判时按状态接口与汇总产物定位问题。
  - 脚本升级回归：涉及链路参数调整时执行最小回归验证。
- User Stories:
  - PRD-TESTING-GOV-RELEASE-001: As a 发布负责人, I want release gates to use runtime-native reward metrics, so that gate outcomes match real system behavior.
  - PRD-TESTING-GOV-RELEASE-002: As a 测试维护者, I want S9/S10 scripts to aggregate per-node metrics deterministically, so that chaos data windows do not cause false negatives.
  - PRD-TESTING-GOV-RELEASE-003: As a Runtime 维护者, I want `/v1/chain/status` to expose reward runtime snapshots, so that failures are diagnosable without guessing.
- Critical User Flows:
  1. Flow-REL-001: `启动 world_chain_runtime -> 开启 reward runtime worker -> 通过 /v1/chain/status 输出快照`
  2. Flow-REL-002: `执行 S9/S10 脚本 -> 聚合每节点 reward 指标 -> 生成 summary/failures 产物`
  3. Flow-REL-003: `门禁失败 -> 依据 invariant/distfs/settlement 指标定位原因 -> 修复后复跑`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| runtime 奖励参数 | `--reward-runtime-enable/disable`、epoch、points、reserve、distfs 参数 | 启动命令透传参数并驱动 worker | `disabled -> enabled -> running` | 默认使用显式传参，缺参按默认策略 | Runtime 维护者可修改，发布者只读 |
| status 奖励快照 | `metrics_available`、mint/distfs/settlement/invariant 字段 | `GET /v1/chain/status` 返回可诊断结构 | `collecting -> available -> stale/error` | 以 runtime 输出为准，不回退兼容推断 | 脚本可读，维护者可扩展 |
| S9/S10 指标聚合 | mint 样本、distfs/settlement 比率、失败分类 | 脚本执行后产出 `summary.json/md` 与 `failures.md` | `running -> summarized -> gated` | 每节点累计取最大值后聚合 | 测试维护者可更新脚本逻辑 |
| invariant 门禁判定 | `reward_runtime.invariant_ok`、`last_error` | 仅在显式 invariant=false 时报违规 | `healthy -> violated` | 与 `running_false/http_failure` 解耦 | 发布门禁策略维护者控制阈值 |
- Acceptance Criteria:
  - AC-1: `world_chain_runtime` 支持并稳定解析 reward runtime 相关参数族。
  - AC-2: `/v1/chain/status` 返回 reward runtime 快照字段，门禁脚本可直接消费。
  - AC-3: `scripts/p2p-longrun-soak.sh` 与 `scripts/s10-five-node-game-soak.sh` 完成真实指标聚合改造。
  - AC-4: `run_config.json`、`summary.json`、`summary.md`、`failures.md` 路径与结构保持兼容。
  - AC-5: 手册、项目文档、devlog 同步收口。
- Non-Goals:
  - 不改动 `third_party/`。
  - 不将奖励网络重构为跨节点签名服务。
  - 不引入与本次门禁策略无关的新发布指标体系。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为 runtime/脚本门禁策略对齐，不涉及 AI 推理组件）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `world_chain_runtime` 的 reward runtime worker 为单一事实来源，脚本直接消费 status 快照并在 S9/S10 执行后生成兼容汇总产物。
- Integration Points:
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
  - `scripts/p2p-longrun-soak.sh`
  - `scripts/s10-five-node-game-soak.sh`
  - `testing-manual.md`
  - `.tmp/release_gate_s10/*/summary.json`
  - `.tmp/release_gate_p2p/*/summary.json`
- Edge Cases & Error Handling:
  - `distfs_total_checks` 在短窗口为 0：标记为 `insufficient_data`，与比例超阈值分离处理。
  - chaos 注入导致 `running_false/http_failure`：不直接判为 invariant 违规。
  - runtime 新参数未编译进二进制：启动即失败并提示先执行 `cargo build -p agent_world --bin world_chain_runtime`。
  - 部分节点指标缺失：聚合时按可用节点最大累计值计算并保留告警。
  - 状态接口不可达：脚本输出 HTTP 失败分类并终止门禁判定。
- Non-Functional Requirements:
  - NFR-REL-1: S9/S10 指标口径在运行时与脚本间保持一致，无兼容字段回退漂移。
  - NFR-REL-2: 门禁失败原因可在一次排查中定位到 invariant、distfs 或 settlement 子项。
  - NFR-REL-3: 产物结构向后兼容，既有消费链路无需变更路径。
  - NFR-REL-4: 复跑门禁结果具备可复现性与审计可读性。
- Security & Privacy: 门禁产物仅包含运行指标与状态摘要，不新增敏感凭据输出。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (RELGATE-1): 完成设计与项目文档建档。
  - v1.1 (RELGATE-2): runtime 接入 reward worker 并扩展 status 快照。
  - v2.0 (RELGATE-3): S9/S10 脚本切换真实指标并修复误判。
  - v2.1 (RELGATE-4): 复跑发布门禁与手册/日志收口。
- Technical Risks:
  - 风险-1: 短窗口采样导致数据不足噪声，需明确“未就绪”与“违规”语义。
  - 风险-2: chaos 场景噪声掩盖真实 invariant 状态，需严格判定来源。
  - 风险-3: 参数与二进制版本不一致导致启动失败，需把重编译纳入回归步骤。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-GOV-RELEASE-001 | RELGATE-1/2 | `test_tier_required` | runtime 参数解析与 status 字段检查 | 发布门禁数据源可信度 |
| PRD-TESTING-GOV-RELEASE-002 | RELGATE-2/3 | `test_tier_required` + `test_tier_full` | S9/S10 脚本聚合逻辑与失败分类回归 | 长跑门禁误判率 |
| PRD-TESTING-GOV-RELEASE-003 | RELGATE-3/4 | `test_tier_required` | 发布门禁复跑 + 手册/devlog 收口审查 | 发布放行决策可靠性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-REL-001 | 直接消费 reward runtime 指标 | 继续依赖兼容字段或放宽阈值 | 真实指标更能反映运行状态，减少误判。 |
| DEC-REL-002 | 每节点累计最大值后聚合比例 | 单节点瞬时值直接判定 | 累计最大值更抗短时波动和采样窗口噪声。 |
| DEC-REL-003 | invariant 仅由 `invariant_ok=false` 触发违规 | 将 `running_false/http_failure` 直接映射违规 | 可避免 chaos 噪声导致误杀。 |

## 原文约束点映射（内容保真）
- 原“目标（修复 S9/S10 指标误判）” -> 第 1 章 Problem/Solution/SC。
- 原“范围/非目标” -> 第 2 章 AC 与 Non-Goals。
- 原“接口/数据（runtime 参数、status 字段、脚本聚合、产物兼容）” -> 第 2 章规格矩阵 + 第 4 章 Integration。
- 原“里程碑 M1~M5” -> 第 5 章 Phased Rollout（RELGATE-1~4）。
- 原“风险与缓解” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
