# Agent World Runtime：异构节点分布式存储最优稳定性改造（2026-02-23）

审计轮次: 3

## 1. Executive Summary
- Problem Statement: 面向“1000+ 节点、容量与在线时长显著异构”的场景，构建可长期稳定运行的分布式存储策略。
- Proposed Solution: 将当前“provider 列表 + 单节点优先请求”升级为“能力感知排序 + 多候选重试 + 退避回退”，降低单点离线与弱节点抖动影响。
- Success Criteria:
  - SC-1: 在不破坏现有协议兼容的前提下，为后续“容量感知副本放置 / 自动修复”预埋数据结构。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：异构节点分布式存储最优稳定性改造（2026-02-23） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `agent_world_proto`
  - AC-2: 扩展 `ProviderRecord`，新增可选节点能力画像（容量、可用空间、在线率、挑战通过率、负载、延迟）。
  - AC-3: `agent_world_net`
  - AC-4: 新增 provider 评分/排序策略模块。
  - AC-5: `DistributedClient` 的 DHT 拉取路径升级为：按评分排序后逐节点定向重试（失败自动降级）。
  - AC-6: 保留现有无画像节点的兼容行为（按时间新鲜度优先）。
- Non-Goals:
  - 完整自动分片放置与在线重平衡调度（另行迭代）。
  - 纠删码（Erasure Coding）与 PoSt 级证明协议完整化。
  - 运维编排系统（跨机自动发现、弹性扩缩容控制面）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.md`
  - `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
### 1) Provider 能力画像（向后兼容）
- `ProviderRecord` 新增可选字段：
  - `storage_total_bytes: Option<u64>`
  - `storage_available_bytes: Option<u64>`
  - `uptime_ratio_per_mille: Option<u16>`
  - `challenge_pass_ratio_per_mille: Option<u16>`
  - `load_ratio_per_mille: Option<u16>`
  - `p50_read_latency_ms: Option<u32>`
- 兼容策略：
  - 历史数据缺失新字段时默认 `None`。
  - 排序策略对 `None` 使用中性分，确保旧节点可继续参与。

### 2) Provider 评分策略
- 新增 `ProviderSelectionPolicy`：
  - `freshness_ttl_ms`
  - `weight_freshness / weight_uptime / weight_challenge / weight_capacity / weight_load / weight_latency`
  - `max_candidates`
- 评分输出：
  - 归一化分值 `0.0~1.0`。
  - 可按分数降序给出候选 provider 序列。

### 3) 拉取重试策略
- `DistributedClient::fetch_blob_from_dht` 调整为：
  1. 读取 provider 列表。
  2. 基于策略排序。
  3. 逐 provider 定向请求（单 provider）并重试，直至成功。
  4. 全部失败后，回退到原始无 provider 拉取路径。
- 目标：降低“列表首节点离线/抖动”导致的失败概率。

#### 当前状态
- 状态：已完成
- 已完成：M0、M1、M2、M3
- 进行中：无
- 未开始：无

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：设计与任务拆解。
  - M1：ProviderRecord 能力画像扩展与兼容。
  - M2：评分排序与逐节点重试落地。
  - M3：回归测试 + 文档/devlog 收口。
- Technical Risks:
  - 评分策略权重不当可能引入倾斜（热点节点过载）。
  - 缓解：引入 `load_ratio_per_mille` 负反馈项，保留可配置权重。
  - 能力画像缺失导致排序不稳定。
  - 缓解：`None` 字段按中性分处理，不阻断请求。
  - 逐节点重试增加请求时延上界。
  - 缓解：`max_candidates` 限制重试范围，最终保留无 provider 回退路径。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-064-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-064-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
