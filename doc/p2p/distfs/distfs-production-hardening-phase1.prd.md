# Agent World Runtime：DistFS 生产化增强（Phase 1）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase1.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase1.project.md`

审计轮次: 5
## ROUND-002 主从口径
- `doc/p2p/distfs/distfs-production-hardening-phase1.prd.md` 为主文档（master）。
- `doc/p2p/distfs/distfs-production-hardening-phase2.prd.md` 至 `doc/p2p/distfs/distfs-production-hardening-phase9.prd.md` 为增量子文档（slave）。
- 本专题通用边界与基线语义以 phase1 为主入口，后续 phase 文档仅维护阶段增量。

## 1. Executive Summary
- Problem Statement: 在不破坏现有 `BlobStore`/`FileStore` 接口兼容性的前提下，补齐 DistFS 的基础生产语义能力。
- Proposed Solution: 提供“可审计、可回收、可同步、可并发保护”的最小闭环，降低文件索引漂移和脏写风险。
- Success Criteria:
  - SC-1: 保持实现集中在 `agent_world_distfs`，便于上层 `agent_world_net` / `agent_world_node` 直接复用。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 生产化增强（Phase 1）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH1-1：条件写删（CAS 语义）**
  - AC-2: 新增路径级 compare-and-set 写入能力，避免“无条件覆盖”导致的并发丢更新。
  - AC-3: 新增路径级 compare-and-set 删除能力，避免“错误版本删除”。
  - AC-4: **DPH1-2：索引审计与孤儿块回收**
  - AC-5: 新增文件索引审计报告，识别：
  - AC-6: 索引引用但 blob 缺失；
- Non-Goals:
  - 多写者冲突自动合并（CRDT/OT）。
  - 跨节点复制协议重构与共识提交。
  - ACL、租约分布式锁、端到端加密与审计追踪。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase1.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 条件写删接口（草案）
```rust
LocalCasStore::write_file_if_match(
  path: &str,
  expected_content_hash: Option<&str>,
  bytes: &[u8]
) -> Result<FileMetadata, WorldError>

LocalCasStore::delete_file_if_match(
  path: &str,
  expected_content_hash: Option<&str>
) -> Result<bool, WorldError>
```

语义：
- `expected_content_hash = Some(hash)` 时，要求当前路径存在且 hash 一致，否则拒绝。
- `expected_content_hash = None` 时，不做版本前置校验，行为与当前写删一致。

### 审计报告（草案）
```rust
FileIndexAuditReport {
  total_indexed_files: usize,
  total_pins: usize,
  missing_file_blob_hashes: Vec<String>,
  dangling_pin_hashes: Vec<String>,
  orphan_blob_hashes: Vec<String>,
}
```

### Manifest（草案）
```rust
FileIndexManifest {
  version: u64,
  files: Vec<FileMetadata>,
}

FileIndexManifestRef {
  content_hash: String,
  size_bytes: u64,
}
```

导出流程：
- 读取 `files_index` -> 规范化排序 -> canonical CBOR -> 落入 CAS -> 返回 `FileIndexManifestRef`。

导入流程：
- 拉取并解码 manifest -> 全量校验 -> 原子替换本地 `files_index`。

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH1-M1**：设计文档与项目管理文档完成。
  - **DPH1-M2**：条件写删能力与测试完成。
  - **DPH1-M3**：审计与孤儿回收能力与测试完成。
  - **DPH1-M4**：Manifest 导出导入与测试完成。
  - **DPH1-M5**：回归、文档状态更新、devlog 收口。
- Technical Risks:
  - 条件写删引入后，调用方若未处理冲突错误，可能出现“重试风暴”；需要上层按版本冲突做退避重试。
  - Manifest 导入是索引级替换，若使用方误传 manifest 可能覆盖本地映射；通过严格校验和错误语义降低风险。
  - 孤儿回收若识别逻辑错误可能误删数据；本期仅回收“未被索引且未被 pin”的集合，并补单测保护。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-067-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-067-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
