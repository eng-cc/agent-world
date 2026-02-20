# README 收口：基础设施模块执行引擎 + 编译 Sandbox 隔离（设计文档）

## 目标
- 收口 README 中“模块可安装至基础设施”的运行时差距：不仅记录安装目标，还要在 runtime tick 中形成可执行语义。
- 收口 README 中“世界内编译”链路的隔离差距：对源码编译入口增加 sandbox 化约束，降低环境漂移和宿主污染风险。
- 保持既有 `install_module_from_artifact` / `compile_module_artifact_from_source` 兼容，不破坏历史快照与既有动作协议。

## 范围
- In scope
  - `crates/agent_world/src/runtime`
    - `WorldState` 持久化记录模块安装目标（按 `module_id`）。
    - `ModuleInstalled` 事件应用后同步安装目标状态，并驱动 tick 路由选择。
    - tick 执行为基础设施安装目标提供独立 origin 语义（`infrastructure_tick`）。
  - `crates/agent_world/src/runtime/module_source_compiler.rs`
    - 增加源码包 sandbox 约束（文件数量/大小/路径边界）。
    - 增加编译进程超时控制与最小化环境注入（env 白名单）。
    - 失败路径返回可审计拒绝原因。
  - `test_tier_required` 测试
    - 覆盖基础设施安装目标的 tick 执行语义。
    - 覆盖编译 sandbox 拒绝与超时路径。
- Out of scope
  - 完整 OS 级隔离容器（seccomp/namespace/jail）实现。
  - 多实例同 `module_id` 同时绑定多个基础设施节点的调度模型。
  - 重写模块治理/激活协议。

## 接口 / 数据
### 1) 基础设施模块执行语义
- 数据扩展
  - `WorldState` 新增：`installed_module_targets: BTreeMap<String, ModuleInstallTarget>`。
  - 在 `DomainEvent::ModuleInstalled` 应用时更新 `installed_module_targets[module_id]`。
- tick 路由规则
  - `SelfAgent`（或未记录目标）沿用现有 `origin.kind = "tick"`。
  - `LocationInfrastructure { location_id }` 使用 `origin.kind = "infrastructure_tick"`。
  - `origin.id` 采用可审计格式：`<location_id>:<world_time>`。
- 兼容要求
  - 历史快照无新字段时默认空 map，行为等价于旧路径。

### 2) 编译 Sandbox 隔离
- 源码包约束（默认）
  - 限制文件数量、单文件大小、总字节数。
  - 路径仍仅允许相对普通路径（禁止绝对路径和 `..`）。
- 进程执行约束
  - 编译命令执行增加 timeout（默认值可配置）。
  - 编译进程使用最小化环境变量集合，显式透传必要变量。
  - `TMPDIR` 指向临时工作目录子路径，避免污染全局临时目录。
- 错误语义
  - timeout / sandbox policy 违规统一返回 `ModuleChangeInvalid`，并携带明确原因。

## 里程碑
- M1：T0 文档冻结（设计 + 项目管理）。
- M2：T1 基础设施模块执行引擎落地 + required 测试。
- M3：T2 编译 sandbox 隔离落地 + required 测试。
- M4：T3 回归（`env -u RUSTC_WRAPPER cargo check` + `CI_VERBOSE=1 ./scripts/ci-tests.sh required`）与文档/devlog 收口。

## 风险
- 兼容风险：`WorldState` 新字段需保持 `serde(default)`，避免历史快照恢复失败。
- 行为风险：tick origin 变化可能影响模块内策略判断，需要测试覆盖 `tick` 与 `infrastructure_tick` 双路径。
- 稳定性风险：编译 timeout 过小会导致误拒绝，需提供可配置并给出清晰错误。
- 可移植风险：不同主机环境下可用编译器路径差异大，env 白名单策略需保留必要变量透传。
