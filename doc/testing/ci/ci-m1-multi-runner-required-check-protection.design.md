# oasis7: m1 多 Runner CI Required Check 保护设计（历史文件名归档提示）

- 对应需求文档: `doc/testing/ci/ci-m1-multi-runner-required-check-protection.prd.md`
- 对应项目管理文档: `doc/testing/ci/ci-m1-multi-runner-required-check-protection.project.md`
- 当前活跃需求文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.prd.md`
- 当前活跃项目管理文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.project.md`

> 状态更新（2026-03-17）:
> - 该旧设计文件名仅保留为追溯提示。
> - 当前活跃设计文档见 `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.design.md`。

## 1. 设计定位
定义 CI 与测试门禁专题设计，统一流水线分层、门禁策略、产物校验与失败保护。

## 2. 设计结构
- 流水线分层：按 required/full、runner、target 或专题阶段划分执行链路。
- 门禁策略层：定义通过条件、阻断条件与 required check 保护。
- 校验执行层：收敛构建、测试、hash/determinism 等自动校验入口。
- 回归治理层：沉淀失败签名、发布影响与后续演进。

## 3. 关键接口 / 入口
- CI workflow / check 入口
- 门禁/required check 配置
- runner/target/产物校验点
- CI 回归与失败签名

## 4. 约束与边界
- 门禁变更必须可审计、可回放。
- 基础门禁与增量专题门禁需边界清晰。
- 不在本专题重构整个平台 CI 基础设施。

## 5. 设计演进计划
- 先冻结门禁与执行分层。
- 再补专题校验与保护策略。
- 最后固化失败签名与回归。
