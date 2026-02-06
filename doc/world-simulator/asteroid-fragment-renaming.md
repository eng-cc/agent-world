# Agent World Simulator：Asteroid Fragment 命名替换（设计文档）

本分册定义将现有 “dust” 概念统一替换为 “asteroid_fragment” 的方案，用于对齐世界观命名。

## 目标
- 统一命名：代码、场景文件、工具与文档中不再出现 “dust” 命名。
- 保持功能等价：仅命名替换，不改变行为与数值语义。
- 保持结构清晰：命名替换覆盖配置字段、类型、场景 ID 与输出字段。

## 范围

### In Scope
- 重命名模块、类型、字段与输出：`DustConfig/DustInitConfig/dust` 等。
- 更新场景文件与场景 ID：`dusty_*` 替换为 `asteroid_fragment_*`。
- 更新工具/测试/文档中的字段与文本。

### Out of Scope
- 物理规则与数值模型调整。
- 旧格式兼容层（不提供 `dust` 旧字段别名）。
- 历史任务日志（保持原样不改写）。

## 接口 / 数据

### 关键命名替换
- `DustConfig` -> `AsteroidFragmentConfig`
- `DustInitConfig` -> `AsteroidFragmentInitConfig`
- `WorldConfig.dust` -> `WorldConfig.asteroid_fragment`
- `WorldInitConfig.dust` -> `WorldInitConfig.asteroid_fragment`
- `dust_seed` / `dust_fragments` -> `asteroid_fragment_seed` / `asteroid_fragment_fragments`

### 场景字段
- JSON 字段：`"dust"` -> `"asteroid_fragment"`
- 场景 ID：`dusty_*` -> `asteroid_fragment_*`

## 里程碑
- **R0**：输出设计文档与项目管理文档。
- **R1**：完成代码/场景/工具/测试命名替换。
- **R2**：完成文档替换与校对。

## 风险
- 命名替换范围大，漏改将导致编译或测试失败。
- 旧场景/配置不再兼容，需要同步更新使用方。
- 历史日志保留旧词汇，无法做到完全无旧词。
