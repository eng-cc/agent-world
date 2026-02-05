# Agent World Simulator：世界初始化（设计分册）

本分册描述世界初始化（World Initialization）的最小实现，用于在不依赖外部输入的情况下生成可运行的初始世界状态。

## 目标
- 提供**确定性**的世界初始化流程：同一配置与种子产生一致的世界模型。
- 支持最小可运行世界：原点地点 + 初始 Agent + 可选尘埃云碎片。
- 初始化产物可直接用于 `WorldKernel`，并保持时间与事件日志语义清晰。

## 范围

### In Scope（V1）
- `WorldInitConfig`：初始化参数（seed、原点地点、尘埃云生成、初始 Agent）。
- 原点地点创建：默认位于空间中心，可配置位置与画像。
- 尘埃云碎片生成：复用 `generate_fragments`，并支持单独 seed 偏移。
- 初始 Agent 生成：按数量与前缀批量创建，默认出生在原点地点。
- 初始化输出：生成 `WorldModel`，并提供 `WorldKernel` 便捷构造。
- 基础校验：越界位置、ID 冲突、出生地点不存在等。

### Out of Scope（V1 不做）
- 初始设施/资产/合约/交易等复杂社会系统初始化。
- 多阵营/多区域的高级场景模板。
- 大规模地图分片与并行生成。

## 接口 / 数据

### 关键结构
- `WorldInitConfig`
  - `seed: u64`
  - `origin: OriginLocationConfig`
  - `dust: DustInitConfig`
  - `agents: AgentSpawnConfig`
- `OriginLocationConfig`
  - `enabled: bool`
  - `location_id/name/profile`
  - `pos: Option<GeoPos>`（空则取空间中心）
- `DustInitConfig`
  - `enabled: bool`
  - `seed_offset: u64`
- `AgentSpawnConfig`
  - `count: usize`
  - `id_prefix: String`
  - `start_index: u32`
  - `location_id: Option<LocationId>`（空则用 origin）
- `WorldInitReport`
  - 统计创建数量（locations/agents）与使用的 seed
- `WorldInitError`
  - 越界位置、ID 冲突、出生地点缺失等错误

### 初始化语义
- 初始化在**时间 0** 完成：
  - `WorldKernel.time = 0`
  - `journal` 为空（初始化视为“世界既成事实”）。
- 生成过程使用 `WorldConfig` 的 `space/dust/power` 配置。
- 生成顺序：origin → dust fragments → agents（确保出生地点已存在）。

## 里程碑
- **I1**：定义初始化配置与报表结构，输出 `WorldModel`。
- **I2**：接入 `WorldKernel` 便捷构造与基础校验；补充单元测试。
- **I3**：扩展初始化模板（设施/资源/多据点）与更多可配置项。

## 风险
- 默认尘埃云生成可能导致初始化耗时波动（需可配置关闭）。
- 依赖浮点随机数的确定性：需确保同平台可复现。
- 初始化不写入事件日志，可能影响“完整事件回放”场景。
