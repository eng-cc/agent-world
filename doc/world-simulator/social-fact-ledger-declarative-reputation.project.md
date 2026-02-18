# 社会系统生产级方案：事实账本 + 声明式关系层（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/world-simulator/social-fact-ledger-declarative-reputation.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展 simulator 数据结构与协议类型（Action/Event/WorldModel）
- [x] T2：实现 kernel 动作处理（发布/质疑/仲裁/撤销/声明）
- [x] T3：实现过期扫描与 replay 对应逻辑
- [x] T4：补齐 `test_tier_required` 与 `test_tier_full` 测试
- [x] T5：执行回归、更新总项目文档与 devlog 收口

## 依赖
- `crates/agent_world/src/simulator/types.rs`
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/replay.rs`
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`
- `crates/agent_world/src/simulator/tests/persist.rs`

## 状态
- 当前阶段：T0~T5 已全部完成。
- 阻塞项：无。
- 下一步：按上层策略需要接入 schema 语义解释器或治理插件（本期不包含固定评分模型）。
