# P2P Builtin Wasm 身份共识与跨平台构建方案

## 目标
- 解决“不同宿主构建产物 hash 漂移导致门禁不稳定”的生产问题，同时保留节点本地可构建能力。
- 将一致性目标从“跨平台产物字节 hash 必须一致”升级为“跨平台模块身份（identity）一致”。
- 在不依赖 Docker/Podman 的前提下，支持多节点构建、验证与产物复用。

## 范围

### In Scope
- 为 builtin wasm 增加 identity manifest（`source_hash`、`build_manifest_hash`、`identity_hash`）。
- 扩展 `sync-m1/m4/m5` 流程：在同步 hash 清单时同时生成/校验 identity manifest。
- 运行时读取 identity manifest，为 bootstrap module manifest 写入可验证的 `artifact_identity`。
- 校验策略升级：`sync --check` 同时验证当前平台产物 hash 与 identity 数据完整性。
- 统一 required/pre-commit 门禁到 `m1/m4/m5` 三套 builtin 模块，避免只校验 `m1` 造成“部分模块漂移漏检”。
- runtime builtin materializer 支持非 canonical 平台节点的本地回退编译与缓存复用，不再把“hash 必须命中清单”作为唯一可用条件。
- `testing-manual.md` 与 `doc/scripts/pre-commit*.md` 同步到 identity 共识口径。
- 归档过时 hash-only 设计文档，避免后续实现继续参考旧目标。

### Out of Scope
- 修改 wasm ABI 协议版本与 runtime 模块执行语义。
- 引入中心化构建服务或强制容器化构建路径。
- 替换现有 `sha256` 算法。

## 接口 / 数据
- 现有 hash 清单继续保留：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.sha256`
- 新增 identity manifest：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.identity.json`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.identity.json`
  - `crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.identity.json`
- runtime 本地缓存索引（新增）：
  - ` .distfs/builtin_wasm/module_hash_index.txt`
  - 语义：记录 `module_id -> 最近一次可验证加载成功的 wasm_hash`，用于非 canonical 平台复用本地构建产物，避免每次启动重复编译。
- identity manifest 每个模块至少包含：
  - `module_id`
  - `source_hash`：模块源码输入集摘要（基于模块 crate 目录 + Cargo.lock）
  - `build_manifest_hash`：构建配方摘要（toolchain/target/build-std/canonicalizer）
  - `identity_hash`：`sha256("<module_id>:<source_hash>:<build_manifest_hash>")`
- 运行时接口：
  - `runtime/m{1,4,5}_builtin_wasm_artifact.rs` 增加读取 identity manifest 的辅助函数。
  - `runtime/world/bootstrap_{power,economy,gameplay}.rs` 改为使用 identity manifest 生成 `ModuleArtifactIdentity`。
  - `runtime/builtin_wasm_materializer.rs` 在“远程拉取失败 + 本地编译回退”路径允许落地非清单 hash（仅本地编译路径），并写入模块 hash 索引。
- 门禁接口：
  - `scripts/ci-tests.sh required` 统一执行：
    - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
    - `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
    - `./scripts/sync-m5-builtin-wasm-artifacts.sh --check`

## 里程碑
- M1：设计文档、项目管理文档落地。
- M2：`sync-m1` 主脚本扩展 identity manifest 生成与校验，并对 `m4/m5` 复用。
- M3：runtime 接入 identity manifest，替换 bootstrap 占位 identity 逻辑。
- M4：测试补齐与 required 回归通过。
- M5：项目文档与 devlog 收口。
- M6：全系统迁移（CI/pre-commit/runtime fallback）与过时文档归档完成。

## 风险
- `source_hash` 输入集合定义不稳定会导致无意义抖动，需要固定“仅模块 crate + Cargo.lock”。
- 旧版本清单迁移窗口内可能出现“有 hash 无 identity”状态，需提供兼容提示与一次性同步脚本。
- 若开发者在非标准环境覆盖构建参数，`build_manifest_hash` 会变化并触发门禁，需要清晰错误提示。
- 非 canonical 平台允许本地回退编译后，必须保证远程拉取路径仍然严格校验 hash 清单，避免把“跨平台兼容”退化成“任意字节可执行”。
