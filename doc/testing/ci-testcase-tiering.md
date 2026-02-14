# Agent World: CI 测试分级细化到 Test Case（设计文档）

## 目标
- 将提交门禁从“整套 `cargo test`”进一步细化到“关键 test case 级别”，缩短本地与 PR 反馈时间。
- 保持分层策略：`required` 用最小闭环 smoke case，`full` 继续覆盖重型特性与联测。
- 让 required 用例清单可读、可维护、可增减，避免脚本里散乱堆叠命令。

## 范围

### In Scope
- 为 test case 增加 feature 标签（`test_tier_required` / `test_tier_full`）。
- `scripts/ci-tests.sh` 的 `required` 级别改为执行 `--tests`，由 feature 标签自动过滤 smoke case。
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
  - `agent_world` 的 `test_tier_required` 用例（模块路由、模块生命周期、状态、仓库、订阅过滤、offline viewer、world init）
- `full` 固定包含：
  - `agent_world` 的 `test_tier_full` 用例（在 `required` 之上追加更重 case）
  - 扩展回归：`wasmtime`（`--lib --bins`）、`libp2p`、viewer live 联测。
- 标签约定：
  - `#[cfg(feature = "test_tier_required")]`：必跑门禁 case。
  - `#[cfg(feature = "test_tier_full")]`：全量回归 case。

## 里程碑
- **T1**：设计文档与项目管理文档。
- **T2**：脚本改造为 test case 级 required 清单。
- **T3**：文档回写、验证、任务日志与提交。

## 风险
- smoke case 清单过窄会引入覆盖盲区；需要后续依据回归历史持续增补。
- feature 标签与脚本目标若不同步，会造成“case 已标记但未被门禁执行”；需变更时同步回写脚本与文档。
