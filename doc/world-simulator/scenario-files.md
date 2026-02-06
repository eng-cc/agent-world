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
- 场景文件覆盖所有现有场景：minimal/two_bases/.../dusty_triad_region_bootstrap。
- 单元测试验证场景文件可加载与稳定性。

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
  "dust": { "enabled": false },
  "agents": { "count": 2, "location_id": "base-a" },
  "locations": [
    { "location_id": "base-a", "name": "Base A", "pos": { "type": "center_offset", "dx_pct": -0.2 } },
    { "location_id": "base-b", "name": "Base B", "pos": { "type": "center_offset", "dx_pct": 0.2 } }
  ]
}
```

### 位置表达
- `center`：空间中心点。
- `center_offset`：以中心为基准按比例偏移（`dx_pct/dy_pct/dz_pct`）。
- `absolute`：直接给出 `x_cm/y_cm/z_cm`。

### 加载策略
- 场景文件通过 `include_str!` 嵌入编译产物，避免运行时 I/O。
- `WorldInitConfig::from_scenario` 读取场景文件并生成初始化配置。

## 里程碑
- **F1**：输出场景文件设计与项目管理文档。
- **F2**：完成场景文件迁移与加载逻辑，更新测试与文档。

## 风险
- 场景文件与代码结构漂移导致解析失败。
- 相对位置表达不当引入边界越界风险。
- 场景文件修改需要重新编译才能生效。
