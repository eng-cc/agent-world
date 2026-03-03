# Agent World Runtime：节点高度/Slot 递进与复制补洞数值语义硬化（15 点清单第三阶段）

- 对应项目管理文档: doc/world-runtime/runtime/runtime-numeric-correctness-phase3.prd.project.md

## 1. Executive Summary
- 延续前两阶段“正确性优先”原则，收口 `agent_world_node` 中长期运行最敏感的高度/slot 计数器路径。
- 把关键链路里的 `saturating_add(1)` 从“静默夹逼继续执行”改为“显式失败 + 不污染状态”。
- 覆盖复制补洞、网络消息摄取、快照恢复三条路径，避免接近 `u64::MAX` 时出现不可观测偏差。

## 2. User Experience & Functionality
### In Scope（第三阶段）
- `PosNodeEngine` 关键递进路径显式溢出语义：
  - `apply_decision` 中 `next_height = height + 1` 改为 `checked_add`，溢出返回 `NodeError`。
  - 复制摄取/补洞路径中 `committed_height + 1` 与 `next_height + 1` 改为显式检查，不再静默饱和。
  - `record_synced_replication_height` 内的 `height + 1` 改为显式检查，拒绝异常边界提交。
  - proposal 摄取时 `next_slot = proposal.slot + 1` 改为显式检查，防止 slot 在上限处停滞。
- 快照恢复边界语义：
  - `restore_state_snapshot` 对 `committed_height + 1` 使用显式检查，越界时返回可观测错误并阻断启动。
- 测试：
  - 新增溢出拒绝测试，覆盖“返回错误 + 状态不被部分更新”。
  - 补齐 node crate 定向回归。

### Out of Scope（后续阶段）
- 全仓库统一引入大整数类型（BigInt/U256）并替换全部 `u64` 计数器。
- 共识消息编码域版本化、跨版本迁移治理。
- 全链路形式化验证与确定性证明平台化。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- `PosNodeEngine::apply_decision`：
  - 由无返回值改为 `Result<(), NodeError>`，关键高度递进失败时显式报错。
- `PosNodeEngine::record_synced_replication_height`：
  - 由无返回值改为 `Result<(), NodeError>`，调用方透传错误。
- `PosNodeEngine::restore_state_snapshot`：
  - 由无返回值改为 `Result<(), NodeError>`，`NodeRuntime::start` 在恢复阶段接线错误返回。
- 复制摄取与补洞：
  - `committed_height + 1` 统一通过受检后继值计算，若越界则返回 `NodeError::Replication` 并停止本轮处理。

## 5. Risks & Roadmap
- M0：Phase3 文档建档并冻结边界。
- M1：Node engine 高度/slot 递进显式溢出语义改造完成。
- M2：快照恢复边界与测试补齐完成。
- M3：定向回归、文档状态与 devlog 收口。

### Technical Risks
- 行为从“饱和后继续”切换为“显式失败”，旧用例可能需要更新预期。
- 多个内部函数签名变更为 `Result`，需要保证调用链一次性改齐，避免遗漏。
- 极端边界测试依赖人工构造状态，若覆盖不足可能漏掉次要路径。

## 15 点清单映射（阶段视角）
- 本阶段优先覆盖：节点高度/slot 递进边界、复制补洞后继高度计算、状态恢复边界校验。
- 前两阶段已覆盖：runtime 账本结算与 PoS 票权主路径溢出语义。
- 后续阶段继续覆盖：剩余非主链路饱和计数器与类型化约束收口。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 已完成。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
