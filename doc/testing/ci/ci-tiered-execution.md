# Agent World: CI 与提交钩子测试分级（设计文档）

## 目标
- 降低本地 `pre-commit` 与 PR 快速反馈耗时，减少“每次提交都跑全量”带来的开发阻塞。
- 保留主干回归覆盖能力：将耗时高、环境依赖重的测试迁移到 GitHub Actions 每日定时任务。
- 让测试入口保持统一，避免脚本分叉造成门禁漂移。

## 范围

### In Scope
- 为 `scripts/ci-tests.sh` 增加分级参数（`required` / `full`）。
- `scripts/pre-commit.sh` 改为默认调用 `required` 级别。
- GitHub Actions 调整为：
  - `push`/`pull_request`：执行 `required`。
  - `schedule`（每日）：执行 `full`。
- 补充文档说明本地与 CI 的分级策略与触发方式。

### Out of Scope
- 进一步按 crate 或变更文件做“按需测试选择”（例如 changed-files matrix）。
- 远程缓存、并行矩阵、self-hosted runner 等性能优化。
- 修改业务测试用例本身的断言逻辑。

## 接口 / 数据
- 统一入口脚本：`scripts/ci-tests.sh [required|full]`
  - `required`：格式化校验、内置工件一致性、`test_tier_required` smoke case。
  - `full`：`test_tier_full` case + `libp2p` 特性测试 + `wasmtime`（`--lib --bins`） + viewer 在线/离线联测。
- 本地提交入口：`scripts/pre-commit.sh`（内部执行 `./scripts/ci-tests.sh required`）。
- GitHub Actions 入口：`.github/workflows/rust.yml`
  - 快速门禁：`CI_VERBOSE=1 ./scripts/ci-tests.sh required`
  - 定时全量：`CI_VERBOSE=1 ./scripts/ci-tests.sh full`

## 里程碑
- **T1**：设计文档与项目管理文档落地。
- **T2**：脚本分级改造完成，并保持向后兼容默认行为。
- **T3**：CI workflow 分流完成（push/PR 快速门禁 + 每日全量）。
- **T4**：文档与任务日志回写，完成验证。

## 风险
- `required` 覆盖下降可能导致某些 feature 回归在每日任务前不可见。
- `full` 仅定时执行后，问题发现延迟增大；需要保留手动触发能力。
- 若团队成员直接调用旧命令（不带参数），需明确默认等级避免误解。
