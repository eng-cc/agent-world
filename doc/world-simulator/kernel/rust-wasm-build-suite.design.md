# Rust 到 Wasm 编译套件设计

- 对应需求文档: `doc/world-simulator/kernel/rust-wasm-build-suite.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/rust-wasm-build-suite.project.md`

## 1. 设计定位
定义从 Rust crate 构建标准化 wasm 产物的工具链闭环，输出可直接对接 `ModuleSandbox/WasmExecutor` 的 artifact 与 metadata。

## 2. 设计结构
- CLI 主入口层：以 `wasm_build_suite build` 作为单一实现源，脚本仅做参数转发。
- 构建解析层：通过 manifest 与 crate metadata 定位构建目标、profile 与 wasm 产物。
- 打包输出层：标准化产出 `.wasm` 与 `.metadata.json`，附带 hash、大小、来源路径等信息。
- 诊断层：支持 dry-run 与 target 缺失、产物定位失败等基础错误提示。

## 3. 关键接口 / 入口
- `wasm_build_suite build --manifest-path ... --module-id ... --out-dir ...`
- `<out-dir>/<module-id>.wasm`
- `<out-dir>/<module-id>.metadata.json`
- `cargo metadata`
- `wasm32-unknown-unknown`

## 4. 约束与边界
- CLI 是唯一权威实现，脚本不能演化出独立行为。
- 目标平台固定为 `wasm32-unknown-unknown`，不在本阶段扩展多目标发布。
- 本专题不处理模块签名、审批与分发网络。
- metadata 必须足够支撑后续装载治理与 artifact 可追溯。

## 5. 设计演进计划
- 先固定 CLI 参数与标准输出目录。
- 再补 metadata、dry-run 与错误诊断。
- 最后通过最小模板与回归测试验证“源码到 wasm”闭环。
