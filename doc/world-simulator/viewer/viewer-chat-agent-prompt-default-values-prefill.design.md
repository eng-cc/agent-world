# Viewer Chat Agent Prompt 默认值预填充设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-agent-prompt-default-values-prefill.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-agent-prompt-default-values-prefill.project.md`

## 1. 设计定位
定义 Chat Panel 中 `Agent Prompt Draft` 的默认值预填充与 patch 语义，使输入框直接展示当前生效值，并在“未修改/改回默认/清空”三类场景下保持可预测的 override 处理。

## 2. 设计结构
- 草稿加载层：优先加载 override，无 override 时加载系统默认值。
- patch 计算层：基于“当前 override + 默认值 + 输入值”三者计算是否生成 patch 或清除 override。
- 输入体验层：用户进入即能直接编辑文本，无需从占位符复制。
- 手册同步层：将“预填充 + 清除 override 等价语义”作为唯一权威口径。

## 3. 关键接口 / 入口
- `DEFAULT_LLM_SYSTEM_PROMPT`
- `DEFAULT_LLM_SHORT_TERM_GOAL`
- `DEFAULT_LLM_LONG_TERM_GOAL`
- `prompt_control.apply`
- override patch 辅助逻辑

## 4. 约束与边界
- 后端协议结构保持不变，只调整前端草稿加载与 patch 构造语义。
- “输入默认值”和“清空输入框”统一视为清除 override。
- 无 override 且输入值等于默认值时不生成无意义 patch。
- 本轮不扩展 preview / rollback / 本地持久化。

## 5. 设计演进计划
- 先统一草稿预填充来源。
- 再实现 patch 计算与清除 override 语义。
- 最后补测试与手册，收口默认值行为主入口。
