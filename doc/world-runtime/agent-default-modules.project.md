# Agent World Runtime：Agent 默认模块体系（项目管理文档）

## 任务拆解

### ADM-S1 方案冻结
- [x] 输出设计文档（`doc/world-runtime/agent-default-modules.md`）
- [x] 冻结默认模块包 V1（body/power/storage/sensor/mobility/memory/cargo）
- [x] 明确身体接口槽位扩容规则（消耗接口模块）

### ADM-S2 模型与动作接入
- [x] 在 runtime 状态中增加 `AgentBodyState`（槽位/扩容等级）
- [x] 增加 `expand_body_interface` 动作与事件（成功/拒绝）
- [x] 将“接口模块实体”接入 cargo 存储与消耗校验

### ADM-S3 默认模块实现
- [x] 落地 `m1.sensor.basic`（基础感知模块）
- [x] 落地 `m1.mobility.basic`（移动语义模块）
- [x] 落地 `m1.memory.core`（记忆模块最小实现）
- [x] 落地 `m1.storage.cargo`（实体存储模块）

### ADM-S4 安装与场景
- [x] 提供 `World::install_m1_agent_default_modules(actor)` 安装入口
- [x] 在场景初始化中支持“是否安装默认模块包”开关
- [x] 保证重复安装幂等（已激活版本跳过）

### ADM-S5 测试与收口
- [x] 单元测试：槽位扩容、模块安装/卸载、实体存储增删
- [x] 集成测试：低电降级顺序、默认模块协同、回放一致性
- [x] 文档回写：`doc/world-runtime.md` / 本项目管理文档 / 当日 devlog

## 依赖
- 现有模块治理链路（`propose -> shadow -> approve -> apply`）
- wasm-only 执行链路（`WasmExecutor`）与模块工件安装入口
- `agent_world_builtin_wasm` 模块常量导出与模块清单结构
- Agent 资源账本与动作路由（pre_action/post_event）

## 状态
- 当前阶段：ADM-S5 完成（默认模块体系 V1 收口）
- 下一阶段：接口模块来源机制分册定稿（制造/交易/回收/奖励）
- 最近更新：完成 BMS-51 文档口径清理，统一到 wasm-only 现状（2026-02-13）
