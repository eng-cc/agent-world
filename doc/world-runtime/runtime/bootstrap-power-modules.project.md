# oasis7 Runtime：生命出厂电力模块（项目管理文档）

- 对应设计文档: `doc/world-runtime/runtime/bootstrap-power-modules.design.md`
- 对应需求文档: `doc/world-runtime/runtime/bootstrap-power-modules.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] T-MIG-20260303 (PRD-ENGINEERING-006): 逐篇阅读旧文档并完成人工重写迁移到 `.prd` 命名。
### B1 文档与方案
- [x] 输出设计文档（`doc/world-runtime/runtime/bootstrap-power-modules.prd.md`）
- [x] 输出项目管理文档（本文件）

### B2 模块实现
- [x] 新增低效率辐射发电模块（`m1.power.radiation_harvest`，wasm 工件语义）
- [x] 新增基础储能模块（`m1.power.storage`，含连续移动约束）
- [x] 导出模块 ID/默认参数常量（`agent_world_builtin_wasm`），供治理安装与测试复用
- [x] 完成默认电力模块安装入口与测试闭环

### B3 运行时接入
- [x] 新增 `World` 一键安装入口（register + activate + governance apply）
- [x] 保证重复安装幂等（已激活版本跳过）
- [x] 覆盖“已注册但停用”场景：重装时仅激活，不重复 register

### B4 验证与回写
- [x] 新增单元测试（安装生效、幂等、辐射发电事件、连续移动受限）
- [x] 新增回归测试（已注册但停用后可重新激活）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::power_bootstrap -- --nocapture`
- [x] 回写本项目管理文档状态与当日 devlog

## 依赖
- `doc/world-runtime/runtime/bootstrap-power-modules.design.md`
- runtime 模块治理链路（`propose -> shadow -> approve -> apply`）
- `ModuleSandbox` 抽象与 wasm-only 执行链路（`WasmExecutor`）
- `WorldEventBody::ModuleEmitted/ModuleStateUpdated` 事件语义

## 状态
- 当前阶段：B4 完成
- 最近更新：完成 BMS-51 文档口径清理，统一到 wasm-only 现状（2026-02-13）
