# Fix Pre-commit（预提交失败修复脚本）

## 目标
- 提供一个一键修复入口，处理本地 `pre-commit` 常见失败（重点是 Rust 格式化不一致）。
- 将“修复 + 复检”流程标准化，减少重复手工命令。

## 范围
- **范围内**：
  - 新增 `scripts/fix-precommit.sh`。
  - 执行全仓 Rust 格式化并将变更重新加入暂存区。
  - 调用既有 `scripts/pre-commit.sh` 做完整复检。
- **范围外**：
  - 不修改 `pre-commit` hook 安装方式。
  - 不新增 lint/type-check 等其他检查项。

## 接口 / 数据
- 脚本路径：`scripts/fix-precommit.sh`
- 调用方式：`./scripts/fix-precommit.sh`
- 执行顺序：
  1. `env -u RUSTC_WRAPPER cargo fmt --all`
  2. `git add -u`（将已跟踪文件的格式化结果加入暂存区）
  3. `./scripts/pre-commit.sh`（内部使用 `rustfmt --edition 2021` 处理暂存 Rust 文件）

## 里程碑
- **M1**：输出设计文档与项目管理文档。
- **M2**：实现修复脚本并完成可执行校验。
- **M3**：更新任务日志并回填状态。

## 风险
- **执行耗时**：会触发完整 `pre-commit` 测试链路，耗时取决于机器性能。
- **暂存区变化**：`git add -u` 会更新已跟踪文件的暂存状态，提交前需再次确认 diff。
