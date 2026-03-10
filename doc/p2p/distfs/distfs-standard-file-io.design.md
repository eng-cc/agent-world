# DistFS 标准文件读写接口设计

- 对应需求文档: `doc/p2p/distfs/distfs-standard-file-io.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-standard-file-io.project.md`

## 1. 设计定位
定义 DistFS 最小可用的标准文件读写接口，让上层能够通过路径访问内容，同时底层仍以 CAS 为真相。

## 2. 设计结构
- 文件接口层：提供 write/read/delete/stat/list 基础能力。
- 索引文件层：`files_index.json` 维护路径到 `FileMetadata` 的映射。
- 路径校验层：禁止空路径、绝对路径和 `..` 穿越。
- 一致性层：先写 CAS，再更新路径索引；删除只删映射。

## 3. 关键接口 / 入口
- `FileStore::*`
- `FileMetadata`
- `files_index.json`
- 路径校验规则

## 4. 约束与边界
- 不做跨节点复制一致性、ACL、租约锁和高阶目录操作。
- 路径索引是可变入口，不替代 CAS 真相。
- 删除映射不等于删除底层 blob。
- 目标是提供上层 runtime/net 可稳定复用的最小接口。

## 5. 设计演进计划
- 先落接口和索引模型。
- 再补路径校验与回归。
- 最后接入上层分布式调用面。
