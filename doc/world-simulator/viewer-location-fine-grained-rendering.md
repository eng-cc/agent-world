# Agent World Simulator：Viewer Location 细粒度渲染（设计文档）

## 目标
- 为 `location` 增加可解释的细粒度几何层级（表面细节 + 辐射外环），避免仅靠单一球体难以区分不同地点状态。
- 在不改动 simulator 数据模型的前提下，直接消费 `LocationProfile`（半径、材质、辐射）驱动渲染细节。
- 提供可复现的联调入口：新增一个开启 `asteroid_fragment` 的专用场景，便于观察大量 location 的细粒度效果。

## 范围

### In Scope
- `agent_world_viewer`：Location 实体渲染由“单球体”扩展为“主球体 + 细节子节点”。
- 细节层级依据 `radius_cm` 与 `radiation_emission_per_tick` 自动分档。
- 新增 scenario：`asteroid_fragment_detail_bootstrap`，用于密集碎片可视化调试。
- 补充单元测试，覆盖细节节点数量、辐射外环开关、场景可解析与可初始化。

### Out of Scope
- 不引入新渲染管线（实例化/LOD/后处理）。
- 不改动 `agent_world` 内核物理规则与碎片生成公式。
- 不新增 UI 面板交互，仅增强 3D 实体表现。

## 接口 / 数据

### 1) Location 渲染输入
- `spawn_location_entity(...)` 增加 `radiation_emission_per_tick: i64` 入参。
- 数据来源：
  - 快照重建：`snapshot.model.locations[*].profile.radiation_emission_per_tick`
  - 增量事件：`WorldEventKind::LocationRegistered.profile.radiation_emission_per_tick`

### 2) 细粒度渲染策略
- 主球体继续按 `radius_cm -> location_radius_m` 渲染。
- 新增 `LocationDetailProfile`（内部结构）：
  - `ring_segments`：表面细节段数（随半径分档）。
  - `halo_segments`：辐射外环段数（辐射 > 0 时启用）。
- 子节点命名约定（用于测试与排障）：
  - `location:detail:ring:<location_id>:<idx>`
  - `location:detail:halo:<location_id>:<idx>`

### 3) 测试场景
- 新增内置场景文件：`crates/agent_world/scenarios/asteroid_fragment_detail_bootstrap.json`
- 特征：
  - `asteroid_fragment.enabled = true`
  - 多 `bootstrap_chunks`
  - `origin.enabled = false`、`agents.count = 0`（聚焦 location 渲染）

## 里程碑
- **LFR1**：设计文档 + 项目管理文档。
- **LFR2**：viewer 细粒度 location 渲染落地。
- **LFR3**：新增 asteroid fragment 调试场景并补齐测试。

## 风险
- location 数量较多时，细节子节点可能放大渲染开销；通过分档与段数上限控制。
- 辐射值分布可能极端，需限制 halo 段数避免视觉噪声。
- 场景白名单工具脚本若未同步，可能导致截图闭环参数校验失败。
