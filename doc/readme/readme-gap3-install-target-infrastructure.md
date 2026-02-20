# README 缺口 3 收口：模块安装目标语义（自身 / 基础设施）（设计文档）

## 目标
- 收口 README 中“模块可安装至自身或基础设施”的实现缺口。
- 在不破坏现有 `install_module_from_artifact` 行为的前提下，引入显式安装目标语义。
- 让 simulator/runtime/LLM 决策链路在安装目标上保持一致可审计。

## 范围
- In scope
  - 新增显式动作：`InstallModuleToTargetFromArtifact`。
  - 新增安装目标类型：`ModuleInstallTarget`（`self_agent` / `location_infrastructure`）。
  - `WorldEventKind::ModuleInstalled`、`DomainEvent::ModuleInstalled`、`InstalledModuleState` 增加 `install_target` 字段。
  - simulator 安装路径支持目标校验与落库。
  - runtime 安装路径支持目标透传与事件审计。
  - LLM 解析与 schema 支持新动作。
  - 覆盖 required-tier 测试（兼容旧动作 + 新动作目标语义）。
- Out of scope
  - 新增“设施级模块执行引擎”。本轮仅定义安装目标语义与审计，不新增独立设施执行时序。
  - 改写既有 `install_module_from_artifact` 为必选目标字段（保留兼容入口）。

## 接口 / 数据
### 1) 新类型
- `ModuleInstallTarget`
  - `SelfAgent`
  - `LocationInfrastructure { location_id: String }`
- 默认值：`SelfAgent`（用于历史事件反序列化与兼容逻辑）。

### 2) 新动作
- `Action::InstallModuleToTargetFromArtifact`
  - 字段：`installer_agent_id`、`module_id`、`module_version`、`wasm_hash`、`activate`、`install_target`。
- 兼容动作 `Action::InstallModuleFromArtifact` 保留：
  - 运行时等价映射为 `install_target = SelfAgent`。

### 3) 事件与状态
- `WorldEventKind::ModuleInstalled` 增加 `install_target`。
- `DomainEvent::ModuleInstalled` 增加 `install_target`。
- `InstalledModuleState` 增加 `install_target`。

### 4) 规则
- simulator：
  - `SelfAgent`：沿用现有 owner 校验。
  - `LocationInfrastructure`：
    - `location_id` 必须存在；
    - 安装者必须位于该 location（避免越权跨域安装）。
- runtime：
  - 接受并记录 `install_target`，用于共识后审计与回放一致性。

### 5) 兼容性
- 历史 `ModuleInstalled` 事件无 `install_target` 时通过 `serde(default)` 回填 `SelfAgent`。
- 旧动作、旧测试、旧 replay 数据可继续工作。

## 里程碑
- M1：T0 文档建档（本设计 + 项目管理文档）。
- M2：T1 数据模型与事件结构扩展完成。
- M3：T2 runtime/simulator 安装流程支持目标语义并完成 LLM 入口接入。
- M4：T3 测试回归、文档/devlog 收口。

## 风险
- 兼容风险：事件结构扩展可能影响旧快照/回放，需 `serde(default)` 和 replay 测试兜底。
- 行为风险：`location_infrastructure` 约束过严会影响策略可用性，需在 reject reason 提示清晰。
- 维护风险：新旧安装动作并存可能产生分叉语义，需在标签与测试中保持等价映射。
