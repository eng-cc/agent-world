# Agent World Simulator：Chat Panel Agent Prompt 默认值内联到输入框（设计文档）

## 目标
- 将 `system prompt`、`短期目标`、`长期目标` 的默认值直接显示在对应输入框内（占位文本）。
- 移除输入框下方单独的“默认值提示”行，降低视觉噪音。
- 保持现有语义不变：输入框仍编辑 override，留空表示不覆盖（走默认值）。

## 范围

### 范围内
- 调整 Chat Panel `Agent Prompt Draft` 三个输入框的 `hint_text` 为对应默认值。
- 删除默认值单独行渲染函数与相关测试。
- 更新 viewer 手册对默认值显示方式的描述。

### 范围外
- 不改动 `prompt_control.apply` 请求协议。
- 不改动 override patch 判定逻辑。
- 不增加新的后端接口。

## 接口 / 数据
- 默认值来源保持：
  - `DEFAULT_LLM_SYSTEM_PROMPT`
  - `DEFAULT_LLM_SHORT_TERM_GOAL`
  - `DEFAULT_LLM_LONG_TERM_GOAL`
- UI 行为变化：
  - 由“输入框下方额外说明”切换为“输入框占位显示默认值”。

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：输入框占位改造与回归完成。
- M3：手册、devlog、项目状态收口完成。

## 风险
- 风险：占位文本会在用户开始输入后消失，可能影响默认值可见性。
  - 缓解：默认值通过“清空输入框”即可再次显示；必要时后续再加可选 tooltip。
