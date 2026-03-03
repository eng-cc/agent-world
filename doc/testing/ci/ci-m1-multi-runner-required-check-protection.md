# Agent World: m1 多 Runner CI Required Check 保护（设计文档）

## 目标
- 将 `Builtin Wasm m1 Multi Runner / verify-m1-multi-runner-summary` 固化为 `main` 分支的 required status check。
- 提供可复用、可审计、可重复执行的自动化脚本，避免手工点选配置漂移。
- 在不破坏现有保护策略的前提下，增量并集 required checks。

## 范围

### In Scope
- 新增 GitHub 分支保护自动化脚本（基于 `gh api`）。
- 支持“分支已保护”和“分支未保护”两种场景：
  - 已保护：仅 patch `required_status_checks`，保留并集。
  - 未保护：创建最小保护策略并注入 required checks。
- 默认目标仓库为当前 `origin`，默认分支 `main`，默认 check 为：
  - `Builtin Wasm m1 Multi Runner / verify-m1-multi-runner-summary`
- 输出明确执行结果与最终生效 checks 列表。

### Out of Scope
- 不在本任务中扩展 `m4/m5` 多 runner required checks。
- 不在本任务中改造 GitHub 组织级 ruleset。
- 不修改现有 CI workflow 的执行逻辑。

## 接口 / 数据
- 新增脚本：`scripts/ci-ensure-required-checks.sh`
- 关键参数：
  - `--repo <owner/repo>`（可选，默认从 `origin` 推导）
  - `--branch <branch>`（默认 `main`）
  - `--check <context>`（可重复；默认注入 m1 多 runner verify job）
  - `--strict <true|false>`（创建保护时使用，默认 `true`）
  - `--dry-run`（仅打印目标配置）
- 依赖：`gh`、`jq`、可写权限 token（`repo` scope）。

## 里程碑
- M1：设计文档与项目管理文档落地。
- M2：脚本实现与本地语法校验。
- M3：在 `eng-cc/agent-world` 的 `main` 分支应用并验证。
- M4：测试手册与项目状态收口。

## 风险
- 仓库管理员权限不足会导致 API 调用 403。
- 分支无保护时创建保护策略需带完整字段，若 payload 不完整会触发 422。
- required check 名称与 GitHub 实际 context 不一致会导致保护误配置，需要在脚本中显式校验。
