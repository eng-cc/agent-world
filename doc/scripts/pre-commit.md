# Pre-commit Checks（本地提交前测试脚本）

## 目标
- 在本地提交前复现 CI 测试清单，确保本地与 CI 一致。
- 以单一脚本形式减少重复维护，降低遗漏风险。

## 范围
- **范围内**：执行 CI 测试清单（包含全量测试、wasmtime 特性测试、viewer 联测）。
- **范围外**：格式化、lint 或其它包的检查。

## 接口 / 数据
- 脚本路径：`scripts/pre-commit.sh`
- 运行命令：`./scripts/pre-commit.sh`
- 执行内容：
  - 调用统一测试清单脚本 `scripts/ci-tests.sh`（与 CI 共用）。

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

## 里程碑
- **M1**：新增本地提交前联测脚本并纳入文档说明。

## 风险
- **执行耗时**：全量测试与集成测试耗时较长，需控制在本地可接受范围。
- **环境差异**：本地与 CI 依赖不同可能造成结果不一致。
