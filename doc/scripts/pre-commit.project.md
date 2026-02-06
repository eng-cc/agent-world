# Pre-commit Checks（本地提交前测试脚本）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/scripts/pre-commit.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增本地提交前联测脚本（`scripts/pre-commit.sh`）
- [x] 安装 git pre-commit hook（调用 `scripts/pre-commit.sh`）
- [x] 更新任务日志
- [x] 运行测试 `./scripts/pre-commit.sh`
- [x] 提交到 git

## 依赖
- `cargo test`（agent_world viewer 联测）

## 状态
- 当前阶段：已提交
- 最近更新：安装 pre-commit hook（2026-02-06）
