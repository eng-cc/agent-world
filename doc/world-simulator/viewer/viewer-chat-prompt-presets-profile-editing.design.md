# Viewer Chat 预设区 Agent Prompt 字段编辑设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets-profile-editing.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets-profile-editing.project.md`

## 1. 设计定位
定义在 Chat Panel 预设区内直接编辑 Agent Prompt 三个核心字段（system/short-term/long-term）的方案，使聊天预设与 Agent Prompt 配置在同一入口闭环。

## 2. 设计结构
- 当前配置加载层：从当前选中 Agent 的最新 profile 载入 override 字段。
- 草稿编辑层：在预设折叠区内展示并编辑三类 Prompt 字段。
- Apply 闭环层：继续复用 `ViewerRequest::PromptControl::Apply` 完成应用。
- 单入口收敛层：与聊天预设编辑共用同一折叠区域，不恢复独立 Prompt Ops 面板。

## 3. 关键接口 / 入口
- `system_prompt_override`
- `short_term_goal_override`
- `long_term_goal_override`
- `加载当前配置`
- `应用到 Agent`
- `ViewerRequest::PromptControl::Apply`

## 4. 约束与边界
- 不恢复独立 Prompt Ops 面板。
- 本轮只做 `apply` 闭环，不扩展 rollback / preview / 持久化。
- 需要与当前选中 Agent 语义严格绑定，避免误写其他 Agent。
- 聊天预设编辑与 Agent Prompt 编辑需在同一折叠区内保持可读布局。

## 5. 设计演进计划
- 先补当前配置读取和草稿状态。
- 再接通 Apply 按钮与协议复用。
- 最后补测试与手册，固定单入口编辑体验。
