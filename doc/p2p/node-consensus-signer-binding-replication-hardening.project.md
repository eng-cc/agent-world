# Node 共识签名身份绑定与复制摄取硬化（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/p2p/node-consensus-signer-binding-replication-hardening.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：实现 P0 数据模型与校验链路（`NodePosConfig` signer 绑定 + 配置校验 + ingest 校验）
- [x] T2：补齐 P0 测试（合法路径、错绑拒绝、缺失公钥拒绝）
- [x] T3：实现 P1 硬化（replication ingest 验证后观测 + 连续高度落库，错误上抛；PoS 状态恢复失败显式失败）
- [x] T4：补齐 P1 测试并执行回归（`agent_world_node` 全量 + 相关定向）

## 依赖
- T2 依赖 T1（先完成数据模型和主链路，再补测试断言）。
- T3 与 T2 可并行但建议串行（先稳定 P0，再推进 P1 降低回归噪音）。
- T4 依赖 T1/T2/T3 全部完成。

## 状态
- 当前阶段：已完成（T0~T4 全部完成）
- 阻塞项：无
- 下一步：等待后续需求
