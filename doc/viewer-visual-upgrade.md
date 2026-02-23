# Viewer 视觉升级设计文档（Phase 10 后增量强化）

## 背景
- 截至 Phase 10，Viewer 的新手体验、Theme Runtime、industrial_v2 主题包与发布回归链路已经闭环，当前工程状态不是“从 0 到 1 的可玩化”，而是“已可发行基线上的增量强化”。
- 当前渲染层已具备可扩展能力（render profile、主题包、材质参数、运行中切换），但部分视觉目标仍缺少模拟层直连数据，例如 Agent 速度与运动耗时语义。
- 当前 Location 的开采/破损语义应以 `fragment_budget` 为准，设计中不再引入未定义变量（如 `remaining_estimate`）。
- 材料差异化在现状中是“部分可见、尚未完全拉开”，后续需要作为增强阶段完成“不同材料明确不同视觉效果”的最终目标。

## 目标

### 核心目标
- 将本次工作定义为 **Phase 10 后增量视觉强化**，在既有稳定基线上迭代，不重启大规模重构。
- 在基础模拟层补齐 Viewer 所需字段与语义，重点补齐 Agent 速度概念与“运动需要时间”的行为模型。
- 统一 Location 破损/衰减语义到 `fragment_budget`，确保设计与现有数据源一致。
- 分阶段实现材料差异化目标：先保证语义与链路正确，再在增强阶段拉开材质/形状/光效差异。
- 为新增视觉开关补齐解析与兼容策略，避免“文档有配置、代码无解析”。

### 非目标
- 不引入第三方 DCC 工具链自动化流程（如 Blender/Maya 自动导入流水线）。
- 不重建 mesh 资产生产方式（继续使用 `scripts/generate-viewer-industrial-theme-assets.py` 工程化生成）。
- 不引入大型角色动画系统、粒子系统或音频系统。

---

## 范围

### 阶段划分

#### S0：基线对齐阶段（设计/数据语义校准）
- 对齐现有工程实体与渲染入参，清理与代码不一致的字段假设。
- 统一 Fragment/破损描述为 `fragment_budget` 语义。
- 列出新增配置项及其解析落点（`viewer_3d_config.rs` / Theme Runtime 适配点）。

#### S1：模拟层补字段阶段（能力补齐）
- 在模拟层新增 Agent 运动学字段（速度、目标、剩余路程/预计到达 tick）。
- 移动行为由“位置瞬移”升级为“按 tick 推进的耗时移动”。
- 输出可供 Viewer 消费的稳定快照字段与事件。

#### S2：视觉增强阶段（可作为增强阶段推进）
- Agent：方向指示、速度驱动视觉反馈（轨迹/发光强度）。
- Location：材料差异化、辐射强度可视化、基于 `fragment_budget` 的破损层级。
- Asset/Power 设施：状态映射（容量、效率、数量、类型）可视化。
- Chunk：边界可见性与状态着色稳定化。

#### S3：质量与验收阶段
- required-tier 与 full-tier 测试收口。
- Web 闭环（Playwright）与性能回归收口，纳入 `testing-manual.md` 约束。

### 不包含内容
- 新主题美术风格大迁移（继续以 industrial_v2 为主）。
- 大型动画、复杂物理特效、音频反馈系统。

---

## 接口/数据

### 1. 现有数据源对齐（当前工程事实）

| 实体 | 当前数据源 | 当前可直接驱动视觉的数据 |
|---|---|---|
| Agent | `world_model::Agent` | `id`/`pos`/`body.height_cm`/`location_id` + `module_visual_entities` 派生 `module_count` |
| Location | `world_model::Location` + `types::LocationProfile` | `material`/`radius_cm`/`radiation_emission_per_tick`/`fragment_budget` |
| Asset | `world_model::Asset` | `owner`/`kind`/`quantity` |
| PowerPlant | `power::PowerPlant` | `capacity_per_tick`/`efficiency`/`degradation` |
| PowerStorage | `power::PowerStorage` | `capacity`/`current_level`/`charge_efficiency`/`discharge_efficiency` |
| Chunk | `ChunkState + bounds` | `state`/边界坐标 |

> 说明：Fragment 当前以 `Location.fragment_profile/fragment_budget` 形式挂载，不作为独立顶层实体流转。

### 2. S1 新增模拟层字段（补齐缺失）

#### 2.1 Agent 运动学字段（建议）
```rust
pub struct AgentKinematics {
    pub speed_cm_per_tick: i64,            // 速度下限保护 >0
    pub move_target_location_id: Option<LocationId>, // 目标 Location（用于多 tick 续走）
    pub move_target: Option<GeoPos>,       // 当前目标点
    pub move_started_at_tick: Option<u64>,
    pub move_eta_tick: Option<u64>,        // 预计到达 tick
    pub move_remaining_cm: i64,            // 剩余路程
}
```

#### 2.2 行为语义（必须）
- 移动不再瞬移：每 tick 按 `speed_cm_per_tick` 推进位置。
- 路程与耗时：`required_ticks = ceil(distance_cm / speed_cm_per_tick)`。
- 到达判定：`remaining_cm <= 0` 时完成移动并清理目标态。
- 若 speed 非法（`<=0`），在内核层返回可观测错误并保持状态不变。

#### 2.3 快照/事件（建议）
- 快照新增 `agent_kinematics` 可见字段，供 Viewer 直接消费。
- 事件新增（或等价事件字段扩展）：`MoveStarted`、`MoveProgressed`、`MoveArrived`。

### 3. Fragment/破损统一语义（替换旧口径）

#### 3.1 统一计算口径
```text
total_mass_g = sum(fragment_budget.total_by_element_g)
remaining_mass_g = sum(fragment_budget.remaining_by_element_g)
remaining_ratio = clamp(remaining_mass_g / total_mass_g, 0.0, 1.0)
damage_ratio = 1.0 - remaining_ratio
```

#### 3.2 视觉分级
- `damage_ratio < 0.2`：无破损。
- `0.2..0.5`：轻度裂纹/轻度暗化。
- `0.5..0.8`：重度裂纹 + 明显暗化。
- `> 0.8`：强破损（形变/塌陷感，可按档位降级）。

> 说明：删除 `remaining_estimate` 相关描述，统一使用 `fragment_budget`。

### 4. 材料差异化目标（增强阶段落地）

| 材料 | 最终目标（S2） | 当前状态 | 落地策略 |
|---|---|---|---|
| Silicate | 粗糙灰褐、低金属、棱角感 | 已有基础 | 补细节贴图与粗糙度层级 |
| Metal | 高金属、低粗糙、反射高光 | 已有基础 | 拉开与 Carbon 对比 |
| Ice | 低粗糙、半透明/冷色发光感 | 已有基础 | 强化冰材质与 halo 协同 |
| Carbon | 深色高粗糙、低反射、颗粒感 | 与 Metal 存在复用 | 新增专属材质槽与贴图 |
| Composite | 复合分层材质、局部金属点缀 | 与 Silicate 存在复用 | 新增专属混合参数/纹理 |

> 最终目标保持不变：不同材料必须有可辨识视觉效果。当前复用可作为过渡，不是终态。

### 5. Viewer 配置项与解析配套

#### 5.1 新增/补齐配置项
```bash
AGENT_WORLD_VIEWER_VISUAL_EFFECTS=minimal|standard|enhanced
AGENT_WORLD_VIEWER_AGENT_VARIANT_PALETTE=<color1>,<color2>,<color3>,<color4>
AGENT_WORLD_VIEWER_AGENT_DIRECTION_INDICATOR=1|0
AGENT_WORLD_VIEWER_AGENT_SPEED_EFFECT=1|0
AGENT_WORLD_VIEWER_AGENT_TRAIL_ENABLED=1|0
AGENT_WORLD_VIEWER_LOCATION_RADIATION_GLOW=1|0
AGENT_WORLD_VIEWER_LOCATION_DAMAGE_VISUAL=1|0
AGENT_WORLD_VIEWER_ASSET_QUANTITY_VISUAL=1|0
AGENT_WORLD_VIEWER_ASSET_TYPE_COLOR=1|0
```

#### 5.2 解析与兼容要求
- 在 `viewer_3d_config.rs` 增加解析与默认值，非法值回退默认档位。
- 与 Theme Runtime 协同的项，需在运行时应用链路中支持刷新（必要时触发场景重建）。
- 兼容策略：未配置时保持当前视觉行为不变，避免破坏既有场景与回归用例。

---

## 里程碑

### M0：设计与语义对齐（S0）
- 完成本文档修订，口径统一到当前工程事实。
- 明确模拟层补字段清单与配置解析落点。

### M1：模拟层运动学补齐（S1）
- 引入 `AgentKinematics` 字段与默认值。
- 移动行为改为按 tick 推进，补齐边界测试。
- 输出快照可见字段（含速度/剩余路程/ETA）。

### M2：视觉增强一期（S2）
- Agent 方向与速度反馈落地。
- Location `fragment_budget` 破损映射落地。
- 新增配置项解析、开关联动与回归测试。

### M3：视觉增强二期（S2）
- 完成 Carbon/Composite 材质专属化，收敛材料差异化目标。
- Asset/Power 状态可视化增强（数量/容量/效率）。

### M4：验收收口（S3）
- required/full 测试通过。
- Playwright Web 闭环与性能基线通过。
- 文档、手册、项目状态与 devlog 收口。

---

## 风险

### 技术风险
- 风险：移动语义从瞬移改为耗时推进，可能影响现有行为与测试假设。
- 缓解：分层改造（内核->快照->Viewer），并在每层补边界测试。

### 性能风险
- 风险：速度轨迹、额外发光与破损细节导致 FPS 下滑。
- 缓解：`minimal|standard|enhanced` 分档；默认 `standard`，高成本效果可关闭。

### 一致性风险
- 风险：新增配置在 native/web 行为不一致。
- 缓解：统一走 `viewer_3d_config` 解析，Web 闭环用 Playwright 复测。

### 兼容风险
- 风险：新增模拟字段影响历史快照反序列化。
- 缓解：新增字段全部提供 `serde(default)` 与向后兼容默认值。
