# Node 共识签名身份绑定与复制摄取硬化（项目管理文档）

- 对应设计文档: `doc/p2p/node/node-consensus-signer-binding-replication-hardening.design.md`
- 对应需求文档: `doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-P2P-MIG-088)：输出设计文档（`doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.md`）
- [x] T0 (PRD-P2P-MIG-088)：输出项目管理文档（本文件）
- [x] T1 (PRD-P2P-MIG-088)：实现 P0 数据模型与校验链路（`NodePosConfig` signer 绑定 + 配置校验 + ingest 校验）
- [x] T2 (PRD-P2P-MIG-088)：补齐 P0 测试（合法路径、错绑拒绝、缺失公钥拒绝）
- [x] T3 (PRD-P2P-MIG-088)：实现 P1 硬化（replication ingest 验证后观测 + 连续高度落库，错误上抛；PoS 状态恢复失败显式失败）
- [x] T4 (PRD-P2P-MIG-088)：补齐 P1 测试并执行回归（`agent_world_node` 全量 + 相关定向）

## 依赖
- T2 依赖 T1（先完成数据模型和主链路，再补测试断言）。
- T3 与 T2 可并行但建议串行（先稳定 P0，再推进 P1 降低回归噪音）。
- T4 依赖 T1/T2/T3 全部完成。

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（T0~T4 全部完成）
- 阻塞项：无
- 下一步：等待后续需求
