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

## 依赖

- `crates/agent_world_wasm_abi`：模块 ABI 与共享契约定义。
- `doc/world-runtime/wasm-interface.md`：底层 wasm-1 接口约束。
- `doc/world-runtime/module-lifecycle.md`：治理流程与生命周期约束。

## 状态

- 当前阶段：E3 完成。
- 下一步：进入 runtime 执行闭环实现（build_factory / schedule_recipe）。
- 最近更新：完成 ABI 接口落地、验证与 devlog 记录（2026-02-14）。
