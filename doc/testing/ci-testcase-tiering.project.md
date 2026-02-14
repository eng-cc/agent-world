# Agent World: CI 测试分级细化到 Test Case（项目管理文档）

## 任务拆解
- [x] T1 输出设计文档（`doc/testing/ci-testcase-tiering.md`）
- [x] T1 输出项目管理文档（本文件）
- [x] T1.1 清理 `check-include-warning-baseline` 旧门禁脚本与调用
- [ ] T2 改造 `scripts/ci-tests.sh`：`required` 执行 test case 级 smoke 清单
- [ ] T2 保持 `full` 回归路径（特性/联测）
- [ ] T3 文档回写（`doc/testing/ci-test-coverage.md`、`doc/scripts/pre-commit.md`）
- [ ] T3 更新任务日志
- [ ] T3 运行验证并提交

## 依赖
- `scripts/ci-tests.sh`
- `.github/workflows/rust.yml`
- `scripts/pre-commit.sh`

## 状态
- 当前阶段：T1 完成，T2 进行中
