# Compound/Hardware 硬迁移：从内建 ResourceKind 移除并转向 WASM 定义（设计文档）

## 目标
- 按“直接迁移、直接移除、无兼容方案”原则，移除内建 `ResourceKind` 中的 `Compound` / `Hardware`。
- 将核心资源模型收敛为最小内建资源：`Electricity` 与 `Data`。
- 清理核心层（simulator/runtime/viewer/README）对 `Compound`/`Hardware` 的内建资源依赖，避免继续由内核维护该语义。

## 范围
- 修改 `ResourceKind` 枚举与相关匹配逻辑。
- 修改 LLM 资源解析与提示中对 `compound/hardware` 的内建资源入口（移除，不做别名兼容）。
- 修改 viewer 与 demo 中对内建 `Compound`/`Hardware` 资源展示统计的代码。
- 修改 README 资源模型描述。
- 更新/修复受影响测试，确保 required 门禁通过。

## 非范围
- 不提供旧快照/旧输入在 `Compound/Hardware` 语义下的自动兼容迁移。
- 不在本次直接实现完整“模块资产标准接口”；本次目标是内核移除内建语义。

## 接口/数据
- 变更前：`ResourceKind = Electricity | Compound | Hardware | Data`
- 变更后：`ResourceKind = Electricity | Data`
- 行为约束：
  - 核心层不再接受 `compound`/`hardware` 作为内建资源 kind。
  - 相关经济或生产语义需由 WASM 模块层定义与维护。

## 里程碑
- M1：T0 文档建档。
- M2：T1 代码迁移（枚举、解析、核心逻辑、viewer、README）。
- M3：T2 测试收口 + 文档/devlog 回写。

## 风险
- 测试破坏风险：大量测试默认包含 `Compound/Hardware` 断言，需要同步迁移。
- 行为漂移风险：收敛到最小资源后，部分工业闭环路径会变更。
- 文档一致性风险：历史设计文档仍会提及旧资源模型；本轮以 README 与当前实现一致性为准。
