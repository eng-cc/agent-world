# 游戏可玩性顶层设计（项目管理文档）

## 任务拆解

### T0 文档与结构对齐
- [x] 将顶层设计文档迁移到 `doc/game/`：`doc/game/gameplay-top-level-design.md`
- [x] 将工程设计分册迁移并重命名为语义化文件：`doc/game/gameplay-engineering-architecture.md`
- [x] 修复工程设计分册 Markdown 围栏问题，确保文档可正常渲染

### T1 顶层设计字段补齐
- [x] 在顶层设计文档中补齐必备字段：目标、范围、接口/数据、里程碑、风险
- [x] 在工程设计分册中补齐范围、接口/数据、里程碑、风险

### T2 设计评审准备
- [ ] 组织一次可玩性评审，确认微/中/长循环是否可验证
- [ ] 将“爽点曲线”映射为可量化指标（留存、冲突频次、联盟活跃度）
- [ ] 对战争与政治机制补充最小可行数值基线（成本/收益/冷却约束）

### T3 工程落地拆解（下阶段）
- [x] 落地 Gameplay Runtime 治理闭环首个生产切片（`doc/game/gameplay-runtime-governance-closure.md`）：ABI gameplay 元数据、Runtime 校验、mode+kind 槽位冲突检测、就绪度报告与测试
- [ ] 拆解 WASM Gameplay Kernel API 的实现任务（读取/提案/事件总线）
- [ ] 拆解 War/Governance/Crisis/Economic/Meta 模块 MVP 任务
- [ ] 为每个模块定义 `test_tier_required` 与 `test_tier_full` 测试矩阵

## 依赖

- 运行时与模块治理基线：`doc/world-runtime.md`
- 测试流程与分层矩阵：`testing-manual.md`
- 世界规则与边界约束：`world-rule.md`

## 状态

- 当前状态：`进行中（设计阶段）`
- 已完成：文档归位、命名语义化、必备字段补齐、工程分册格式修复
- 未完成：设计评审、量化指标固化、代码与测试任务拆解
- 阻塞项：暂无硬阻塞，待进入实现阶段后按模块优先级推进
