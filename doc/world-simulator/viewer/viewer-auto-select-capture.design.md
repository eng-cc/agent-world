# Viewer 启动自动化步骤与自动选中设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-auto-select-capture.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-auto-select-capture.project.md`

## 1. 设计定位
定义 Viewer 启动后的自动化步骤系统，通过 `wait/mode/focus/pan/zoom/orbit/select` 等步骤，把截图与验证流程从“单点自动选中”扩展为可编排的多步自动链路。

## 2. 设计结构
- 自动步骤层：按步骤驱动相机状态和选中状态变化。
- 目标解析层：支持 `agent:<id>`、`location:<id>`、`first_agent`、`first_location` 等目标语法。
- 单步入口层：使用 `OASIS7_VIEWER_AUTO_SELECT*` 提供单步快捷入口。
- 脚本桥接层：截图脚本透传自动选中目标和完整步骤串。

## 3. 关键接口 / 入口
- `OASIS7_VIEWER_AUTO_SELECT`
- `OASIS7_VIEWER_AUTO_SELECT_TARGET`
- `OASIS7_VIEWER_AUTOMATION_STEPS`
- `mode/focus/pan/zoom/orbit/select/wait`
- `capture-viewer-frame.sh --auto-select-target --automation-steps`

## 4. 约束与边界
- 自动步骤只在显式配置时启用，不改变默认交互路径。
- 目标别名首版仅覆盖 agent/location，不扩展设施/chunk。
- 步骤解析失败必须有明确错误边界，不允许半解析漂移。
- 本轮不修改鼠标拾取或右侧详情协议。

## 5. 设计演进计划
- 先实现步骤语法与目标解析。
- 再接通相机/选中状态机与脚本透传。
- 最后用配置解析与截图场景回归固定自动化闭环。
