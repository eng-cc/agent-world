# Pre-commit Checks（本地提交前测试脚本）

## 目标
- 在本地提交前复现 CI 测试清单，确保本地与 CI 一致。
- 以单一脚本形式减少重复维护，降低遗漏风险。

## 范围
- **范围内**：执行本地提交前格式化（仅格式化已暂存 Rust 文件）与 CI 测试清单（含格式化校验、全量测试、wasmtime 特性测试、viewer 联测）。
- **范围外**：lint 或其它包的静态检查。

## 接口 / 数据
- 脚本路径：`scripts/pre-commit.sh`
- 运行命令：`./scripts/pre-commit.sh`
- 执行内容：
  - 先格式化已暂存的 Rust 文件：`env -u RUSTC_WRAPPER rustfmt --edition 2021 <staged .rs files>`，并自动 `git add` 回暂存区。
  - 再调用统一测试清单脚本 `scripts/ci-tests.sh`（与 CI 共用）。
- CI 格式化校验：`scripts/ci-tests.sh` 与 `.github/workflows/rust.yml` 会执行 `env -u RUSTC_WRAPPER cargo fmt --all -- --check`。

## Git Hook
- 已在本仓库安装 `pre-commit` hook：提交前会自动执行 `scripts/pre-commit.sh`。
- 如需重装，可在仓库根目录创建 `.git/hooks/pre-commit` 并调用脚本：
```
#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

./scripts/pre-commit.sh
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

## 风险
- **执行耗时**：全量测试与集成测试耗时较长，需控制在本地可接受范围。
- **环境差异**：本地与 CI 依赖不同可能造成结果不一致。
