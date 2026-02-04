# Agent World Runtime：WASM 执行器接入（设计分册）

本分册描述将真实 WASM 执行器接入 `ModuleSandbox` 的最小方案。

## 目标
- 在现有 `ModuleSandbox` 抽象之上提供真实 WASM 执行实现（可选 Wasmtime/Wasmer）。
- 与既有 ABI/序列化约定对齐，保证输入/输出可验证、可回放。
- 提供确定性与资源限制（内存、燃料、超时、输出大小）并可审计。

## 范围

### In Scope（V1）
- 以 `ModuleSandbox` 为适配层的执行器实现（不改动 world 内核调用流程）。
- 基本资源限制：内存上限、燃料/指令预算、超时、输出大小。
- 最小编译缓存：按 `wasm_hash` 缓存已编译模块。
- 可配置的执行器参数（燃料、超时、并发上限、缓存容量）。

### Out of Scope（V1 不做）
- 多线程并行执行与跨模块共享状态。
- 复杂 I/O host functions（保持纯函数模型）。
- JIT 运行时热更新或远程分发。

## 接口 / 数据

### 关键接口
- `ModuleSandbox`：保持现有 `call(request) -> ModuleOutput` 入口不变。
- `WasmExecutorConfig`（新增）：执行器配置（燃料、超时、内存上限、缓存上限）。
- `WasmExecutor`（新增实现）：封装底层引擎并实现 `ModuleSandbox`。

### 执行流程（概念）
1. 校验 `ModuleCallRequest`（limits 与运行时最大值）。
2. 按 `wasm_hash` 获取/编译模块（命中缓存或新编译）。
3. 绑定 host functions（仅暴露 ABI 必需的接口）。
4. 调用模块入口（`reduce` 或 `compute`），传入序列化输入。
5. 读取并反序列化输出，执行 `ModuleOutput` 校验。
6. 超时/超限返回 `ModuleCallFailure`，写入 `ModuleCallFailed` 事件。

### 资源限制与确定性
- **燃料/指令预算**：优先使用引擎原生 fuel/epoch 机制。
- **内存限制**：WASM memory pages + 运行时限制双重校验。
- **超时**：引擎 epoch 或外部 watchdog 触发超时。
- **确定性**：禁用非确定性 host function（时间、随机、I/O）。

## 里程碑
- **E1**：选择 WASM 引擎并完成配置结构体与沙箱实现骨架。
- **E2**：接入燃料/超时/内存限制，输出校验与错误码映射。
- **E3**：实现编译缓存与并发安全策略。
- **E4**：补充集成测试（真实 wasm、超限失败、确定性回放）。

## 风险
- 引擎版本升级导致行为变化（需锁定版本/回放验证）。
- 资源限制不一致（引擎与内核限制口径差异）。
- ABI 变更导致兼容性破坏（需版本化接口）。
