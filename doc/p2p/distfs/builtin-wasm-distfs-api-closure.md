# World Runtime：Builtin Wasm DistFS API 闭环

## 目标
- 将 builtin wasm 的“写入/读取”统一走 `agent_world_distfs` API，避免旁路文件操作。
- 让后续 wasm 模块扩展复用同一 DistFS API 路径（脚本落盘、运行时读取、hash 校验语义一致）。
- 保持当前治理边界：git 继续追踪 `module_id -> sha256` 清单，wasm 二进制仍不入 git。

## 范围
- In Scope：
  - 为 `agent_world_distfs::LocalCasStore` 增加可选 hash 算法策略（默认 `blake3`，新增 `sha256`）。
  - 新增 DistFS 工具入口，用于按 hash manifest + built wasm 目录写入 blob（走 `LocalCasStore::put`）。
  - `scripts/sync-m4-builtin-wasm-artifacts.sh` 改为调用 DistFS 工具完成写入。
  - runtime builtin wasm 读取改为使用 `LocalCasStore` API（读取并按 sha256 校验）。
- Out of Scope：
  - 远程分发与拉取。
  - 模块签名体系。
  - wasm hash 主算法迁移（仍为 sha256）。

## 接口 / 数据
- DistFS API：
  - `LocalCasStore::new_with_hash_algorithm(root, HashAlgorithm)`
  - `HashAlgorithm::{Blake3,Sha256}`
  - `BlobStore::put/get/has` 在 `LocalCasStore` 上按实例 hash 策略工作。
- 工具入口：
  - `cargo run -p agent_world_distfs --bin hydrate_builtin_wasm -- --root <distfs-root> --manifest <sha256-manifest> --built-dir <wasm-dir>`
- 运行时：
  - `m1/m4` builtin wasm 读取改为 `LocalCasStore` API。
  - 缺失/校验失败仍返回 `ModuleChangeInvalid`，并包含定位信息。

## 里程碑
- M1：设计文档与项目管理文档。
- M2：`agent_world_distfs` hash 策略扩展 + hydrate 工具。
- M3：sync 脚本与 runtime 读取切到 DistFS API。
- M4：required 回归 + 文档/devlog 收口。

## 风险
- 脚本每次触发 `cargo run` 的耗时会增加；先保证正确性，后续可做工具常驻/缓存优化。
- hash 策略扩展需保持默认 `blake3` 行为不变，避免影响现有 distributed 路径。
