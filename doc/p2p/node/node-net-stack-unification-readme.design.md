# README P2 缺口收口：Node Replication 统一到 agent_world_net 网络栈设计

- 对应需求文档: `doc/p2p/node/node-net-stack-unification-readme.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-net-stack-unification-readme.project.md`

## 1. 设计定位
定义 Node Replication 统一到 `agent_world_net` 网络栈的 README P2 收口设计，确保节点复制路径与统一网络抽象对齐。

## 2. 设计结构
- 统一网络层：把 node replication 收敛到 `agent_world_net` 抽象。
- 桥接兼容层：为旧复制入口提供迁移桥接与兼容兜底。
- 能力收口层：统一请求、复制、观测和错误语义。
- 回归收口层：围绕迁移后的主路径执行 required-tier 回归。

## 3. 关键接口 / 入口
- `agent_world_net` 网络抽象
- Node replication 迁移入口
- 桥接兼容开关
- 网络回归矩阵

## 4. 约束与边界
- 统一后仍需保留必要兼容路径，避免一次性切断旧链路。
- 错误语义需与统一网络栈保持一致。
- 不在本专题扩展新的网络协议族。

## 5. 设计演进计划
- 先冻结 README/P2 缺口与接口边界。
- 再完成网络栈迁移接线。
- 最后补齐回归与文档收口。
