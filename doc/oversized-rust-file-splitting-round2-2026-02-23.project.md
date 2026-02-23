# Rust 超限文件拆分（第二轮，2026-02-23）项目管理

## 任务拆解
- [x] T0：输出设计文档（`doc/oversized-rust-file-splitting-round2-2026-02-23.md`）与本项目管理文档
- [x] T1：拆分 `crates/agent_world_distfs/src/challenge.rs` 的测试模块并回归
- [ ] T2：拆分 `crates/agent_world_consensus/src/pos.rs` 的测试模块并回归
- [ ] T3：拆分 `crates/agent_world_consensus/src/membership_recovery/replay.rs` 的测试模块并回归
- [ ] T4：拆分 `crates/agent_world_consensus/src/membership_recovery/replay_archive_federated.rs` 的测试模块并回归
- [ ] T5：拆分 `crates/agent_world_node/src/replication.rs` 的测试模块并回归
- [ ] T6：统一复核行数阈值、执行补充回归并收口文档/devlog

## 依赖
- T1/T2/T3/T4/T5 依赖各文件测试模块准确迁移且 `mod tests;` 路径可解析。
- T3 需特别注意测试块位于文件中后段而非末尾，避免误改生产代码。
- T6 依赖 T1~T5 全部完成。

## 状态
- 当前阶段：进行中（T0/T1 已完成，T2 待执行）。
- 阻塞项：无。
- 下一步：执行 T2（`pos.rs` 测试模块拆分）。
