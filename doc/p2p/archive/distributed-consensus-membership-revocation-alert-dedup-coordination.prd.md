> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销告警抑制去重与调度多节点协同

## 1. Executive Summary
- Problem Statement: 在吊销对账告警上报链路中增加去重抑制，降低持续异常场景的告警噪音。
- Proposed Solution: 为对账调度提供多节点协同入口，避免多个节点重复执行同一周期任务。
- Success Criteria:
  - SC-1: 与现有 store/sink 调度编排接口兼容，支持增量接入。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销告警抑制去重与调度多节点协同 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 告警去重抑制策略与状态：
  - AC-2: `MembershipRevocationAlertDedupPolicy`
  - AC-3: `MembershipRevocationAlertDedupState`
  - AC-4: `deduplicate_revocation_alerts(...)`
  - AC-5: 多节点调度协同抽象与内存实现：
  - AC-6: `MembershipRevocationScheduleCoordinator`
- Non-Goals:
  - 协调锁持久化到外部 KV（Redis/etcd）。
  - 分布式时钟漂移校正与 NTP 容错。
  - 告警级联降噪（聚合、升级、恢复策略）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alert-dedup-coordination.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alert-dedup-coordination.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 告警去重
- `MembershipRevocationAlertDedupPolicy`
  - `suppress_window_ms`
- `MembershipRevocationAlertDedupState`
  - `last_emitted_at_by_key`
- `deduplicate_revocation_alerts(...)`
  - 输入候选告警列表
  - 基于 key + 时间窗口输出可发送告警

### 多节点协同
- `MembershipRevocationScheduleCoordinator`
  - `acquire(...)` / `release(...)`
- `MembershipRevocationCoordinatedRunReport`
  - `acquired`
  - `emitted_alerts`
  - `run_report`
- `run_revocation_reconcile_coordinated(...)`
  - 先协调获取执行权
  - 成功后执行 schedule/store/alert 编排
  - 可选应用告警去重

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：告警去重策略与状态实现。
  - **MR3**：多节点协同抽象/内存实现与编排入口实现。
  - **MR4**：测试、导出、总文档与日志更新。
- Technical Risks:
  - 去重窗口配置过大会掩盖真实高频故障，过小则噪声抑制不足。
  - 协同锁基于本地内存，仅适用于单进程模拟或测试环境。
  - 节点异常退出未显式释放时，需要依赖 TTL 或下一轮协调回收。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-007-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-007-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
