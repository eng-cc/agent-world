# README P2 缺口收口：Node Replication 统一到 agent_world_net 网络栈（项目管理文档）

## 任务拆解
- [x] NUS-0：输出设计文档与项目管理文档，并明确 wasm32 非 full node 定位。
- [ ] NUS-1：实现 `Libp2pReplicationNetwork` 对 `agent_world_net::Libp2pNetwork` 的封装迁移。
- [ ] NUS-2：补齐/调整 node replication 单测，覆盖轮换重试与无 peer 回退语义。
- [ ] NUS-3：执行回归测试、更新文档与 devlog 并收口。

## 依赖
- NUS-1 依赖 NUS-0（先冻结设计边界再改实现）。
- NUS-2 依赖 NUS-1（实现稳定后再做行为回归验证）。
- NUS-3 依赖 NUS-1/NUS-2 完成。

## 状态
- 当前阶段：NUS-0 已完成；NUS-1 进行中；NUS-2/NUS-3 待执行。
- 阻塞项：无。
- 下一步：完成 node replication 封装迁移并跑回归。
