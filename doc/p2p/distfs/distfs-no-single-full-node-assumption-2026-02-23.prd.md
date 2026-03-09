# Agent World Runtime：分布式存储去单机完整依赖改造（2026-02-23）

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 明确移除“任意单机可独立提供完整执行数据”的隐含假设。
- Proposed Solution: DHT 路径下的数据拉取必须依赖 provider 索引，不允许无 provider 时回退到非定向全网请求。
- Success Criteria:
  - SC-1: 在回放/启动校验链路中增加分布覆盖约束，避免“单节点全覆盖”成为系统可接受常态。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式存储去单机完整依赖改造（2026-02-23） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `agent_world_net::DistributedClient`
  - AC-2: `fetch_blob_from_dht` 改为严格 DHT provider 模式：
  - AC-3: 无 provider -> 失败；
  - AC-4: provider 全失败 -> 失败；
  - AC-5: 不再回退 `fetch_blob(content_hash)`。
  - AC-6: `agent_world_net`
- Non-Goals:
  - 自动副本修复调度。
  - 分片迁移与负载重平衡控制面。
  - 纠删码编码/解码协议改造。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.md`
  - `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
### 1) 严格 DHT 拉取语义
- `DistributedClient::fetch_blob_from_dht`：
  - 输入：`world_id/content_hash/dht`。
  - 行为：仅按 DHT provider 列表排序后逐节点重试。
  - 失败语义：
    - provider 为空：`DistributedValidationFailed`；
    - 全 provider 失败：返回最后一次错误或统一 `DistributedValidationFailed`。

### 2) 分布覆盖策略
- 新增 `ProviderDistributionPolicy`：
  - `min_replicas_per_blob: usize`（默认 2）
  - `forbid_single_provider_full_coverage: bool`（默认 true）
- 审计输入：`world_id` + 一组执行数据 hash（block/snapshot_manifest/journal_segments/chunks/segments）。
- 审计输出：
  - 通过：覆盖满足约束。
  - 失败：给出副本不足 hash 与违规 provider 信息。

#### 当前状态
- 状态：已完成
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 已完成：M0、M1、M2、M3
- 进行中：无
- 未开始：无

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：设计与任务拆解（已完成）。
  - M1：严格 DHT 拉取落地（去掉单机回退，已完成）。
  - M2：覆盖审计模块与 DHT 批量拉取接线（已完成）。
  - M3：回归、文档与日志收口（已完成）。
- Technical Risks:
  - 风险：严格模式会暴露历史环境中 provider 注册不全问题。
  - 缓解：错误信息显式包含缺失 hash，便于运维补齐 provider 发布流程。
  - 风险：覆盖审计增加 DHT 查询开销。
  - 缓解：仅在启用批量 DHT 拉取分布审计时触发，默认单 blob 读取不引入额外查询。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-065-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-065-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
