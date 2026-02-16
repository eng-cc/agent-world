# Agent World Simulator：Chat Panel Agent Prompt 字段默认值可见化（设计文档）

## 目标
- 在最右侧 Chat Panel 的 `Agent Prompt Draft` 区域中，为 `system prompt`、`短期目标`、`长期目标` 显示默认值。
- 让用户在“未设置 override”时也能直观看到实际默认提示词，不必依赖代码或环境变量排查。
- 保持现有 `apply` 语义：编辑框仍表示 override 草稿，默认值仅用于展示，不自动写入。

## 范围

### 范围内
- 在 Chat Panel 折叠区中新增三项默认值可视化文案（对应 system/short/long）。
- 默认值来源使用 `agent_world::simulator` 暴露的 `DEFAULT_LLM_*` 常量。
- 为默认值展示逻辑补充单元测试。
- 更新 viewer 手册说明该行为。

### 范围外
- 不改动 `prompt_control.apply` 请求结构。
- 不改动 override patch 判定逻辑。
- 不引入新的后端接口。

## 接口 / 数据
- 复用现有 UI 状态：`AgentChatDraftState` 的三个编辑字段仍表示 override 草稿。
- 新增 UI 辅助函数（viewer 侧）：
  - 根据字段类型输出默认值展示文本（中英文）。
- 默认值常量来源：
  - `DEFAULT_LLM_SYSTEM_PROMPT`
  - `DEFAULT_LLM_SHORT_TERM_GOAL`
  - `DEFAULT_LLM_LONG_TERM_GOAL`

## 里程碑
- M1：设计文档 + 项目管理文档完成。
- M2：Chat Panel 三字段默认值展示完成。
- M3：测试、手册、devlog 与项目状态收口完成。

## 风险
- 风险：默认值与运行时按 agent 覆盖的环境变量不一致，可能引发“显示值与实际值”认知偏差。
  - 缓解：文案标注为“系统默认值（未覆盖时使用）”，并保留“加载当前配置”按钮。
- 风险：折叠区信息增加导致密度上升。
  - 缓解：默认值采用弱强调小字号展示，不增加额外输入控件。
