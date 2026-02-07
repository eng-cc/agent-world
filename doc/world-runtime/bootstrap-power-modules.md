# Agent World Runtime：生命出厂电力模块（辐射发电 + 基础储能）

## 目标

- 将“硅基生命出厂自带基础供能能力”落到 runtime 的模块化机制中，避免把外部工业设施硬编码为世界初始事实。
- 以 **预置 WASM 模块** 形式提供两项基础能力：
  - 低效率辐射发电（只能提供缓慢补能）
  - 小容量储能（只能支持短时活动）
- 保持内核最小可信边界不变：内核仍只保证位置/资源/审计，供能语义由模块决定。

## 范围

### In Scope
- 新增 2 个内置模块（builtin module）与对应模块清单：
  - `m1.power.radiation_harvest`
  - `m1.power.storage`
- 提供 `World` 级安装入口：一键注册工件、注册模块、激活模块（走治理事件链路）。
- 模块行为约束：
  - 发电为低效率、低速率；
  - 储能容量有限，连续移动将耗尽并被规则拒绝。
- 补充单元测试覆盖：模块安装、发电事件、连续移动受限。

### Out of Scope
- 真实 WASM 字节码执行（本阶段仍为 builtin sandbox）。
- 与 simulator 侧 `PowerPlant/PowerStorage` 设施状态的双向同步。
- 完整电网、跨地点输电、市场定价与工业化建造流程。

## 接口 / 数据

### 模块清单（Manifest）
- `m1.power.radiation_harvest`
  - `kind=Reducer`, `role=Domain`
  - 订阅：
    - `post_event`: `domain.agent_registered`, `domain.agent_moved`
    - `pre_action`: `action.*`（按 tick 生成辐射采集事件）
- `m1.power.storage`
  - `kind=Reducer`, `role=Body`
  - 订阅：
    - `post_event`: `domain.agent_registered`, `domain.agent_moved`
    - `pre_action`: `action.move_agent`（移动前做储能校验）

### 行为语义（V1）
- 发电模块：
  - 基于 Agent 位置计算简化辐射强度，按 tick 产出小额发电量。
  - 输出 `module.emitted(kind="power.radiation_harvest")` 作为审计与观测事件。
- 储能模块：
  - 为每个 Agent 维护私有储能状态（capacity/level）。
  - 每 tick 先被动充能，再对移动动作按距离扣能；能量不足则 `RuleDenied`。
  - 设计目标是“不能长期连续移动”，但允许短程/间歇移动。

### World 入口
- 新增安装入口（命名暂定）：`World::install_m1_power_bootstrap_modules(actor)`
  - 自动注册模块工件
  - 自动生成模块变更集（register + activate）
  - 通过 propose → shadow → approve → apply 完成生效
  - 已安装版本应幂等跳过

## 里程碑

- **P1**：完成设计文档与项目管理文档。
- **P2**：完成内置发电/储能模块实现与导出。
- **P3**：完成 `World` 安装入口与治理链路接入。
- **P4**：完成测试与文档状态回写。

## 风险

- runtime 当前未维护地点级辐射场，辐射模型需采用位置近似，真实性有限。
- 模块状态与 simulator 电力设施并行存在，短期会出现“双轨语义”。
- 若后续切换真实 WASM 执行器，需保证当前 builtin 语义与 ABI 保持兼容。
