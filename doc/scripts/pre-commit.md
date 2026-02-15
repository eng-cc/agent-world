# Pre-commit Checks（本地提交前测试脚本）

## 目标
- 在本地提交前执行必跑门禁（required tier），尽快反馈常见回归。
- 以单一脚本形式减少重复维护，降低遗漏风险。

## 范围
- **范围内**：执行本地提交前格式化（仅格式化已暂存 Rust 文件）与 `required` 级别测试（格式化校验、工件一致性、feature 标签驱动的 smoke case）。
- **范围外**：lint 或其它包的静态检查。
- **范围外**：`libp2p`/`wasmtime` 特性回归与 viewer 在线/离线联测（由 `full` 级别承担）。

## 接口 / 数据
- 脚本路径：`scripts/pre-commit.sh`
- 运行命令：`./scripts/pre-commit.sh`
- 执行内容：
  - 先格式化已暂存的 Rust 文件：`env -u RUSTC_WRAPPER rustfmt --edition 2021 <staged .rs files>`，并自动 `git add` 回暂存区。
  - 再执行 builtin wasm 校验与 DistFS 落盘：
    - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
    - `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
  - 最后调用统一测试清单脚本：`CI_SKIP_BUILTIN_WASM_CHECKS=1 ./scripts/ci-tests.sh required`（避免重复执行同一校验）。
- 用例分级标签：
  - `test_tier_required`：本地提交与 PR 必跑 case。
  - `test_tier_full`：每日定时全量回归 case。
- 统一脚本支持两级：
  - `required`：本地提交与 PR 必跑。
  - `full`：每日定时与手动触发全量回归。
- CI 格式化校验仍由 `scripts/ci-tests.sh` 统一执行 `env -u RUSTC_WRAPPER cargo fmt --all -- --check`。

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
- 当 `pre-commit` 因格式化差异失败时，可执行：`./scripts/fix-precommit.sh`。
- 该脚本会自动执行：
  - `env -u RUSTC_WRAPPER cargo fmt --all`
  - `git add -u`
  - `./scripts/pre-commit.sh`

## 里程碑
- **M1**：新增本地提交前联测脚本并纳入文档说明。
- **M2**：提交前加入自动格式化时机，并在 CI 增加格式化检查。
- **M3**：补充“新仓库需重新注册 hook”文档与操作步骤。

## 风险
- **覆盖时延**：重型回归从提交路径迁移到定时任务后，问题发现时间可能延后至每日任务窗口。
- **环境差异**：本地与 CI 依赖不同可能造成结果不一致。
