# Viewer Agent 渲染改造：模块可见 + 体积尺寸匹配

## 目标
- 将 Agent 从“单球体”改为更接近机器人形态的渲染方式，提升 3D 场景可读性。
- 尽量直接体现 Agent 的模块数量，让观察者无需展开右侧详情即可判断“模块密度”。
- 将模块渲染升级为“多个小立方体拼接”的机器人式表达，做到“一眼可见模块数量与分布”。
- 保持 Agent 体积与 `height_cm` 真实尺寸口径一致（至少高度严格对齐），避免视觉尺寸误导。

## 范围

### 范围内
- 将 Agent 主体 mesh 从球体改为纵向机体（胶囊体）渲染。
- 基于 Agent 模块数量渲染“模块立方体拼接体”（robot-like），直观看出模块数量差异。
- 模块数量优先读取 `Agent.body_state.slots` 中 `installed_module` 数量。
- 统一 Agent 尺寸映射：`height_cm -> body_height_m`，并按固定宽高比推导机体半径。
- 保持现有选择/高亮/标签机制兼容。

### 范围外
- 不改 viewer 协议。
- 不引入复杂骨骼动画或外部高模资产。
- 不改右侧详情字段结构（仅渲染层表达增强）。

## 接口 / 数据

### 1) Agent 尺寸映射
- 输入：`Agent.body.height_cm`
- 规则：
  - `body_height_m = clamp(height_cm/100, AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M)`
  - `body_radius_m = body_height_m * body_radius_ratio`（固定比例）
- 验收口径：高度映射严格随 `height_cm` 变化，2m Agent 的主体高度≈2m。

### 2) 模块数量映射
- 主数据源：`Agent.body_state.slots[*].installed_module`
- 计算：`module_count = count(installed_module.is_some())`
- 渲染：
  - 每个 Agent 渲染 `module_count` 个模块立方体（设置上限，避免极端 draw call）。
  - 立方体按预定义“机器人骨架槽位”优先放置（头/躯干/肩/臂/腿），超出后按环层补位。
  - 单个立方体对应一个模块，保证数量可解释性。

### 3) Viewer 侧结构调整
- `Viewer3dAssets` 新增 Agent 模块立方体 mesh/material 句柄。
- `AgentMarker` 扩展 `module_count` 字段，便于调试和测试断言。
- `spawn_agent_entity` 入参增加 `module_count`，并在更新路径中同步刷新模块立方体布局。

## 里程碑
- **AMR-1**：设计与项目文档完成。
- **AMR-2**：Agent 主体渲染从球体替换为胶囊体，尺寸映射落地。
- **AMR-3**：模块数量渲染接入并与 Snapshot 数据联动。
- **AMR-4**：模块渲染升级为“立方体拼接机器人”布局。
- **AMR-5**：补齐测试、截图闭环验证、文档收口。

## 风险
- 立方体数量过高导致遮挡/性能抖动：通过上限与分层布局控制缓解。
- 非常小/非常大的 Agent 宽高比失真：通过尺寸 clamp 保底。
- 事件增量场景中模块数不同步：首版先以快照数据为准，后续再补事件级精细同步。
