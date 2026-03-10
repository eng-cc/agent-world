# DistFS 生产化增强 Phase 1 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase1.project.md`

## 1. 设计定位
定义 DistFS 生产化增强的主入口方案：在不破坏 `BlobStore`/`FileStore` 接口兼容性的前提下，补齐可审计、可回收、可同步、可并发保护的最小闭环。

## 2. 设计结构
- 条件写删层：为路径级写入和删除增加 CAS 语义，避免并发丢更新。
- 索引审计层：输出文件索引审计报告，识别缺失 blob、悬挂 pin 和孤儿块。
- 回收与 manifest 层：支持孤儿 blob 回收、manifest 导出导入和版本引用。
- 主从口径层：phase1 作为主入口，phase2~9 只维护增量。

## 3. 关键接口 / 入口
- `write_file_if_match`
- `delete_file_if_match`
- `FileIndexAuditReport`
- `FileIndexManifest` / `FileIndexManifestRef`

## 4. 约束与边界
- 不做 CRDT/OT 多写者合并。
- 不重构跨节点复制协议与共识提交。
- ACL、租约锁和端到端加密不在本轮范围。
- phase1 负责总基线，后续 phase 应避免重复定义通用边界。

## 5. 设计演进计划
- 先落 CAS 写删。
- 再补索引审计与孤儿回收。
- 最后通过 manifest 导入导出和回归收口主入口。
