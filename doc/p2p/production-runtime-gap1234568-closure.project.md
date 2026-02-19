# Agent World Runtime：生产级收口（Gap 1/2/3/4/5/6/8）项目管理文档

## 任务拆解
- [x] T0：输出设计文档（`doc/p2p/production-runtime-gap1234568-closure.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：共识提交-执行强绑定 + 默认投票策略收口（Gap 3/4）
- [x] T2：writer epoch failover（Gap 5）
- [x] T3：replication block exchange 协议与 handler（Gap 1/2）
- [x] T4：网络优先补洞同步主路径（Gap 1/2）
- [ ] T5：存储挑战纳入共识门控（Gap 6）
- [ ] T6：默认分布式网络集成收口（Gap 8）
- [ ] T7：回归验证（`env -u RUSTC_WRAPPER cargo check` + required-tier）+ 文档/devlog 收口

## 依赖
- T2 依赖 T1 的主循环稳定（避免 failover 与执行绑定互相放大回归面）。
- T4 依赖 T3 的协议与 handler 已可用。
- T5 依赖 T3 的 blob exchange 可用于挑战网络校验。
- T6 可并行，但在 T7 统一回归。

## 状态
- 当前阶段：进行中（T0/T1/T2/T3/T4 完成，T5 起）
- 阻塞项：无
- 下一步：执行 T5
