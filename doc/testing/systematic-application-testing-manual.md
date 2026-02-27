# Agent World：系统性应用测试手册工程化收口（设计文档）

## 目标
- 将仓库内分层测试模型、触发矩阵、证据标准固化为统一执行口径。
- 让 Human/AI 在同一入口下执行“可复盘、可审计、可门禁”的整应用测试。

## 范围

### In Scope
- 维护并发布根目录手册 `testing-manual.md`。
- 维护测试分层（L0~L5）与套件目录（S0~S8）映射。
- 对齐 CI 已覆盖项与手工闭环补齐项（尤其 S6 Web 闭环）。

### Out of Scope
- 不在本设计内直接修改业务逻辑。
- 不在本设计内引入新测试框架。

## 接口/数据
- 主手册：`testing-manual.md`
- Web 闭环分册：`doc/testing/web-ui-playwright-closure-manual.md`
- 主脚本入口：`scripts/ci-tests.sh`
- 相关门禁脚本：`scripts/viewer-release-qa-loop.sh`、`scripts/viewer-release-full-coverage.sh`

## 里程碑
- M1：完成测试手册迁移与命名统一。
- M2：完成分层模型与套件矩阵收口。
- M3：完成 Web 闭环分册拆分与引用对齐。
- M4：持续维护增量规则并对齐 CI 变更。

## 风险
- 手册与脚本若不同步，会造成“文档正确但执行失败”的运维风险。
- 测试入口分散会导致团队执行口径漂移，需要持续收敛。
