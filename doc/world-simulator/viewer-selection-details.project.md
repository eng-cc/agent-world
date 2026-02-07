# Viewer 选中对象详情面板（含 LLM 决策 I/O）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/viewer-selection-details.md`）
- [x] 输出项目管理文档（本文件）
- [x] 扩展 LLM 决策 trace 数据结构与行为接口
- [x] live viewer 协议新增 `DecisionTrace` 消息并在 LLM 模式推送
- [x] viewer 新增“选中对象详情”面板（Agent/Location 分支）
- [x] 详情面板接入最近事件与最近 LLM trace 展示
- [x] LLM 诊断字段扩展（模型/耗时/token/重试）
- [x] 选中对象扩展：Asset/PowerPlant/PowerStorage（3D marker + 点击 + 详情）
- [x] 选中对象扩展：Chunk（3D marker + 点击 + 详情）
- [x] 新增/更新测试（协议 round-trip + viewer UI 文案 + live trace 流）
- [x] 新增/更新测试（Asset/PowerPlant 详情文案）
- [x] 事件对象联动补齐：`ModuleVisualEntityUpserted/Removed` 映射到可点击对象
- [x] 右侧信息区增加滚动条与滚轮滚动（长内容可浏览）
- [x] 新增/更新测试（模块可视事件映射、滚动行为）
- [x] M7：修复右侧面板重叠（顶部固定区与内容区不再高度叠加）
- [x] M7：事件联动与时间轴按钮区支持换行布局，避免窄宽度下覆盖
- [x] M7：统一右侧面板视觉样式（间距、边框、背景层次）
- [x] 新增/更新测试（事件联动按钮排版回归）
- [x] M8：新增顶部控制区折叠按钮（Hide/Show Top）
- [x] M8：顶部控制区折叠状态联动面板显隐
- [x] M8：时间轴/诊断/覆盖层文本字号微调
- [x] 新增/更新测试（顶部折叠交互回归）
- [x] 更新可视化文档与总项目管理文档状态
- [x] 更新任务日志并提交

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/viewer/protocol.rs`
- `crates/agent_world/src/viewer/live.rs`
- `crates/agent_world_viewer/src/main.rs`

## 状态
- 当前阶段：M8（顶部折叠 + 文本密度优化）完成
- 下一阶段：按需增加“折叠状态持久化（local config）”与更细分区块折叠
- 最近更新：完成 M8（顶部折叠 + 字号微调 + 回归测试，2026-02-07）
