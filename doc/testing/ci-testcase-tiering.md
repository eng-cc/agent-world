# Agent World: CI 测试分级细化到 Test Case（设计文档）

## 目标
- 将提交门禁从“整套 `cargo test`”进一步细化到“关键 test case 级别”，缩短本地与 PR 反馈时间。
- 保持分层策略：`required` 用最小闭环 smoke case，`full` 继续覆盖重型特性与联测。
- 让 required 用例清单可读、可维护、可增减，避免脚本里散乱堆叠命令。

## 范围

### In Scope
- `scripts/ci-tests.sh` 的 `required` 级别改为执行显式 test case 列表（带 `--exact`）。
- 保持 `full` 级别的扩展回归（`libp2p`、`wasmtime`、viewer 联测）不变。
- 文档补充“命令级分层 + case 级筛选”的执行策略。

### Out of Scope
- 基于 changed-files 的动态选测。
- 自动生成/自动学习 smoke case 清单。
- 修改业务测试代码断言。

## 接口 / 数据
- 统一入口：`scripts/ci-tests.sh [required|full]`
- `required` 固定包含：
  - 静态门禁（fmt、工件一致性）
  - 关键 smoke case（运行时、模块仓库、网络、执行器、viewer）
- `full`：`required` + 特性/联测全量回归。

## 里程碑
- **T1**：设计文档与项目管理文档。
- **T2**：脚本改造为 test case 级 required 清单。
- **T3**：文档回写、验证、任务日志与提交。

## 风险
- smoke case 清单过窄会引入覆盖盲区；需要后续依据回归历史持续增补。
- 用例名变更后 `--exact` 会直接失败；需在重构测试名称时同步更新脚本。
