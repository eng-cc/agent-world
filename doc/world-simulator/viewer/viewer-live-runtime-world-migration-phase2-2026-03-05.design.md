# Viewer Live runtime/world 接管 Phase 2 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.project.md`

## 1. 设计定位
定义 runtime/world 接管的第二阶段方案：继续扩大 runtime live 的动作、状态与诊断覆盖，让 Viewer/live 脚本在更大范围内脱离 simulator fallback。

## 2. 设计结构
- 动作桥接层：扩展 runtime live 对关键控制、交互和错误语义的覆盖。
- 状态一致性层：校齐 runtime 输出与现有 Viewer 快照/事件语义。
- 诊断拒绝层：无法安全映射的动作继续返回结构化拒绝，而非隐式回退。
- 回归对比层：通过定向测试确认 runtime/live 行为与既有主路径等价。

## 3. 关键接口 / 入口
- runtime live control plane
- Viewer 快照/事件适配层
- 结构化 `ActionRejected` 语义
- viewer live 定向回归

## 4. 约束与边界
- 本阶段继续保持协议兼容，不引入新的前端字段。
- 动作桥接优先安全映射，不能为了覆盖率牺牲行为确定性。
- simulator fallback 仍只作为阶段性保底，设计目标是为 Phase 3 单链路收敛铺路。
- 诊断输出要可脚本断言，避免运行期只剩模糊日志。

## 5. 设计演进计划
- 先扩大 runtime action/state 覆盖。
- 再补拒绝语义与等价回归。
- 最后为 Phase 3 删除旧分支准备单链路前提。
