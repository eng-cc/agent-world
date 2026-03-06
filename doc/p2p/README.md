# p2p 文档索引

审计轮次: 5

## 入口
- PRD: `doc/p2p/prd.md`
- 项目管理: `doc/p2p/prd.project.md`
- 文件级索引: doc/p2p/prd.index.md

## 主题目录
- `distfs/`: DistFS 设计与稳定性加固。
- `node/`: 节点能力、奖励、身份与复制链路。
- `observer/`: 观察者同步模式与可观测性。
- `blockchain/`: 区块链与 P2PFS 硬化阶段。
- `token/`: 主链 token 分配与治理分发。
- `viewer-live/`: viewer live 发行与开关策略。
- `consensus/`: 共识相关专题。
- `distributed/`: 分布式运行时专题。
- `network/`: 网络桥接专题。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`prd.project.md`、`prd.index.md`。

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为变更需同步更新 `prd.md` 与 `prd.project.md`。
