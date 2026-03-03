> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-24

# Rust 超限文件拆分（设计文档）

## 目标
- 将超出仓库约束（单文件 >1200 行）的 Rust 文件拆分为可维护模块，且不改变外部行为。
- 本次至少收口以下两处：
  - `crates/agent_world/src/simulator/llm_agent.rs`
  - `crates/agent_world/src/viewer/live.rs`

## 范围
- In scope
  - `llm_agent`：将行为实现方法按职责迁移到 `llm_agent/` 子模块文件（prompt/guardrails/runtime helper），保留公共 API 与测试入口不变。
  - `viewer/live`：将内部辅助函数迁移到 `viewer/live/` 子模块文件，保留对外 `ViewerLiveServer` 行为不变。
  - 回归 `cargo check` 与定向 required tests。
- Out of scope
  - 业务语义重构、接口改名、协议变更。
  - 非超限文件的额外整理。

## 接口 / 数据
- 对外接口保持兼容，不新增对外公开类型。
- 仅做模块内代码搬迁：
  - `LlmAgentBehavior` 的方法实现拆分到多个 `impl` 文件。
  - `viewer/live` 内辅助函数迁移为 `pub(super)` 级别子模块函数。
- 不修改序列化字段、不修改网络协议、不修改测试断言语义。

## 里程碑
- M1：完成文档与任务拆解（T0）。
- M2：完成 `llm_agent` 拆分并通过定向回归（T1）。
- M3：完成 `viewer/live` 拆分并通过定向回归（T2）。
- M4：统一回归与文档/devlog 收口（T3）。

## 风险
- 方法搬迁风险：漏搬或重复定义导致编译失败。
- 可见性风险：子模块函数可见性不当导致调用失败。
- 测试稳定性风险：提交钩子 required 流程较长，需保证每任务提交前本地最小回归通过。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
