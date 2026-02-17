# Agent World Runtime：WASM 运行时激进迁移（设计文档，Phase 4）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 目标
- 继续推进 WASM 运行时类型下沉，将模块清单与变更计划结构从 `agent_world` 迁移到 `agent_world_wasm_abi`。
- 让 ABI crate 承载更多跨 crate 通用定义，减少 runtime crate 对模块数据定义的独占耦合。

## 范围

### In Scope（本次）
- 迁移以下结构到 `agent_world_wasm_abi`：
  - `ModuleRole`
  - `ModuleManifest`
  - `ModuleChangeSet`
  - `ModuleActivation`
  - `ModuleDeactivation`
  - `ModuleUpgrade`
- `agent_world` 通过 re-export 继续对外暴露同名类型，保持现有调用方兼容。
- 回归验证 runtime 相关模块治理与迁移路径测试。

### Out of Scope（本次不做）
- 迁移 `ModuleRecord/ModuleEvent/ModuleEventKind`（仍依赖 runtime 时间/事件类型）。
- 变更治理流程语义、模块事件 schema、分布式同步协议。
- 调整 executor/router 运行逻辑。

## 接口 / 数据
- `agent_world_wasm_abi` 新增并导出模块清单与变更计划类型。
- `agent_world` 的 `runtime::modules` 保留 re-export，不主动破坏上层导入路径。
- 序列化语义保持不变（字段名与默认值策略不变）。

## 里程碑
- **R4-0**：文档与任务拆解完成。
- **R4-1**：模块清单与变更计划类型迁移到 ABI crate 并回归通过。

## 风险
- 类型迁移涉及序列化结构，若字段属性偏移会影响治理事件回放兼容。
- re-export 若漏改，可能导致部分测试/模块导入路径编译失败。
