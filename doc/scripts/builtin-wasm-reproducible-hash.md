# Builtin Wasm Hash 可复现构建（跨环境一致）

## 目标
- 根治 builtin wasm hash 在本地与 CI 间不一致的问题，保证 `--check` 校验可稳定通过。
- 保留当前门禁口径：继续使用“现编 wasm 与 git 清单比对”作为一致性基线。
- 将修复限定在构建脚本层，不改变 runtime 装载协议与 hash 算法。

## 范围
- In Scope：
  - `scripts/build-wasm-module.sh` 增加可复现构建参数，消除机器绝对路径进入 wasm 产物。
  - `tools/wasm_build_suite` 在打包阶段对 wasm 做 canonicalize（移除 custom sections）后再计算 hash。
  - 重新同步 `m1/m4` builtin wasm hash 清单并验证 `--check`。
  - 回归 required tier，确认不会引入提交门禁回归。
- Out of Scope：
  - 调整 `SHA-256` 算法或 manifest 文件格式。
  - 修改 runtime 读取 DistFS blob 的行为。
  - 修改 CI workflow 触发条件与执行拓扑。

## 接口 / 数据
- 入口脚本：`scripts/build-wasm-module.sh`
- 产物规范化入口：`tools/wasm_build_suite/src/lib.rs`（`canonicalize_wasm_bytes`）
- 关键构建参数：`RUSTFLAGS` 附加以下 remap 规则：
  - `--remap-path-prefix=$HOME/.cargo=/cargo`
  - `--remap-path-prefix=$HOME/.rustup=/rustup`
  - `--remap-path-prefix=<rustc_sysroot>=/rustup/toolchain`
  - `--remap-path-prefix=<repo_root>=/workspace`
- 规范化策略：
  - 保留 wasm 核心语义 section（type/import/function/table/memory/global/export/start/element/code/data 等）。
  - 移除 custom sections（含 debug/name/producers/path 相关元数据）以消除跨主机漂移。
- 校验脚本：
  - `scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh --check`
- 清单数据：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`

## 里程碑
- M1：设计文档与项目管理文档落地。
- M2：构建脚本接入 remap-path-prefix。
- M3：同步 m1/m4 hash 清单并通过 `--check`。
- M4：required tier 回归通过，文档与 devlog 收口。
- M5：构建产物 canonicalize 落地并覆盖测试，消除 remap 后残余跨平台漂移。

## 风险
- 若开发者自定义 `RUSTFLAGS` 且与 remap 规则冲突，可能引入构建参数行为差异。
- remap 后 wasm hash 会整体变化，需同步更新清单；短窗口内可能出现一次性门禁抖动。
- 若后续新增构建入口未复用该脚本，可能再次引入跨环境 hash 漂移。
- canonicalize 会移除 custom sections，影响调试可读性（不影响执行语义）；若未来需要保留调试信息，需在非门禁路径单独保留原始产物。
