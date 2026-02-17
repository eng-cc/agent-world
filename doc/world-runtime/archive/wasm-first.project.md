# Agent World Runtime：WASM First（除位置/资源/基础物理外全模块化）（项目管理文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


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
- [x] 定义 `BodyKernelView` 与 `BodyAttributesUpdated/Rejected` 事件
- [x] 内核守卫校验（范围/变化率/上限）
- [x] 机体/零件动作与资源消耗通过 Body Module 表达
  - [x] 定义 body action/emit action 与路由约定
  - [x] 内核处理 emit body attributes（守卫校验/拒绝）
  - [x] 内置 Body Module（用于测试/示例）
  - [x] 单元测试覆盖 body action + 资源成本

### 5. M1 规则迁移
- [x] 5.1 移动规则迁移为 Rule Module（成本/同位拒绝，内置模块先行）
- [x] 5.2 可见性规则迁移为 Rule Module（观测快照，内置模块先行）
- [x] 5.3 交互/资源转移规则迁移为 Rule Module（内置模块先行）
- [x] 5.4 保持内核仅执行几何与资源守恒（移除旧规则）

### 6. 测试与回放
- [x] Rule 模块冲突/拒绝/覆盖的确定性测试
  - [x] 验证规则模块调用顺序稳定（module_id 字典序）
  - [x] 冲突覆盖时拒绝动作（conflicting override）
  - [x] deny 覆盖 modify（拒绝优先）
  - [x] 一致 override 应稳定应用
- [x] Body 模块更新守卫与回放一致性测试
- [x] 治理升级（shadow/approve/apply）与审计覆盖

### 7. 收尾整理
- [x] 项目状态更新与评审准备

## 依赖
- `crates/agent_world` runtime 与模块治理基础设施
- `doc/world-runtime/wasm-interface.md`
- `doc/world-runtime/runtime-integration.md`

## 状态
- 当前阶段：W7（收尾整理/评审）
- 下一步：无（已于 2026-02-17 归档到 `doc/world-runtime/archive`）
- 最近更新：收尾整理完成（2026-02-06）
