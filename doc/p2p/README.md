# p2p 文档索引

审计轮次: 8

## 入口
- PRD: `doc/p2p/prd.md`
- 设计总览: `doc/p2p/design.md`
- 标准执行入口: `doc/p2p/project.md`
- 文件级索引: `doc/p2p/prd.index.md`

## 模块职责
- 维护 P2P、共识、DistFS、节点奖励与网络桥接等核心链路口径。
- 汇总 blockchain / distfs / node / observer / token / viewer-live / consensus / distributed / network 九类专题。
- 承接跨 runtime、launcher、viewer-live 的分布式运行与发布约束收口。

## 主题目录
- `distfs/`：DistFS 设计与稳定性加固。
- `node/`：节点能力、奖励、身份与复制链路。
- `observer/`：观察者同步模式与可观测性。
- `blockchain/`：区块链与 P2PFS 硬化阶段。
- `token/`：主链 token 分配、创世分桶、低流通与治理分发。
- `viewer-live/`：viewer live 发行与开关策略。
- `consensus/`：共识相关专题。
- `distributed/`：分布式运行时专题。
- `network/`：网络桥接专题。

## 近期专题
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.prd.md`
- `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
- `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.md`

## 根目录收口
- 模块根目录主入口保留：`README.md`、`prd.md`、`design.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `blockchain/`、`distfs/`、`node/`、`observer/`、`token/`、`viewer-live/`、`consensus/`、`distributed/`、`network/`。

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为、默认参数或跨模块分布式口径变化时，需同步更新 `prd.md` 与 `project.md`。
- 新增专题后，需同步回写 `doc/p2p/prd.index.md` 与本目录索引。
