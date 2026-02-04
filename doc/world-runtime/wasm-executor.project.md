# Agent World Runtime：WASM 执行器接入（项目管理文档）

## 任务拆解
- [x] E1 定义执行器配置结构（WasmExecutorConfig）
- [x] E1 实现 `ModuleSandbox` 的执行器骨架（占位实现）
- [ ] E1 选择 WASM 引擎并落地基础依赖
- [ ] E2 接入燃料/超时/内存限制与错误码映射
- [ ] E2 补充输出校验失败路径测试（超限/超时）
- [ ] E3 编译缓存与并发安全策略
- [ ] E4 集成测试：真实 wasm 调用、确定性回放
- [ ] 文档更新：运行时集成分册补充执行器细节

## 依赖
- `ModuleSandbox` 接口与模块 ABI 文档（`doc/world-runtime/wasm-interface.md`）
- 模块加载缓存与存储实现（`doc/world-runtime/module-storage.md`）

## 状态
- 当前阶段：E1（配置结构与骨架已完成，引擎选择待定）
