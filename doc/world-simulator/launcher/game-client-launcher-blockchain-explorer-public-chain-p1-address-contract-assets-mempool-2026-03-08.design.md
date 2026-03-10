# 启动器区块链浏览器公共主链 P1 设计（地址/合约/资产/内存池）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.project.md`

## 1. 设计定位
定义 explorer P1 在 runtime、控制面与 launcher UI 三层之间的查询接口、分页模型与只读边界。

## 2. 设计结构
- 运行时层：address/contracts/contract/assets/mempool 查询聚合。
- 控制面层：`world_web_launcher` 对应 `/api/chain/explorer/*` 路由代理。
- UI 层：Address/Contracts/Assets/Mempool 四视图统一状态机。

## 3. 关键接口 / 入口
- `/v1/chain/explorer/address|contracts|contract|assets|mempool`
- explorer 面板分页/过滤状态模型
- public chain 只读查询返回结构

## 4. 约束与边界
- 不引入写操作与重型索引服务。
- 非法参数必须返回结构化 `invalid_request`。
- 当前链不支持 NFT 时应显式返回 `nft_supported=false`。

## 5. 设计演进计划
- 先完成 Design 补齐与互链回写。
- 再沿项目文档推进实现与验证。
