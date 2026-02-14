# M4 社会经济系统：工业链路与 WASM 模块化（项目管理文档）

## 任务拆解

### E1 设计收口
- [x] 输出设计文档：`doc/m4-industrial-economy-wasm.md`
- [x] 明确资源分层、工厂渐进建造、Recipe/Product/Factory 三类模块接口

### E2 ABI 接口基础落地
- [x] 在 `agent_world_wasm_abi` 增加经济模块接口数据结构
- [x] 增加 Recipe/Product/Factory 的 trait 接口草案
- [x] 增加序列化与最小行为单元测试

### E3 验证与文档回写
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world_wasm_abi`
- [x] 回写项目管理文档状态
- [x] 记录当日 devlog（任务完成内容 + 遗留事项）

### E4 runtime 最小执行闭环
- [x] 在 runtime 动作层新增 `BuildFactory` / `ScheduleRecipe`
- [x] 在 runtime 事件层新增建造/排产开始与完成事件（支持回放）
- [x] 在 `WorldState` 新增材料库存、工厂状态、建造队列、配方队列
- [x] 在 step 流程新增“到期任务结算”（工厂完工、配方完工）
- [x] 新增 runtime 经济闭环测试（建造时序、排产时序、产线容量、库存与电力扣减）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`

## 依赖

- `crates/agent_world_wasm_abi`：模块 ABI 与共享契约定义。
- `doc/world-runtime/wasm-interface.md`：底层 wasm-1 接口约束。
- `doc/world-runtime/module-lifecycle.md`：治理流程与生命周期约束。

## 状态

- 当前阶段：E4 完成（runtime 最小执行闭环已落地）。
- 下一步：接入基于 wasm 模块的 recipe/factory 在线评估与治理装载模板。
- 最近更新：完成 runtime 建造/排产闭环实现与测试（2026-02-14）。
