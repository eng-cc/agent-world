# Agent World Simulator：场景文件化（设计文档）

本分册描述将 `WorldScenario` 的内置定义迁移为**场景文件**，用于提升可维护性与可扩展性。

## 目标
- 将现有所有 `WorldScenario` 定义迁移为**场景文件**（JSON），作为单一来源。
- 保持现有 API（`WorldInitConfig::from_scenario` / `WorldScenario::parse` 等）可用。
- 提供最小位置表达（center/center_offset/absolute），在文件中表达相对位置。

## 范围

### In Scope
- `crates/agent_world/scenarios/*.json` 作为默认场景文件集合。
- `WorldScenario` 从场景文件加载配置（include_str 嵌入编译产物）。
- 位置表达 `ScenarioPos`：`center` / `center_offset` / `absolute`。
- 场景文件覆盖所有现有场景：minimal/two_bases/.../asteroid_fragment_triad_region_bootstrap。
- 单元测试验证场景文件可加载与稳定性。
- `world_init_demo` 支持 `--scenario-file` 从 JSON 文件加载场景。

### Out of Scope
- 运行时从任意路径加载自定义场景文件（仅内置文件）。
- 复杂场景 DSL（条件生成、随机分布等）。
- 场景文件的版本迁移工具。

## 接口 / 数据

### 场景文件结构（JSON）
```json
{
  "id": "two_bases",
  "name": "Two Bases",
  "asteroid_fragment": { "enabled": false, "min_fragment_spacing_cm": 50000 },
  "agents": { "count": 2, "location_id": "base-a" },
  "locations": [
    { "location_id": "base-a", "name": "Base A", "pos": { "type": "center_offset", "dx_pct": -0.2 } },
    { "location_id": "base-b", "name": "Base B", "pos": { "type": "center_offset", "dx_pct": 0.2 } }
  ]
}
```

`asteroid_fragment.min_fragment_spacing_cm` 为可选字段，用于覆盖小行星碎片最小间距（cm）；未设置则沿用 `WorldConfig.asteroid_fragment`。

### 位置表达
- `center`：空间中心点。
- `center_offset`：以中心为基准按比例偏移（`dx_pct/dy_pct/dz_pct`）。
- `absolute`：直接给出 `x_cm/y_cm/z_cm`。

### 加载策略
- 场景文件通过 `include_str!` 嵌入编译产物，避免运行时 I/O。
- `WorldInitConfig::from_scenario` 读取场景文件并生成初始化配置。
- `world_init_demo --scenario-file` 走运行时加载路径，便于调试自定义场景文件。

## 里程碑
- **F1**：输出场景文件设计与项目管理文档。
- **F2**：完成场景文件迁移与加载逻辑，更新测试与文档。

## 风险
- 场景文件与代码结构漂移导致解析失败。
- 相对位置表达不当引入边界越界风险。
- 场景文件修改需要重新编译才能生效。

## 场景测试覆盖矩阵（2026-02-06）

> 目标：将“场景是否有意义”转化为可验证口径。每个场景至少对应一个**稳定断言**，避免保留“只存在但无测试价值”的场景。

| 场景 ID | 主要测试目标 | 关键覆盖测试 |
| --- | --- | --- |
| `minimal` | 最小初始化基线（origin + 默认 agent） | `scenario_specs_match_ids`、`scenario_templates_build_models`、`scenarios_are_stable`、`world_init_demo_runs_summary_only`、`world_init_demo_runs_from_scenario_file` |
| `two_bases` | 双基地拓扑与双 agent 基础分布 | `scenario_specs_match_ids`、`scenario_templates_build_models`、`scenarios_are_stable`、`scenario_aliases_parse(two-bases)` |
| `llm_bootstrap` | LLM 驱动预置基线（双站点 + 辐射 profile + data/electricity 资源） | `scenario_specs_match_ids`、`scenario_templates_build_models`、`scenarios_are_stable`、`scenario_aliases_parse(llm)`、`world_init_demo_runs_llm_bootstrap_summary` |
| `power_bootstrap` | 电力设施（plant/storage）与 owner 约束 | `scenario_specs_match_ids`、`scenario_templates_build_models`、`scenarios_are_stable`、`scenario_aliases_parse(bootstrap)` |
| `resource_bootstrap` | 资源初值注入（origin/agent） | `resource_bootstrap_seeds_stock`、`scenario_specs_match_ids`、`scenarios_are_stable`、`scenario_aliases_parse(resources)` |
| `twin_region_bootstrap` | 双区域结构（location/agents） | `twin_region_bootstrap_seeds_regions`、`scenarios_are_stable`、`scenario_aliases_parse(twin-regions)`、`plan_demo_actions_includes_move_for_multi_location_scenario` |
| `triad_region_bootstrap` | 三区域结构（location/agents/resource） | `triad_region_bootstrap_seeds_regions`、`scenarios_are_stable`、`scenario_aliases_parse(triad-regions)`、`world_init_demo_runs_triad_summary` |
| `triad_p2p_bootstrap` | P2P 节点化分布（spawn_locations） | `triad_p2p_bootstrap_seeds_nodes_and_agents`、`scenarios_are_stable`、`scenario_aliases_parse(p2p-triad)` |
| `asteroid_fragment_bootstrap` | 碎片分块生成 + bootstrap chunk + 预算账本（无默认设施） | `asteroid_fragment_bootstrap_seeds_fragments_and_resources`、`scenarios_are_stable`、`world_init_demo_runs_asteroid_fragment_summary` |
| `asteroid_fragment_twin_region_bootstrap` | 碎片分块 + 双区域结构联动（无默认设施） | `asteroid_fragment_twin_region_bootstrap_seeds_fragments_and_regions`、`scenarios_are_stable`、`world_init_demo_runs_asteroid_fragment_twin_summary` |
| `asteroid_fragment_triad_region_bootstrap` | 碎片分块 + 三区域结构联动（无默认设施） | `asteroid_fragment_triad_region_bootstrap_seeds_fragments_and_regions`、`scenarios_are_stable`、`world_init_demo_runs_asteroid_fragment_triad_summary` |

说明：
- 自 2026-02-07 起，除 `power_bootstrap` 外，内置场景不再默认注入 `power_plants`/`power_storages`；如需设施，需在场景 JSON 中显式声明。
- `scenario_specs_match_ids` 定位于 `crates/agent_world/src/simulator/scenario.rs`，用于约束“枚举 ID 与 JSON ID 一致”。
- 其余命名测试主要位于 `crates/agent_world/src/simulator/tests/init.rs` 与 `crates/agent_world/tests/world_init_demo.rs`。
- 场景矩阵应随测试变更同步更新，避免“文档保留但测试漂移”。
