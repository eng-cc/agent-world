# Agent World Simulator：Rust 到 Wasm 编译套件（KWT）设计文档

## 目标
- 提供一套可复用的“Rust 源码 -> wasm 模块产物”构建流程，降低模块开发与交付门槛。
- 将构建结果标准化为 `wasm` 文件 + 元数据清单（hash、大小、来源、目标平台），便于后续装载治理与分发。
- 保持与当前运行时契约兼容：目标平台为 `wasm32-unknown-unknown`，产物可直接对接 `ModuleSandbox/WasmExecutor` 体系。

## 范围
- In Scope：
  - 新增独立工具套件（CLI + 脚本封装），支持指定 Rust crate 构建 wasm。
  - 自动解析 crate 元信息并定位构建产物，输出标准化打包目录与 metadata。
  - 支持 dry-run（仅校验与命令拼装，不执行构建）与基础错误诊断。
  - 提供最小示例模板，验证“从 Rust 代码到 wasm 文件”的闭环。
- Out of Scope：
  - 模块签名、证书信任链与发布审批流程。
  - 分布式 artifact 上传/拉取与多副本同步。
  - 自动生成业务级 `ModuleManifest` 版本策略（仅输出构建 metadata）。

## 接口 / 数据
- CLI（草案）：
  - `wasm_build_suite build --manifest-path <path> --module-id <id> --out-dir <dir> [--profile release|dev] [--dry-run]`
- 输出结构（草案）：
  - `<out-dir>/<module-id>.wasm`
  - `<out-dir>/<module-id>.metadata.json`
- metadata 字段（草案）：
  - `module_id`
  - `target`（固定 `wasm32-unknown-unknown`）
  - `profile`
  - `source_manifest_path`
  - `artifact_path`
  - `wasm_hash_sha256`
  - `wasm_size_bytes`

## 里程碑
- M1：完成 KWT-1（构建套件主功能：构建、定位、打包、metadata 输出）。
- M2：完成 KWT-2（单元测试与示例模板，形成可验证闭环）。
- M3：完成 KWT-3（回归验证、文档与 devlog 收口）。

## 风险
- 不同 crate 的 `lib` 命名与构建配置差异可能导致产物定位失败：通过 `cargo metadata` + 明确报错兜底。
- 环境未安装 `wasm32-unknown-unknown` target：工具提供清晰诊断并给出修复建议。
- 脚本与 CLI 行为漂移：以 CLI 为单一实现源，脚本仅做参数转发。
