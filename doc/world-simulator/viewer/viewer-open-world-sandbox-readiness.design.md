# Viewer 开放世界沙盒可玩化准备设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-open-world-sandbox-readiness.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-open-world-sandbox-readiness.project.md`

## 1. 设计定位
定义 Viewer 从调试器优先演进到开放世界运营终端的双模式信息架构、Prompt Ops 控制面与规模化退化策略。

## 2. 设计结构
- 模式层：`Observe / Prompt Ops` 双视图信息架构。
- 控制层：Prompt 预览、提交、回滚与版本审计。
- 性能层：事件窗口、标签 LOD、预算监控与自动降级。

## 3. 关键接口 / 入口
- `ViewerRequest/ViewerResponse` 的 `prompt_control.*` 协议
- `AgentPromptProfile` / `AgentPromptUpdated` 事件
- 预算指标与运营态 UI 面板

## 4. 约束与边界
- 玩家交互仅限 Prompt 修改，不开放直接动作协议。
- Prompt 变更必须可回滚、可审计、可回放。
- 规模化退化必须有预算指标支撑，避免盲降级。

## 5. 设计演进计划
- 先完成 Design 补齐与互链回写。
- 再沿项目文档推进实现与验证。
