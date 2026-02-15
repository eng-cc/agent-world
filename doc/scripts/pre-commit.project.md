# Pre-commit Checks（本地提交前测试脚本）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/scripts/pre-commit.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增本地提交前联测脚本（`scripts/pre-commit.sh`）
- [x] 安装 git pre-commit hook（调用 `scripts/pre-commit.sh`）
- [x] 更新任务日志
- [x] 运行测试 `./scripts/pre-commit.sh`
- [x] 提交到 git
- [x] 对齐 CI 测试清单（改为调用 `scripts/ci-tests.sh`）
- [x] 提交前新增代码格式化时机（`cargo fmt --all`）
- [x] CI 增加格式化检查（`cargo fmt --all -- --check`）
- [x] 文档补充：新仓库需重新注册 pre-commit hook（2026-02-07）
- [x] 移除默认 pre-commit/CI 的 builtin wasm hash 校验（改为手动按需执行，2026-02-14）
- [x] pre-commit 重新启用 builtin wasm hash 校验并落盘 DistFS（2026-02-15）

## 依赖
- `rustfmt`（staged `.rs`）/ `cargo fmt -- --check`
- `cargo test`（agent_world viewer 联测）

## 状态
- 当前阶段：已提交
- 最近更新：pre-commit 重新启用 `sync-m1/m4-builtin-wasm-artifacts --check`，并通过 `CI_SKIP_BUILTIN_WASM_CHECKS=1` 避免重复执行（2026-02-15）
