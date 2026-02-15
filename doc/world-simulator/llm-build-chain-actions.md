# LLM 建造链路动作扩展（llm_bootstrap）

## 目标
- 以 `llm_bootstrap` 为示例场景，将 LLM 可决策动作从“移动/采集”扩展到“调拨/精炼”，形成可连续执行的建造链路。
- 以“做出制成品”为目标口径，落地当前 simulator 能力下的制成品闭环：`electricity -> hardware`（`RefineCompound`），并支持跨 Agent/Location 的资源调拨。
- 保持 Web 端闭环验证流程（Playwright）可用，确保功能改动不会破坏线上 viewer 调试链路。

## 范围
- In Scope:
  - 扩展 LLM 决策 JSON：支持 `transfer_resource`、`refine_compound`。
  - 扩展 `execute_until` 对新增动作的兼容路径（沿用既有机制）。
  - 更新 Prompt 输出约束与推荐模板，避免 LLM 输出歧义字段。
  - 补充单元测试与 Web 闭环验证。
- Out of Scope:
  - 不引入 runtime M4 工业动作到 simulator（`BuildFactory*`/`ScheduleRecipe*` 暂不接入 LLM）。
  - 不调整 viewer 协议与渲染结构。

## 接口 / 数据

### 1) LLM 决策 JSON 扩展
- `transfer_resource`
  - 字段：
    - `from_owner`: `self | current_location | agent:<id> | location:<id>`
    - `to_owner`: `self | current_location | agent:<id> | location:<id>`
    - `kind`: `electricity | hardware | data`
    - `amount`: 正整数
- `refine_compound`
  - 字段：
    - `owner`: `self | current_location | agent:<id> | location:<id>`（缺省为 `self`）
    - `compound_mass_g`: 正整数

### 2) 解析策略
- 所有 owner 语法统一映射到 `ResourceOwner`。
- `self` 默认映射为当前 agent。
- `current_location` 默认映射为当前 agent 所在 location（由 observation 上下文提供）。
- 非法 owner / 非法资源类型 / 非法数量统一降级为 `Wait` 并写入 `parse_error`。

### 3) Prompt 约束
- 在 `[Decision JSON Schema]` 中显式新增两类动作。
- 提供最小推荐模板（transfer/refine）。
- 保留“单 JSON 输出 + execute_until 仅最终阶段可用”硬约束。

## 里程碑
- LBA-1：文档建模与任务拆解。
- LBA-2：LLM 解析与序列化扩展（含错误处理）。
- LBA-3：Prompt schema 与测试回归。
- LBA-4：Web 闭环验证（console error=0 + screenshot）。

## 风险
- LLM 字段漂移导致解析失败：通过严格 schema 和 parse_error 回写缓解。
- 新动作引发循环执行风险：依赖既有 `execute_until` 上限与 replan 机制兜底。
- Web 闭环环境差异（Node/Playwright 版本）：沿用仓库既定 wrapper 与产物目录口径。
