# Agent World：M6 3D 可视化（Bevy）

## 目标
- **大目标**：以 3D 场景观测所有 Agent 的活动，可直观查看世界状态（位置、移动、关键事件）。
- **小目标**：三节点场景（`triad_p2p_bootstrap`）在 3D viewer 中可跑通（可见 3 个节点与 3 个 Agent 的位置与移动）。
- 维持现有 2D UI 面板与回放控制，新增 3D 视图作为主画面。

## 范围
- **范围内**
  - `agent_world_viewer` 增加 3D 场景层（Bevy 3D Camera + Mesh）。
  - 复用现有 viewer 协议，使用 `WorldSnapshot` + `WorldEvent` 驱动 3D 视图。
  - 将 `GeoPos` 映射为 3D 坐标（可配置比例尺），显示 Location/Agent 标记。
  - 支持最小事件更新：LocationRegistered / AgentRegistered / AgentMoved。
  - 3D 视图与 UI 面板共存（UI 继续显示世界状态与事件列表）。
  - 简单相机控制（旋转 / 平移 / 缩放）与基础拾取（点击选中 Agent/Location）。
- **范围外**
  - 复杂模型资产、材质/贴图、地形与光照系统。
  - 多客户端协作与权限控制。
  - 海量对象的高性能渲染优化（实例化/LOD）。

## 接口 / 数据

### 数据来源
- **Viewer 协议**保持不变：`ViewerResponse::Snapshot`、`ViewerResponse::Event`。
- **Snapshot**：用于初始化 3D 世界与实体表。
- **Event**：用于增量更新实体状态（移动、注册）。

### 3D 映射
- **坐标映射**：`GeoPos { x_cm, y_cm, z_cm }` → `Vec3`，使用 `cm_to_unit` 缩放。
- **世界边界**：基于 `WorldConfig.space` 绘制边界或参考网格（可选）。

### 3D 实体
- **Location**：静态标记（低亮度球/柱），附带名称标签。
- **Agent**：动态标记（高亮球/小锥），可根据状态变色（如能量不足）。
- **索引表**：维护 `HashMap<String, Entity>`（Agent/Location id → Bevy Entity）。

### 事件映射（最小闭环）
- `LocationRegistered` → 若未存在则生成 Location 实体。
- `AgentRegistered` → 生成 Agent 实体并附加位置。
- `AgentMoved` → 更新 Agent 实体 Transform。

### 配置项（Viewer 侧）
- `Viewer3dConfig`：
  - `cm_to_unit`：空间尺度（默认 `0.00001`，1km → 1u）。
  - `show_locations` / `show_agents`：显示开关。
  - `highlight_selected`：选中高亮。

## 里程碑
- **M6.1** 3D 场景骨架与坐标映射（相机 + 基础 Mesh）。
- **M6.2** Snapshot 初始化与事件增量更新（Agent/Location 可见、移动可见）。
- **M6.3** 交互增强：相机控制与基础拾取，选中信息在 UI 显示。

## 风险
- **尺度与精度**：世界单位大、坐标精度损失；需统一缩放策略。
- **事件一致性**：事件顺序与快照不同步会导致 3D 状态错乱。
- **渲染性能**：对象数量上升时帧率下降，需要后续实例化/裁剪策略。
