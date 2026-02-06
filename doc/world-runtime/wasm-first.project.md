# Agent World Runtime：WASM First（除位置/资源/基础物理外全模块化）（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime/wasm-first.md`）
- [x] 输出项目管理文档（本文件）

### 1. Kernel 不变量与规则边界
- [x] 定义 Kernel 不变量清单（位置/资源/基础物理）与守卫策略
- [x] 明确基础物理与规则模块的职责分界与示例

### 2. ModuleManifest/Subscription 扩展
- [x] 增加 `ModuleRole` 与订阅 `stage` 字段
- [x] 更新 wasm-interface/runtime-integration 文档与示例

### 3. Rule Modules 路由与合并
- [x] 实现 pre_action/post_action 路由流程
- [x] 定义 RuleDecision 结构与冲突合并策略
- [x] 定义 RuleDecisionRecorded/ActionOverridden 事件与审计导出
- [x] 订阅 stage 与 event/action kinds 组合校验与默认值策略
- [x] 明确 ResourceDelta 资源类型/单位与余额不足语义
- [x] 接入资源扣费与 Action 覆盖逻辑

### 4. Body Modules 机体模块化
- [ ] 定义 `BodyKernelView` 与 `BodyAttributesUpdated/Rejected` 事件
- [ ] 内核守卫校验（范围/变化率/上限）
- [ ] 机体/零件动作与资源消耗通过 Body Module 表达

### 5. M1 规则迁移
- [x] 5.1 移动规则迁移为 Rule Module（成本/同位拒绝，内置模块先行）
- [x] 5.2 可见性规则迁移为 Rule Module（观测快照，内置模块先行）
- [ ] 5.3 交互/资源转移规则迁移为 Rule Module
- [ ] 5.4 保持内核仅执行几何与资源守恒（移除旧规则）

### 6. 测试与回放
- [ ] Rule 模块冲突/拒绝/覆盖的确定性测试
- [ ] Body 模块更新守卫与回放一致性测试
- [ ] 治理升级（shadow/approve/apply）与审计覆盖

## 依赖
- `crates/agent_world` runtime 与模块治理基础设施
- `doc/world-runtime/wasm-interface.md`
- `doc/world-runtime/runtime-integration.md`

## 状态
- 当前阶段：W4（迁移 M1 规则到 Rule Modules）
- 下一步：W4.3（交互/资源转移规则迁移为 Rule Module）
- 最近更新：可见性规则迁移为 Rule Module（2026-02-06）
