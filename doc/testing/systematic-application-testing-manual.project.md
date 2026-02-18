# Agent World: 系统性应用测试手册（项目管理文档）

## 任务拆解
- [x] T1 盘点当前仓库实现与测试分布（`agent_world` / `viewer` / `node` / `net` / `consensus` / `distfs`）
- [x] T2 盘点现有 `required/full` 与 CI 实际覆盖范围，明确缺口
- [x] T3 重写设计文档（`testing-manual.md`）
- [x] T4 建立分层模型（L0~L5）与套件目录（S0~S8）
- [x] T5 建立“改动路径 -> 必跑套件”针对性矩阵
- [x] T6 固化 Human/AI 共用执行剧本、充分度标准、证据规范与失败分诊
- [x] T7 更新任务日志并完成本轮收口
- [x] T8 按最新要求将测试手册迁移到项目根目录并修正引用
- [x] T9 按最新命名将手册统一为 `testing-manual.md`，并在 `AGENTS.md` 增加测试引用入口
- [x] T10 将 `AGENTS.md` 的 Agent Web 闭环细节迁移到 `testing-manual.md`，`AGENTS.md` 仅保留短入口与约束

## 依赖
- `AGENTS.md`（开发工作流、测试分层、Web 闭环优先口径）
- `scripts/ci-tests.sh`
- `scripts/pre-commit.sh`
- `.github/workflows/rust.yml`
- `scripts/run-viewer-web.sh`
- `scripts/viewer-owr4-stress.sh`
- `scripts/llm-longrun-stress.sh`
- `testing-manual.md`
- `doc/viewer-manual.md`
- `doc/world-simulator/scenario-files.md`
- `doc/testing/ci-tiered-execution.md`
- `doc/testing/ci-testcase-tiering.md`
- `doc/testing/ci-test-coverage.md`

## 状态
- 当前阶段：已完成（`AGENTS.md` 与 `testing-manual.md` 已完成“迁移 + 引用”分工）
- 最近更新：2026-02-19
