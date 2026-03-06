# Pre-commit Checks（本地提交前测试脚本）

审计轮次: 3

- 对应项目管理文档: doc/scripts/precommit/pre-commit.prd.project.md

## 目标
- 在本地提交前执行必跑门禁（required tier），尽快反馈常见回归。
- 以单一脚本形式减少重复维护，降低遗漏风险。

## 范围
- **范围内**：执行本地提交前格式化（仅格式化已暂存 Rust 文件）、`required` 级别测试（文档治理 + 格式校验 + required 测试套件）以及 viewer 的 `wasm32` 编译检查。
- **范围外**：lint 或其它包的静态检查。
- **范围外**：`libp2p`/`wasmtime` 特性回归与 viewer 在线/离线联测（由 `full` 级别承担）。

## 接口 / 数据
- 脚本路径：`scripts/pre-commit.sh`
- 运行命令：`./scripts/pre-commit.sh`
- 执行内容：
  - 先格式化已暂存的 Rust 文件：`env -u RUSTC_WRAPPER rustfmt --edition 2021 <staged .rs files>`，并自动 `git add` 回暂存区。
  - 调用统一测试清单脚本：`./scripts/ci-tests.sh required`（required 具体命令以该脚本与 `doc/testing/ci/ci-test-coverage.prd.md` 为准）。
  - 执行 viewer wasm 编译门禁：`env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`。
- 规则归属：
  - required/full 分层定义：`doc/testing/ci/ci-tiered-execution.prd.md`
  - case 标签定义（`test_tier_required`/`test_tier_full`）：`doc/testing/ci/ci-testcase-tiering.prd.md`
  - required/full 命令矩阵：`doc/testing/ci/ci-test-coverage.prd.md` 与 `scripts/ci-tests.sh`

## 最小验收命令
- `./scripts/pre-commit.sh`
- `./scripts/ci-tests.sh required`
- `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`

## Git Hook
- **注意**：Git hooks 不会随仓库内容一并版本化；克隆到新仓库（或重新初始化 `.git`）后，默认不会自动带上 `pre-commit` hook，需要手动重新注册。
- 在仓库根目录重新注册：
```
cat > .git/hooks/pre-commit <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

./scripts/pre-commit.sh
HOOK

chmod +x .git/hooks/pre-commit
```
- 可用以下命令确认是否已注册：
```
test -x .git/hooks/pre-commit && echo "pre-commit hook installed"
```

## 失败修复
- 当 `pre-commit` 失败时，统一走 `./scripts/fix-precommit.sh`；修复流程与边界以 `doc/scripts/precommit/precommit-remediation-playbook.prd.md` 为准。

## 里程碑
- **M1**：新增本地提交前联测脚本并纳入文档说明。
- **M2**：提交前加入自动格式化时机，并在 CI 增加格式化检查。
- **M3**：补充“新仓库需重新注册 hook”文档与操作步骤。

## 风险
- **覆盖时延**：重型回归从提交路径迁移到定时任务后，问题发现时间可能延后至每日任务窗口。
- **环境差异**：本地与 CI 依赖不同可能造成结果不一致。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
