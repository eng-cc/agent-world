# Agent World: Builtin Wasm 构建确定性护栏（项目管理文档）

## 任务拆解
- [x] T1 设计文档：`doc/testing/wasm-build-determinism-guard.md`
- [x] T1 项目管理文档：`doc/testing/wasm-build-determinism-guard.project.md`
- [x] T2 入口脚本护栏：`scripts/build-wasm-module.sh` 强制 canonical 构建输入 + 环境变量拦截
- [x] T3 构建工具护栏：`tools/wasm_build_suite/src/lib.rs` 增加 `--locked` 与 workspace build.rs/proc-macro 拦截
- [x] T4 测试与回归：补单元测试并执行构建相关验证
- [x] T5 devlog 回写并提交

## 依赖
- wasm 构建入口：`scripts/build-wasm-module.sh`
- wasm 构建实现：`tools/wasm_build_suite/src/lib.rs`
- required 门禁入口：`scripts/ci-tests.sh`

## 状态
- 当前阶段：已完成
