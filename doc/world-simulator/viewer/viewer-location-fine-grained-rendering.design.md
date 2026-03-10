# Viewer Location 细粒度渲染设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-location-fine-grained-rendering.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-location-fine-grained-rendering.project.md`

## 1. 设计定位
定义 Location 从单球体到“主球体 + 细节子节点 + 辐射外环”的细粒度渲染方案，在不改数据模型的前提下提升不同地点状态的可解释性。

## 2. 设计结构
- 输入扩展层：`spawn_location_entity` 读取 `radiation_emission_per_tick` 与 radius 输入。
- 细节分档层：依据半径与辐射值生成 `LocationDetailProfile`，控制 ring/halo 段数。
- 子节点命名层：统一 detail/halo 命名规则，便于测试与排障。
- 调试场景层：`asteroid_fragment_detail_bootstrap` 提供密集 location 可视化入口。

## 3. 关键接口 / 入口
- `spawn_location_entity(...)`
- `LocationDetailProfile`
- `location:detail:ring:*` / `location:detail:halo:*`
- `asteroid_fragment_detail_bootstrap.json`

## 4. 约束与边界
- 不引入新渲染管线、LOD 或额外交互面板。
- 细节段数必须受控，避免 location 多时放大渲染开销。
- 辐射外环只反映已有 profile 数据，不制造新的物理含义。
- 场景白名单和截图脚本需要同步，避免闭环链路断裂。

## 5. 设计演进计划
- 先落细粒度 location 渲染与 detail profile。
- 再补调试场景与脚本白名单。
- 最后通过测试和截图闭环确认表现层稳定。
