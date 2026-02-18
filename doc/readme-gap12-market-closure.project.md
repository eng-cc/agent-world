# README 高优先级缺口收口：模块交易 + 动态电价（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap12-market-closure.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：Runtime 模块交易闭环（上架/购买动作 + 事件 + 状态 + 测试）
- [ ] T2：Simulator 动态电价闭环（报价 + 自动定价 + 价格带护栏 + 测试）
- [ ] T3：回归验证（`cargo check` + 定向 tests）并回写文档/devlog

## 依赖
- Runtime：`crates/agent_world/src/runtime/events.rs`、`runtime/world/module_actions.rs`、`runtime/state.rs`
- Simulator：`crates/agent_world/src/simulator/power.rs`、`simulator/world_model.rs`、`simulator/kernel/actions.rs`
- 测试：`crates/agent_world/src/runtime/tests/module_action_loop.rs`、`crates/agent_world/src/simulator/tests/power.rs`

## 状态
- 当前阶段：T2 进行中
- 阻塞项：无
- 下一步：实现 simulator 动态电价机制并补齐测试
