# Builtin Wasm Nightly build-std 可复现构建方案

## 目标
- 采用 nightly + `-Z build-std` 重建 wasm 目标 std，实现 builtin wasm 构建输入闭环可控。
- 在保留现有 hash/manifest 校验机制前提下，消除宿主预编译 std 差异导致的 hash 漂移。
- 延续现有路径归一化策略（`--remap-path-prefix`）与 wasm custom section canonicalize，确保 hash 仅反映可执行语义。

## 范围
- In Scope：
  - 固定 builtin wasm 构建工具链到 pinned nightly（含 `rust-src` 与 `wasm32-unknown-unknown`）。
  - 在 wasm 构建调用链启用 `-Z build-std`、`-Z build-std-features`。
  - 更新 CI required/full gate 的 wasm 构建环境变量与组件安装步骤。
  - 重新同步 `m1/m4` hash 清单并回归 `sync --check` 与 required tier。
- Out of Scope：
  - runtime ABI、DistFS 协议、hash 算法与 manifest 文件格式改动。
  - 引入容器化 canonical 构建路径（本方案明确不采用）。

## 接口 / 数据
- 构建入口：
  - `scripts/build-wasm-module.sh`
  - `scripts/build-builtin-wasm-modules.sh`
- 构建参数（新增/固化）：
  - `AGENT_WORLD_WASM_TOOLCHAIN`（默认 `nightly-2025-12-11`）
  - `AGENT_WORLD_WASM_BUILD_STD`（默认 `1`）
  - `AGENT_WORLD_WASM_BUILD_STD_COMPONENTS`（默认 `std,panic_abort`）
  - `AGENT_WORLD_WASM_BUILD_STD_FEATURES`（默认空，不追加 `-Z build-std-features`）
- wasm build suite：
  - `tools/wasm_build_suite/src/lib.rs` 在 cargo build 参数注入 `-Z build-std*`（受环境变量控制）。
- 清单与校验：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh --check`

## 里程碑
- M1：文档与任务拆解完成。
- M2：nightly + build-std 在 wasm 构建链路落地并通过本地构建。
- M3：CI required/full 固化 nightly build-std 环境。
- M4：m1/m4 清单同步，`sync --check` 与 required tier 回归通过。

## 风险
- `-Z build-std` 会显著增加首次构建成本（时间与网络下载）。
- nightly 版本升级会影响 hash，需要固定日期并建立升级流程。
- 若 nightly 源发生不可用/回滚，CI 会受影响；需允许显式切换到新 pinned nightly。
