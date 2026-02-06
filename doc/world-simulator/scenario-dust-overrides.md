# Agent World Simulator：场景 Dust 配置覆盖（设计文档）

本分册定义场景文件对小行星带碎片生成配置的覆盖方式，用于在不同场景中微调碎片分布。

## 目标
- 允许场景文件覆盖 `DustConfig` 的关键字段（先支持 `min_fragment_spacing_cm`）。
- 覆盖仅影响初始化阶段的碎片生成，保持运行时配置与行为可解释。
- 在同一 seed 下保持生成结果确定性。

## 范围

### In Scope
- 扩展 `DustInitConfig` 支持 `min_fragment_spacing_cm` 覆盖。
- `generate_fragments` 使用覆盖后的 dust 配置生成碎片。
- 补充场景文件字段说明与单元测试。

### Out of Scope
- 运行时动态调整碎片分布参数。
- 覆盖其他复杂物理参数（如 `cluster_noise`、`size_powerlaw_q`）。
- 轨道/碰撞等连续物理模拟。

## 接口 / 数据

### 场景文件结构（JSON）
```json
{
  "id": "dusty_bootstrap",
  "name": "Dusty Bootstrap",
  "seed": 42,
  "dust": {
    "enabled": true,
    "seed_offset": 77,
    "min_fragment_spacing_cm": 50000
  },
  "agents": { "count": 1 }
}
```

### DustInitConfig
- `min_fragment_spacing_cm: Option<i64>`
  - **含义**：小行星碎片最小间距（cm）。
  - **默认**：未设置时沿用 `WorldConfig.dust.min_fragment_spacing_cm`。
  - **规则**：`<= 0` 视为关闭最小间距限制。

### 应用策略
- 初始化时构造 `effective_dust`：
  - 从 `WorldConfig.dust` 拷贝；
  - 若 `DustInitConfig.min_fragment_spacing_cm` 有值，则覆盖对应字段；
  - 用 `effective_dust` 调用 `generate_fragments`。

## 里程碑
- **O1**：扩展场景 dust 覆盖字段并更新测试与文档。

## 风险
- 覆盖过大的间距可能导致碎片数量显著降低。
- 初始化使用的 dust 配置与运行时 `WorldConfig` 可能不一致，需要文档强调。
