# DistFS 执行产物路径索引设计

- 对应需求文档: `doc/p2p/distfs/distfs-runtime-path-index.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-runtime-path-index.project.md`

## 1. 设计定位
定义 DistFS `FileStore` 接入 `execution_storage` 的路径索引方案，为执行产物提供稳定的路径回读入口，同时保持 CAS 真相。

## 2. 设计结构
- CAS 真相层：content hash 继续作为底层权威。
- 路径映射层：执行结果写入时同步生成 `world_id + height` 等可读路径。
- 读取闭环层：后续模块可按路径回读区块和最新产物。
- 执行接线层：让 execution_storage 写入与 FileStore 索引同步。

## 3. 关键接口 / 入口
- `FileStore`
- `execution_storage`
- 执行产物路径规则
- 路径索引回读入口

## 4. 约束与边界
- 路径只是索引，不替代 CAS 数据真相。
- 不在本轮重构复杂目录操作或跨节点一致性。
- 设计重点是让 runtime/net 上层更稳定消费执行产物。
- 写入和索引更新需要保持最小闭环一致。

## 5. 设计演进计划
- 先接 execution_storage 到 FileStore。
- 再补路径回读闭环。
- 最后通过执行产物回归收口。
