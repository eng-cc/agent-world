# Agent World: CI 安装 wasm32-unknown-unknown target（设计文档）

## 目标
- 修复 CI 在执行 builtin wasm 工件校验时因缺少 `wasm32-unknown-unknown` target 导致的构建失败。
- 保持 `scripts/ci-tests.sh required` 与 `scripts/ci-tests.sh full` 在 GitHub Actions 中稳定可复现。
- 将 target 依赖显式化，避免 runner 环境变化引发的隐式回归。

## 范围

### In Scope
- 更新 `.github/workflows/rust.yml`：在 `required-gate` 与 `full-regression` 两个 job 中显式安装 `wasm32-unknown-unknown` target。
- 保持现有 CI 分级策略（required/full）与测试命令不变。
- 补充本次任务日志与项目状态回写。

### Out of Scope
- CI 缓存、矩阵并行、runner 类型切换。
- builtin wasm 构建脚本本身逻辑调整。
- 其他 target（如 `wasm32-wasip1`）的新增与治理。

## 接口 / 数据
- CI workflow 文件：`.github/workflows/rust.yml`
- 新增执行步骤命令：
  - `rustup target add wasm32-unknown-unknown`
- 受影响的现有门禁路径：
  - `CI_VERBOSE=1 ./scripts/ci-tests.sh required`
  - `CI_VERBOSE=1 ./scripts/ci-tests.sh full`
  - 其中 `required` 会调用：
    - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
    - `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`

## 里程碑
- **WASMCI-1**：完成设计文档与项目管理文档。
- **WASMCI-2**：完成 workflow 修改，两个 job 均安装 wasm target。
- **WASMCI-3**：完成最小回归验证、任务日志与提交收口。

## 风险
- GitHub Actions 网络波动可能导致 `rustup target add` 偶发失败。
- 若后续脚本引入更多 target，仍可能出现同类问题；需继续保持“显式安装依赖”的策略。
