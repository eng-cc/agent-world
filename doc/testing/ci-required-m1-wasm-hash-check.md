# Agent World: Required Tier 接入 M1 Builtin Wasm Hash 校验（设计文档）

## 目标
- 将 `m1` builtin wasm hash manifest 校验纳入 `required` 级别门禁，避免 PR 合入后才暴露清单漂移问题。
- 保持现有分级测试结构不变，仅在 required 的“前置检查”阶段增加一条确定性校验。
- 与现有 nightly + build-std + canonicalize 构建链路保持一致，不改动 runtime 协议与 hash 算法。
- 在 macOS/Linux 构建产物 hash 不一致时，允许同一 module 声明多组可接受 hash，保持本地与 CI 同时可校验。

## 范围

### In Scope
- 修改 `scripts/ci-tests.sh`，在 `required` 路径执行：
  - 本地与 CI 均执行 `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- 改造 m1 wasm hash 清单格式为 `module_id hash1 hash2 ...`，用于表达多平台 hash。
- 改造 hash 校验与 DistFS hydration：
  - `--check` 时 built hash 命中任一 hash 即视为通过；
  - hydration 以 built bytes 的实际 hash 入库，并校验该 hash 在 manifest 的允许列表内。
- 改造 runtime builtin wasm materializer：
  - 支持每个 module 传入候选 hash 列表；
  - DistFS 命中、fetch fallback、compile fallback 都允许匹配任一候选 hash。
- 保持 `full` 路径复用 required 前置检查（即同样覆盖 m1 校验）。
- 补充任务日志与项目管理文档回写。

### Out of Scope
- `m4` builtin wasm hash 校验是否并入 required（本次不变更）。
- 调整 GitHub Actions job 拓扑（本次仅改统一入口脚本）。
- 调整 builtin wasm 的构建参数（toolchain、build-std、canonicalize）与 hash 算法。

## 接口 / 数据
- 统一入口：`scripts/ci-tests.sh [required|full]`
- 新增 required 前置检查命令：
  - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- 依赖文件：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- manifest 行格式：
  - 旧：`module_id hash`
  - 新：`module_id hash1 hash2 ...`（至少一个 hash，按需累积平台 hash）

## 里程碑
- M1：设计文档与项目管理文档创建。
- M2：`scripts/ci-tests.sh` 接入 m1 校验（本地与 CI 强制执行）。
- M3：同步 m1 hash 清单并完成 required 回归，更新 devlog 与项目状态。
- M4：修复跨平台 hash 差异导致的 CI 失败，落地多 hash manifest + loader/hydrator 兼容。

## 风险
- required 门禁耗时上升（新增 wasm 构建与 hash 校验）。
- 若开发机缺少 nightly/build-std 依赖，首次执行 required 会失败；需要按脚本提示补齐 rustup 组件。
- 若本地构建结果与已提交清单不一致，本地提交会被阻断；需要先执行 `scripts/sync-m1-builtin-wasm-artifacts.sh` 更新清单。
- 仅覆盖 m1，不覆盖 m4，仍存在 m4 清单漂移晚发现风险。
- 多 hash 清单会随平台矩阵增长，需要定期清理不再支持的平台 hash。
