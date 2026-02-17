# Agent World Runtime：WASM 运行时激进迁移（设计文档，Phase 5）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 目标
- 继续推进 WASM 运行时数据类型下沉，将模块注册表与生命周期事件结构从 `agent_world` 迁移到 `agent_world_wasm_abi`。
- 让 `agent_world` 的 `runtime::modules` 收敛为兼容层（re-export），降低主 crate 对模块 ABI 类型定义的耦合。

## 范围

### In Scope（本次）
- 迁移以下结构到 `agent_world_wasm_abi`：
  - `ModuleRegistry`
  - `ModuleRecord`
  - `ModuleEvent`
  - `ModuleEventKind`
- 保留并迁移 `ModuleRegistry::record_key` 逻辑，确保注册键计算规则不变。
- `agent_world` 继续通过 `runtime::modules` 对外导出同名类型，保持现有导入路径兼容。
- 回归验证模块治理/回放/模块存储等依赖路径。

### Out of Scope（本次不做）
- 迁移 `ModuleStore`（文件落盘实现仍属于 runtime 基础设施）。
- 调整治理状态机语义（propose/shadow/apply）与审计事件模型。
- 变更网络同步协议或事件日志格式版本。

## 接口 / 数据
- `agent_world_wasm_abi` 新增模块注册表与生命周期事件结构定义。
- 跨 crate 字段保持同名同语义；时间/事件/提案编号保持 `u64` 序列化语义不变。
- `agent_world` 保留兼容导出，避免外部调用方改导入路径。

## 里程碑
- **R5-0**：文档与任务拆解完成。
- **R5-1**：模块注册表与生命周期事件类型迁移到 ABI crate，并完成回归。

## 风险
- 若迁移时字段或 serde tag 发生漂移，会影响历史事件回放兼容。
- 若 re-export 覆盖不完整，可能引入编译错误或外部接口回归。
