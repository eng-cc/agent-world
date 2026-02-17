# Builtin Wasm Canonical 构建环境固化方案

## 目标
- 固化 builtin wasm 的 canonical 构建环境，使 hash 清单由单一环境产出并可重复生成。
- 消除“同源码在不同宿主生成不同 wasm 字节”的门禁抖动，稳定 `sync --check` 与 CI required gate。
- 保持产物运行目标不变：仍为 `wasm32-unknown-unknown`，可跨平台加载执行。

## 范围
- In Scope：
  - 设计并落地 canonical wasm builder（容器化优先），固定 Rust toolchain/target/构建入口。
  - 将 `scripts/sync-m1-builtin-wasm-artifacts.sh` 与 `scripts/sync-m4-builtin-wasm-artifacts.sh` 切换为调用 canonical builder。
  - 更新 CI workflow，使 hash 校验仅基于 canonical builder 产物。
  - 增加一致性验收：同一提交在不同宿主触发构建，最终 hash 清单一致。
- Out of Scope：
  - 修改 wasm 业务模块语义与 ABI。
  - 更换 hash 算法、清单格式或 runtime 加载协议。
  - 重构 world/runtime 的模块注册与 DistFS 协议。

## 接口 / 数据
- Canonical builder 入口（拟新增）：
  - `scripts/build-builtin-wasm-canonical.sh`
  - 输入：`--module-ids-path`、`--out-dir`、`--profile`（默认 `release`）
  - 输出：`<out-dir>/<module_id>.wasm` 与 metadata（保持现有格式）
- 构建环境固化要素：
  - Builder image：固定镜像 tag（示例：`ghcr.io/eng-cc/agent-world-wasm-builder:rust-1.92.0`）
  - 固定 toolchain：`1.92.0`
  - 固定 target：`wasm32-unknown-unknown`
  - 固定 workspace 挂载路径：`/workspace`
  - 固定 `CARGO_HOME`、`RUSTUP_HOME`、`SOURCE_DATE_EPOCH`（用于减少非确定性输入）
- 现有清单与校验保持不变：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh --check`
- 回退机制（仅应急）：
  - 当容器运行时不可用时，允许显式设置 `AGENT_WORLD_ALLOW_NON_CANONICAL_WASM=1` 才能走本机构建。
  - CI 禁止回退路径。

## 里程碑
- M1：方案文档/项目管理文档落地，明确接口、镜像策略、验收口径。
- M2：实现 canonical builder 脚本与镜像构建说明，支持本地 dry-run。
- M3：接入 `sync-m1/m4`，默认走 canonical builder，补充必要测试。
- M4：CI required gate 切换并验证通过（Ubuntu runner）。
- M5：增加跨宿主一致性回归（至少 1 组非 Linux 宿主复核），收口遗留开关。

## 风险
- 容器运行时依赖（Docker/Podman）在部分开发机不可用，可能影响本地更新清单体验。
- 镜像版本漂移会破坏可复现性；必须固定 tag，并建立升级流程。
- 若 canonical builder 与 runtime 真实加载链路脱节，可能出现“清单通过但运行异常”；需保留 runtime smoke check。
- 初次切换会导致清单 hash 整体变化，需要一次性同步并与团队对齐。

