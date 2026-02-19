# README P1 缺口收口：分布式网络主路径生产化（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/p2p/readme-p1-network-production-hardening.md`）与项目管理文档（本文件）。
- [x] T1：实现 libp2p request 多 peer 轮换重试 + 无 peer 可控回退策略，并补测试。
- [ ] T2：实现 node 共识消息 libp2p pubsub 主路径（ingest/broadcast）并补测试。
- [ ] T3：执行 `env -u RUSTC_WRAPPER cargo test -p agent_world_node` + `env -u RUSTC_WRAPPER cargo check`，回写文档/devlog 收口。

## 依赖
- T2 依赖 T1（先稳定网络请求层，再接共识主循环）。
- T3 依赖 T1/T2 全部完成。

## 状态
- 当前阶段：进行中（T0/T1 完成，进入 T2）。
- 阻塞项：无。
- 下一步：执行 T2。
