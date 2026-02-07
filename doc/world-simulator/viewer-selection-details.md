# Viewer 选中对象详情面板（含 LLM 决策 I/O）

## 目标
- 在 viewer 中新增“选中对象详情”面板：点击对象后显示尽量详细的信息。
- 对 Agent 详情补充 LLM 决策调试信息，至少包含最近决策的 LLM 输入（prompt）与输出（completion/错误）。
- 扩展覆盖对象：除 Agent/Location 外，新增 Asset/PowerPlant/PowerStorage/Chunk 详情能力。
- 保持离线回放兼容：无 LLM trace 数据时仍可展示基础详情并明确提示“无 LLM trace”。
- 遵循可视化总原则：通过详情面板以最直接的方式获取对象相关的模拟信息。
- 修复右侧 UI 在高信息密度下的面板重叠/挤压问题，并统一基础视觉风格。

## 范围
- **范围内**
  - `agent_world_viewer` 新增详情 UI 区块与文本渲染逻辑。
  - 选中对象范围扩展到 Asset/PowerPlant/PowerStorage/Chunk。
  - `viewer` 协议新增决策 trace 消息（用于 live 模式）。
  - `LlmAgentBehavior` 暴露最近一次决策 trace（输入/输出/解析错误）。
  - `world_viewer_live` 在 LLM 驱动模式下推送决策 trace 给 viewer。
  - 新增/更新单元测试，覆盖协议 round-trip、live trace 推送、详情文案渲染。
  - 右侧信息面板支持滚动条与滚轮滚动，避免长文本截断。
  - 事件列表对象联动补齐 `module_visual_entities`（WASM 模块可视实体）。
  - 右侧顶部控制区与下部详情区改为“可收缩+可滚动”布局，避免高度叠加导致重叠。
  - 调整事件联动区与时间轴按钮排版（支持换行），避免窄面板下按钮互相覆盖。
- **范围外**
  - 不实现历史 trace 分页/检索系统。
  - 不为非 LLM agent 伪造 LLM I/O（仅显示“非 LLM 或无 trace”）。

## 接口 / 数据
- 新增数据结构：`AgentDecisionTrace`
  - 字段：`agent_id`、`time`、`decision`、`llm_input`、`llm_output`、`llm_error`、`parse_error`
  - 诊断字段：`llm_diagnostics`（`model`、`latency_ms`、`prompt_tokens`、`completion_tokens`、`total_tokens`、`retry_count`）
- 协议扩展：`ViewerResponse::DecisionTrace { trace }`
- 详情面板展示策略：
  - Agent：基础状态（位置、坐标、资源、电力/热状态）+ 最近相关事件 + 最近 LLM trace
  - Location：基础状态（名称、坐标、资源、profile 摘要、碎片预算摘要）+ 最近相关事件
  - Asset：资产种类、数量、归属者与归属者相关事件
  - PowerPlant：设施状态（容量/输出/成本/效率）与相关电力事件
  - PowerStorage：设施状态（容量/电量/充放电参数）与相关电力事件
  - Chunk：坐标/状态/边界/资源预算摘要与 ChunkGenerated 事件
  - Module Visual（复用 Asset 选中类型）：展示 `module_id/kind/anchor/label` 与模块可视事件

## 里程碑
- **M1**：完成文档与任务拆解。
- **M2**：打通 LLM trace 采集与 live 协议下发。
- **M3**：完成 viewer 详情面板渲染与测试覆盖。
- **M4**：完成 Asset/PowerPlant/PowerStorage 详情扩展与测试覆盖。
- **M5**：完成 Chunk 详情扩展与测试覆盖。
- **M6**：补齐模块可视实体事件联动与右侧面板滚动能力。
- **M7**：修复右侧面板重叠并完成基础视觉美化（层次、间距、按钮排版）。

## 风险
- **信息量过大**：详情文本可能过长，需控制展示窗口（保留最近 N 条事件/trace）。
- **锚点偏差**：Asset/设施在 3D 中采用近邻锚点偏移渲染，需避免误导为真实坐标。
- **模式差异**：离线与 script 模式没有 LLM trace，UI 需要明确降级文案。
- **兼容性**：协议新增字段需保持向后兼容（未知消息不影响已有逻辑）。
- **可用性**：滚动区域命中与滚轮行为若实现不当，可能影响左侧 3D 相机交互。
- **布局稳定性**：固定高度 + 100% 高度混用可能导致重叠，需通过 flex 增长和滚动容器约束规避。
