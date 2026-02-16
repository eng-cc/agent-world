# Agent World Runtime：DistFS 标准文件读写接口（项目管理文档）

## 任务拆解
- [x] DFIO-1：设计文档与项目管理文档落地。
- [x] DFIO-2：实现 `FileStore` 与本地文件索引（`files_index.json`）。
- [x] DFIO-3：补齐单元测试并完成 crate 级回归。
- [x] DFIO-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_distfs`
- `doc/world-runtime/distributed-runtime.md`
- `README.md`（crate 分工）

## 状态
- 当前阶段：DistFS 标准文件读写接口阶段完成（DFIO-1~DFIO-4 全部完成）。
- 下一步：进入上层分布式能力链路，接入文件路径接口到 runtime/net 的调用面。
- 最近更新：2026-02-16。
