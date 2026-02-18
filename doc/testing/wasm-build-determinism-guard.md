# Agent World: Builtin Wasm 构建确定性护栏（设计文档）

## 目标
- 在 builtin wasm 构建脚本入口强制关键输入一致（toolchain/target/build-std/env），降低“本地与 CI 构建参数不一致”带来的 hash 漂移。
- 在编译期增加 workspace 级拦截，阻断会引入主机/时间/环境耦合风险的 `build.rs` 与 `proc-macro` 目标进入 builtin wasm 模块构建链路。
- 保持现有 m1/m4 hash manifest 与 DistFS 加载机制不变，聚焦“构建输入治理”。

## 范围

### In Scope
- 修改 `scripts/build-wasm-module.sh`：
  - 默认启用确定性护栏（可显式关闭用于本地实验）；
  - 强制 canonical 构建输入（toolchain/target/build-std 相关 env）；
  - 拦截会污染构建结果的一组继承环境变量（如 `RUSTFLAGS`、`CARGO_ENCODED_RUSTFLAGS`、`CARGO_TARGET_DIR` 等）；
  - 固定复现友好环境（`CARGO_INCREMENTAL=0`、`SOURCE_DATE_EPOCH`、`TZ/LANG/LC_ALL`）。
- 修改 `tools/wasm_build_suite/src/lib.rs`：
  - `cargo metadata` 与 `cargo build` 增加 `--locked`；
  - 新增 workspace 编译期确定性校验：禁止 workspace 本地包存在 `custom-build(build.rs)` 或 `proc-macro` 目标。
- 为新增拦截逻辑补充单元测试。

### Out of Scope
- 重构/替换 wasm canonicalize 算法。
- 对 third-party 依赖（`source != None`）的 build.rs/proc-macro 做全面封禁。
- 调整 runtime builtin wasm loader 协议或 hash 清单格式。

## 接口 / 数据
- 入口脚本：`scripts/build-wasm-module.sh`
  - 新增开关：`AGENT_WORLD_WASM_DETERMINISTIC_GUARD`（默认 `1`）。
- 构建工具：`tools/wasm_build_suite`
  - 默认使用 `--locked` 保证 lockfile 一致。
  - 新增开关：`AGENT_WORLD_WASM_VALIDATE_WORKSPACE_COMPILETIME`（默认启用）。
- 失败信息：在脚本和构建工具层给出“哪个输入不符合约束/哪个 package 被拦截”的可定位报错。

## 里程碑
- M1：设计文档与项目管理文档创建。
- M2：入口脚本确定性护栏落地。
- M3：`wasm_build_suite` 增加 `--locked` 与 workspace 编译期拦截。
- M4：补测试并完成回归验证（最小 + required）。

## 风险
- 入口护栏更严格后，历史依赖自定义 `RUSTFLAGS` 的本地流程会被阻断，需要显式关闭护栏用于实验。
- workspace 级 `build.rs/proc-macro` 拦截可能影响未来模块设计自由度，需要在设计评审时提前约束。
- `--locked` 会在 lockfile 未同步时直接失败，提升了稳定性同时也提高了日常维护门槛。
