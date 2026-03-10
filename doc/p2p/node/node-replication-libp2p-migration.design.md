# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈设计

- 对应需求文档: `doc/p2p/node/node-replication-libp2p-migration.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-replication-libp2p-migration.project.md`

## 1. 设计定位
定义 Node Replication 迁移到 libp2p 统一网络栈的方案，收敛复制网络、peer 管理与消息传输语义。

## 2. 设计结构
- 迁移接线层：把复制流量切到 libp2p request/pubsub 主路径。
- peer 管理层：统一连接、发现、重试与失效剔除。
- 兼容回退层：为旧链路保留可控迁移窗口与回退策略。
- 回归稳定层：围绕复制成功率和时序一致性执行回归。

## 3. 关键接口 / 入口
- libp2p replication 入口
- peer 管理状态
- 兼容回退开关
- 复制迁移回归矩阵

## 4. 约束与边界
- 迁移期间需要保持复制语义稳定。
- peer 管理错误需有可观测失败信号。
- 不在本专题同时引入额外协议栈。

## 5. 设计演进计划
- 先完成接口迁移设计。
- 再切主路径并保留回退。
- 最后执行稳定性回归。
