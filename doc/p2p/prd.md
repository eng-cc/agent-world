# p2p PRD

## 目标
- 建立 p2p 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 p2p 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 p2p 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/p2p/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/p2p/prd.md`
- 项目管理入口: `doc/p2p/prd.project.md`
- 追踪主键: `PRD-P2P-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 网络、共识、DistFS 与节点激励相关设计迭代频繁，缺少统一 PRD 导致跨子系统改动难以同时满足可用性、安全性与可审计性。
- Proposed Solution: 以 p2p PRD 统一定义分布式系统的目标拓扑、共识约束、存储策略、奖励机制与发布门禁。
- Success Criteria:
  - SC-1: P2P 关键改动 100% 映射到 PRD-P2P-ID。
  - SC-2: 多节点在线长跑套件按计划执行并形成可追溯结果。
  - SC-3: 共识与存储链路关键失败模式具备回归测试覆盖。
  - SC-4: 发行前完成网络/共识/DistFS 三线联合验收。

## 2. User Experience & Functionality
- User Personas:
  - 协议工程师：需要明确网络与共识边界。
  - 节点运营者：需要稳定部署和可观测运行信号。
  - 安全评审者：需要签名、治理、资产流转的可审计证据。
- User Stories:
  - PRD-P2P-001: As a 协议工程师, I want explicit protocol boundaries, so that multi-crate changes remain coherent.
  - PRD-P2P-002: As a 节点运营者, I want reliable longrun validation, so that production confidence increases.
  - PRD-P2P-003: As a 安全评审者, I want auditable cryptographic and governance flows, so that risk is controlled.
- Acceptance Criteria:
  - AC-1: p2p PRD 覆盖网络、共识、存储、激励四条主线。
  - AC-2: p2p project 文档任务项明确映射 PRD-P2P-ID。
  - AC-3: 与 `doc/p2p/production-grade-blockchain-p2pfs-roadmap.md` 等设计文档口径一致。
  - AC-4: S9/S10 相关测试套件在 testing 手册中有对应条目。
- Non-Goals:
  - 不在本 PRD 细化 viewer UI 交互。
  - 不替代 runtime 内核的模块执行细节设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 长跑脚本、链路探针、反馈注入、共识日志分析工具。
- Evaluation Strategy: 以在线稳定时长、分叉恢复成功率、反馈链路可用性、错误收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: p2p 模块负责 `agent_world_net`/`agent_world_consensus`/`agent_world_distfs` 与 node 侧分布式运行协同，强调一致性与故障恢复。
- Integration Points:
  - `doc/p2p/production-grade-blockchain-p2pfs-roadmap.md`
  - `doc/p2p/distributed/distributed-hard-split-phase7.md`
  - `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.md`
  - `testing-manual.md`
- Security & Privacy: 需保证节点身份、签名、账本与反馈数据链路的完整性；所有关键动作必须具备可审计记录。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化网络/共识/存储统一设计基线。
  - v1.1: 补齐在线长跑失败模式和恢复手册。
  - v2.0: 建立分布式质量趋势看板（稳定性、时延、恢复、失败率）。
- Technical Risks:
  - 风险-1: 多子系统并行演进带来接口漂移。
  - 风险-2: 长跑测试覆盖不足导致线上异常暴露滞后。
