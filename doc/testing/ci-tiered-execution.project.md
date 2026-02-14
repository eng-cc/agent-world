# Agent World: CI 与提交钩子测试分级（项目管理文档）

## 任务拆解
- [x] T1 输出设计文档（`doc/testing/ci-tiered-execution.md`）
- [x] T1 输出项目管理文档（本文件）
- [ ] T2 改造 `scripts/ci-tests.sh` 支持 `required/full`
- [ ] T2 调整 `scripts/pre-commit.sh` 默认跑 `required`
- [ ] T3 调整 `.github/workflows/rust.yml`：push/PR 跑 `required`，每日定时跑 `full`
- [ ] T4 文档回写（`doc/scripts/pre-commit.md`、`doc/testing/ci-test-coverage.md`）
- [ ] T4 更新任务日志
- [ ] T4 运行验证并提交

## 依赖
- 统一测试入口：`scripts/ci-tests.sh`
- 本地提交入口：`scripts/pre-commit.sh`
- GitHub Actions workflow：`.github/workflows/rust.yml`

## 状态
- 当前阶段：T1 完成，T2 进行中
