# Agent World: Required Tier 接入 M1 Builtin Wasm Hash 校验（设计文档）

## 目标
- 将 `m1` builtin wasm hash manifest 校验纳入 `required` 级别门禁，避免 PR 合入后才暴露清单漂移问题。
- 保持现有分级测试结构不变，仅在 required 的“前置检查”阶段增加一条确定性校验。
- 与现有 nightly + build-std + canonicalize 构建链路保持一致，不改动 runtime 协议与 hash 算法。

## 范围

### In Scope
- 修改 `scripts/ci-tests.sh`，在 `required` 路径执行：
  - 本地与 CI 均执行 `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- 保持 `full` 路径复用 required 前置检查（即同样覆盖 m1 校验）。
- 补充任务日志与项目管理文档回写。

### Out of Scope
- `m4` builtin wasm hash 校验是否并入 required（本次不变更）。
- 调整 GitHub Actions job 拓扑（本次仅改统一入口脚本）。
- 调整 builtin wasm 的构建参数（toolchain、build-std、canonicalize）与 hash 清单格式。

## 接口 / 数据
- 统一入口：`scripts/ci-tests.sh [required|full]`
- 新增 required 前置检查命令：
  - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- 依赖文件：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`

## 里程碑
- M1：设计文档与项目管理文档创建。
- M2：`scripts/ci-tests.sh` 接入 m1 校验（本地与 CI 强制执行）。
- M3：同步 m1 hash 清单并完成 required 回归，更新 devlog 与项目状态。

## 风险
- required 门禁耗时上升（新增 wasm 构建与 hash 校验）。
- 若开发机缺少 nightly/build-std 依赖，首次执行 required 会失败；需要按脚本提示补齐 rustup 组件。
- 若本地构建结果与已提交清单不一致，本地提交会被阻断；需要先执行 `scripts/sync-m1-builtin-wasm-artifacts.sh` 更新清单。
- 仅覆盖 m1，不覆盖 m4，仍存在 m4 清单漂移晚发现风险。
