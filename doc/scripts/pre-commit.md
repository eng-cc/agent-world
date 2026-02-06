# Pre-commit Checks（本地提交前测试脚本）

## 目标
- 在本地提交前快速复现 CI 的 viewer 联测，提前发现协议连通性问题。
- 以最小脚本形式减少重复操作，降低遗漏风险。

## 范围
- **范围内**：执行 viewer 在线/离线联测的 cargo test 命令。
- **范围外**：全量测试、格式化、lint 或其它包的检查。

## 接口 / 数据
- 脚本路径：`scripts/pre-commit.sh`
- 运行命令：`./scripts/pre-commit.sh`
- 执行内容：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_offline_integration`

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
- **执行耗时**：集成测试可能耗时较长，需控制在本地可接受范围。
- **环境差异**：本地与 CI 依赖不同可能造成结果不一致。
