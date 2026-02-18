# README 生产级缺口收口（二次）：默认 WASM 执行 + Replication RR + 分布式 Triad + 玩家节点身份（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-prod-gap1245-wasm-repl-topology-player.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：默认构建启用 `wasmtime`（缺口 1）并补齐测试/校验
- [x] T2：实现 node libp2p replication request/handler 通道（缺口 2）并补齐测试
- [x] T3：实现跨主机 `triad_distributed` 拓扑（缺口 4）并补齐 CLI/启动测试
- [ ] T4：实现玩家节点身份模型下沉（缺口 5）并补齐配置/动作/消息测试
- [ ] T5：回归验证（`env -u RUSTC_WRAPPER cargo check` + `CI_VERBOSE=1 ./scripts/ci-tests.sh required`）与文档/devlog 收口

## 依赖
- T2 与 T4 可并行，但 T4 落地后需复查 T2 的消息结构兼容。
- T3 依赖 T4 的节点身份字段（`player_id`）用于 triad role 默认绑定。
- T5 依赖 T1/T2/T3/T4 全部完成后统一执行。

## 状态
- 当前阶段：进行中（T0/T1/T2/T3 已完成，T4~T5 待执行）。
- 阻塞项：无。
- 下一步：执行 T4（玩家节点身份模型下沉）。
