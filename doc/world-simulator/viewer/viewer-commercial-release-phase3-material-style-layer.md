# Viewer 商业化发行缺口收敛 Phase 3：材质风格覆盖层

## 目标
- 在已有“外部 mesh 可替换”能力之上，补齐“材质风格可配置”能力，减少导入真实美术后出现的风格割裂。
- 允许按实体类别覆盖基础色与自发光色，提升 Viewer 在不同发行主题（工业风、冷色科幻、沙盒写实）下的快速调优能力。
- 保持默认行为兼容：未配置时延续当前配色与材质参数，不影响现有测试与截图基线。

## 范围

### In Scope
- 新增材质风格覆盖配置资源（agent/location/asset/power_plant/power_storage）。
- 新增环境变量入口，支持通过十六进制颜色字符串（`#RRGGBB`）覆盖基础色与自发光色。
- `setup_3d_scene` 接入覆盖逻辑（覆盖值优先、默认值兜底）。
- 补齐配置解析测试与关键材质构建逻辑测试。
- 更新 `doc/viewer-manual.md` 的材质风格配置说明。

### Out of Scope
- 本阶段不引入纹理贴图（albedo/normal/orm）自动装载管线。
- 本阶段不引入骨骼动画、后处理 LUT 资产包或风格化 shader。
- 本阶段不改动 world 数据协议与运行时语义。

## 接口 / 数据
- 新增配置资源：`ViewerExternalMaterialConfig`。
- 新增环境变量（首批）：
  - `AGENT_WORLD_VIEWER_AGENT_BASE_COLOR`
  - `AGENT_WORLD_VIEWER_AGENT_EMISSIVE_COLOR`
  - `AGENT_WORLD_VIEWER_LOCATION_BASE_COLOR`
  - `AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_COLOR`
  - `AGENT_WORLD_VIEWER_ASSET_BASE_COLOR`
  - `AGENT_WORLD_VIEWER_ASSET_EMISSIVE_COLOR`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_BASE_COLOR`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_COLOR`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_COLOR`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_COLOR`
- 值语义：`#RRGGBB`（大小写不敏感，不接受短格式）。
- 运行时策略：
  - 有合法配置：使用覆盖颜色。
  - 无配置或配置非法：使用当前默认颜色。

## 里程碑
- VCR3-0：设计文档与项目管理文档建档。
- VCR3-1：材质风格覆盖配置结构与环境变量解析落地。
- VCR3-2：`setup_3d_scene` 接入覆盖逻辑并补测试。
- VCR3-3：手册、项目状态与 devlog 收口。

## 风险
- 颜色配置误填导致观感异常：
  - 缓解：仅接受严格 `#RRGGBB`；非法值自动回退默认色。
- 覆盖过多导致基线截图漂移：
  - 缓解：默认不启用覆盖；保留 snapshot 基线脚本作为回归门禁。
- 配置项扩张增加运维复杂度：
  - 缓解：按实体类别收敛命名，统一前缀，手册给出最小示例。
