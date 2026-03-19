# oasis7: 高频脚本参数契约与失败语义（2026-03-11）

- 对应设计文档: `doc/scripts/governance/script-parameter-contracts-2026-03-11.design.md`
- 对应项目管理文档: `doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: `TASK-SCRIPTS-002` 已经明确了脚本分层与主入口，但高频脚本仍缺统一的“参数契约 + 失败语义”文档。开发者与 CI 维护者只能靠 `--help` 或读源码猜输入输出边界，导致误用与回归成本上升。
- Proposed Solution: 为高频主入口脚本冻结最小参数契约，统一记录 `用途 / 核心参数 / 默认值 / 典型失败语义 / 推荐调用场景`，供本地开发、QA 和 CI 引用。
- Success Criteria:
  - SC-1: 至少覆盖 `ci-tests.sh`、`release-gate.sh`、`build-game-launcher-bundle.sh`、`run-viewer-web.sh`、`site-link-check.sh`。
  - SC-2: 每个脚本都给出最小调用形式、关键参数、默认值与失败语义。
  - SC-3: `doc/scripts/project.md` 能据此关闭 `TASK-SCRIPTS-003`。
  - SC-4: 契约内容与现有 `--help` 输出一致，不发明未实现参数。

## 2. User Experience & Functionality
- User Personas:
  - 开发者：需要最小可执行命令和参数默认值。
  - `qa_engineer`：需要知道脚本失败时该如何判定是参数误用还是环境问题。
  - CI 维护者：需要识别哪些参数可以跳步、哪些会改变门禁覆盖范围。
- User Scenarios & Frequency:
  - 本地验证：每次手工执行高频脚本时。
  - CI 调整：每次修改 workflow 或 release gate 时。
  - 故障排查：脚本返回非零时，用失败语义快速归因。
- User Stories:
  - PRD-SCRIPTS-CONTRACT-001: As a 开发者, I want minimal invocation examples, so that I can run canonical scripts without guessing.
  - PRD-SCRIPTS-CONTRACT-002: As a `qa_engineer`, I want explicit failure semantics, so that I can classify script failures consistently.
  - PRD-SCRIPTS-CONTRACT-003: As a CI maintainer, I want skip/dry-run options clearly documented, so that pipeline variants remain auditable.
- Critical User Flows:
  1. `选择主入口脚本 -> 阅读最小命令 -> 传入必要参数 -> 执行并观察返回码`
  2. `脚本失败 -> 对照失败语义 -> 判断是参数、环境还是门禁失败`
  3. `CI 调整 skip/dry-run 配置 -> 回查契约 -> 确认覆盖范围变化`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 最小命令示例 | 脚本名、最小调用、适用场景 | 直接复制执行 | `unknown -> runnable` | 先给最短可跑版本 | 全员可读 |
| 参数契约 | 参数名、默认值、必填性、影响范围 | 读文档后决定是否覆盖默认值 | `default -> overridden` | 必填 > 常用可选 > 诊断参数 | 维护者可更新 |
| 失败语义 | 返回码来源、典型失败签名、推荐动作 | 失败时对照排障 | `failed -> classified` | 参数错误优先于环境错误解释 | `qa_engineer` / CI 可引用 |
- Acceptance Criteria:
  - AC-1: 专题文档以表格列出至少 5 个高频脚本契约。
  - AC-2: 至少包含一种 `dry-run` 和一种 `skip-*` 语义说明。
  - AC-3: `run-viewer-web.sh` 被明确为 trunk serve 入口，失败多归于环境/依赖层。
  - AC-4: `release-gate.sh` 的 skip 选项被明确解释为“覆盖范围变化”，不等同正常放行。
- Non-Goals:
  - 不为所有低频脚本补齐契约。
  - 不在本轮新增脚本自检逻辑。
  - 不改动 shell 实现。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: Bash `--help` 输出与脚本文档。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 该专题承接 `TASK-SCRIPTS-002` 的入口层级，在“已经知道该用哪个脚本”之后，进一步回答“这个脚本怎么调、失败说明什么”。
- Integration Points:
  - `scripts/ci-tests.sh`
  - `scripts/release-gate.sh`
  - `scripts/build-game-launcher-bundle.sh`
  - `scripts/run-viewer-web.sh`
  - `scripts/site-link-check.sh`
- Edge Cases & Error Handling:
  - `--help` 输出来自底层工具（例如 `trunk serve`）：文档必须注明脚本只是入口封装，而非自定义 CLI。
  - `skip-*` 参数：允许改变覆盖范围，但必须在文档中注明不可替代完整门禁。
  - `dry-run`：只输出命令，不得被视为真实通过记录。
- Non-Functional Requirements:
  - NFR-SPC-1: 文档内容必须可直接映射到当前帮助输出或脚本行为。
  - NFR-SPC-2: 至少为每个脚本提供一个“典型失败来源”分类。
  - NFR-SPC-3: 契约表可被 grep 检索，不依赖截图。
- Security & Privacy: 仅记录公开参数与失败语义，不暴露敏感环境变量。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (`SPC-1`): 补齐高频主入口脚本契约表。
  - v1.1 (`SPC-2`): 增补更多脚本和失败签名。
  - v2.0 (`SPC-3`): 将关键脚本契约转为自动化 help 校验。
- Technical Risks:
  - 风险-1: 部分脚本实际行为由底层工具决定，文档容易误写成自定义语义。
  - 风险-2: skip 参数若被误读成“推荐配置”，会稀释门禁约束。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-SCRIPTS-CONTRACT-001 | `TASK-SCRIPTS-003` / `SPC-1` | `test_tier_required` | 检查最小命令与参数表存在 | 本地开发与 QA 调用一致性 |
| PRD-SCRIPTS-CONTRACT-002 | `TASK-SCRIPTS-003` / `SPC-1` | `test_tier_required` | 抽样检查失败语义字段 | 脚本失败分类一致性 |
| PRD-SCRIPTS-CONTRACT-003 | `TASK-SCRIPTS-003` / `SPC-1` | `test_tier_required` | 检查 `dry-run` / `skip-*` 说明 | CI / release gate 可审计性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| `DEC-SPC-001` | 先记录高频脚本最小契约 | 一次性覆盖所有脚本 | 先把 80% 高频路径稳住更有效。 |
| `DEC-SPC-002` | `--help` 与脚本行为为唯一事实源 | 在文档中补充未实现“理想参数” | 避免文档超前于实现。 |
| `DEC-SPC-003` | `skip-*` 明确标成覆盖范围缩减 | 将其视作常规推荐参数 | 避免门禁被错误弱化。 |
