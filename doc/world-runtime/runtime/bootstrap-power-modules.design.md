# 生命出厂电力模块设计

- 对应需求文档: `doc/world-runtime/runtime/bootstrap-power-modules.prd.md`
- 对应项目管理文档: `doc/world-runtime/runtime/bootstrap-power-modules.project.md`

## 1. 设计定位
定义出厂电力模块的模块划分、行为语义、治理安装入口与运行时接入方式。

## 2. 设计结构
- 模块清单：`m1.power.radiation_harvest` 与 `m1.power.storage` 两个 wasm 工件模块。
- 行为语义：辐射采集产生缓慢补能，储能模块对移动前进行能量校验。
- 世界接入：通过 `World` 一键安装入口走治理 apply 生效，并支持幂等重装。

## 3. 关键接口 / 入口
- `World::install_m1_power_bootstrap_modules(actor)`
- 模块 manifest、事件 `ModuleEmitted/ModuleStateUpdated`
- `agent_world_builtin_wasm` 中的默认参数与模块 ID 常量

## 4. 约束与边界
- 内核只保证位置/资源/审计，供能语义由模块定义。
- 模块状态与 simulator 设施短期双轨并存，但接口语义必须稳定。
- 安装流程必须复用治理链路，不能绕过 register/activate/apply。

## 5. 设计演进计划
- 先完成设计补齐与互链回写。
- 再按项目文档任务拆解推进实现与验证。
