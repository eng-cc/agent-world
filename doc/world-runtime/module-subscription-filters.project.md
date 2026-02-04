# Agent World Runtime：模块订阅过滤器（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/module-subscription-filters.md`）
- [x] 定义过滤规则结构与匹配语义（JSON Pointer + eq）
- [x] 实现订阅过滤逻辑（事件/动作路由）
- [x] 补充测试（事件过滤、动作过滤）
- [x] 校验过滤器 schema（Shadow/Apply 拒绝非法 filters）
- [x] 补充非法 filters 的拒绝测试
- [x] 更新项目管理文档与任务日志
- [x] 运行测试 `env -u RUSTC_WRAPPER cargo test -p agent_world`
- [x] 提交到 git

## 依赖
- `ModuleSubscription` 数据结构（`crates/agent_world/src/runtime/modules.rs`）
- 模块路由实现（`crates/agent_world/src/runtime/world/module_runtime.rs`）

## 状态
- 当前阶段：完成
