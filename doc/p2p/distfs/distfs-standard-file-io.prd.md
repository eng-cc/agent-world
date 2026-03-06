# Agent World Runtime：DistFS 标准文件读写接口

审计轮次: 4
## 1. Executive Summary
- Problem Statement: 在 `agent_world_distfs` 内提供最小可用的“标准文件读写接口”，补齐当前仅有 blob/CAS 接口的缺口。
- Proposed Solution: 保持内容寻址（CAS）为底层真相，文件路径只是到 `content_hash` 的可变索引映射。
- Success Criteria:
  - SC-1: 提供本地可测试闭环：写文件、读文件、列目录、查询元信息、删除映射。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 标准文件读写接口 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `agent_world_distfs` 增加文件抽象接口（path -> content_hash）。
  - AC-2: 本地索引存储（JSON）及原子写入。
  - AC-3: 路径校验（禁止空路径、`..` 穿越、绝对路径）。
  - AC-4: 基础单元测试：读写回读、覆盖写、删除、路径校验。
- Non-Goals:
  - 跨节点分布式复制一致性。
  - 文件权限模型、ACL、租约写锁。
  - 目录树高阶操作（move/copy/glob/watch）。
  - 加密存储与内容访问审计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-standard-file-io.prd.md`
  - `doc/p2p/distfs/distfs-standard-file-io.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 文件接口（草案）
- `FileStore::write_file(path, bytes)`：写入文件路径，返回文件元信息。
- `FileStore::read_file(path)`：按路径读取内容。
- `FileStore::delete_file(path)`：删除路径映射（不强制删除底层 blob）。
- `FileStore::stat_file(path)`：读取路径对应元信息。
- `FileStore::list_files()`：列出已登记路径。

### 元信息模型（草案）
```rust
FileMetadata {
  path: String,
  content_hash: String,
  size_bytes: u64,
  updated_at_ms: i64,
}
```

### 本地索引文件（草案）
- 文件名：`files_index.json`
- 结构：
```rust
FileIndexFile {
  version: u64,
  files: BTreeMap<String, FileMetadata>,
}
```
- 版本：`version = 1`

### 语义约束
- 写入文件时：
  - 先写入底层 CAS（`put_bytes`）
  - 再更新路径索引
- 删除文件时：
  - 仅删除路径映射
  - 底层 blob 生命周期由 pin/evict/GC 决策

## 5. Risks & Roadmap
- Phased Rollout:
  - DFIO-1：设计文档与项目管理文档。
  - DFIO-2：文件接口与本地索引实现。
  - DFIO-3：单元测试与回归。
- Technical Risks:
  - 文件路径到 hash 的索引可能与外部期望不一致（如覆盖语义），需通过接口文档明确。
  - 长期运行下索引文件可能增长，需要后续分页或分片。
  - 仅本地语义尚未覆盖跨节点并发写冲突。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-080-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-080-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
