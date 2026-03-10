# DistFS Builtin WASM Storage 设计

- 对应需求文档: `doc/p2p/distfs/distfs-builtin-wasm-storage.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-builtin-wasm-storage.project.md`

## 1. 设计定位
定义 DistFS 内建 WASM 存储能力的统一方案：让标准文件读写、append-only、tombstone 和作者签名控制具备可扩展的内建存储底座。

## 2. 设计结构
- 存储模型层：定义内建 WASM 存储的数据形态与读写语义。
- 权限控制层：围绕签名作者、append-only、tombstone 建立基础访问规则。
- 审计限流层：对写入、删除和反馈操作提供审计与限流约束。
- 运行时接线层：把存储能力接入 DistFS/Node Runtime 的活跃链路。

## 3. 关键接口 / 入口
- 内建 WASM storage 模型
- 作者签名控制
- append-only / tombstone 语义
- 审计与限流配置

## 4. 约束与边界
- 本专题定义内建存储底座，不扩展完整查询网关和内容审核平台。
- 权限与签名控制优先保证正确性，不先追求复杂策略表达。
- 标准文件 IO 与内建存储要保持概念边界清晰。
- 设计需为后续 feedback/open ledger/p2p bridge 主题复用。

## 5. 设计演进计划
- 先冻结 storage 数据模型和作者控制语义。
- 再补 append-only / tombstone / 限流审计。
- 最后接入运行时并通过回归收口。
