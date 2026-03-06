# ROUND-004 文档设计质量审查执行清单

审计轮次: 4

## 目标
- 形成“设计审读 -> 问题分级 -> 整改落地 -> 复审收口”的可执行闭环。
- 确保文档体系在架构、分工、追溯、可执行性与可达性上长期可维护。

## 执行阶段
| 阶段 | 动作 | 产物 | 状态 |
| --- | --- | --- | --- |
| P0 | 建立启动文档与审读清单 | `consistency-review-round-004.md`、`round-004-reviewed-files.md`、本清单 | done |
| P1 | 按 D4-001~D4-008 分区审读并登记问题 | 设计问题清单（I4-*） | done |
| P2 | 输出整改方案与验收命令 | A4-* 整改项 + 验收命令 | todo |
| P3 | 执行整改并回写索引/引用 | 修订文档 + 索引/引用更新 | todo |
| P4 | 回写 `审计轮次: 4` 与复审结论 | `S_round004` + ROUND-004 复审结果 | todo |

## 并行批次（6 子代理）
| 批次 | Agent | 审读范围 | 状态 |
| --- | --- | --- | --- |
| B4-001 | Locke | `doc/core/**` + `doc/engineering/**` | done |
| B4-002 | Aristotle | `doc/world-simulator/**` | done |
| B4-003 | Beauvoir | `doc/p2p/**` | done |
| B4-004 | Socrates | `doc/testing/**` + `doc/scripts/**` + `doc/playability_test_result/**` | done |
| B4-005 | Feynman | `doc/site/**` + `doc/readme/**` + `doc/game/**` | done |
| B4-006 | Turing | `doc/world-runtime/**` + `doc/headless-runtime/**` + 根入口补查 | done |

## 即时回写规则（并行强制）
- 每个子代理读完 1 篇文档后，立即执行：
  - 回写该文档 `审计轮次: 4`。
  - 追加 1 条到 `doc/core/reviews/round-004-audit-progress-log.md`（pass/issue_open/blocked）。
- 主代理每轮汇总前先校验日志与文档标记一致性，避免“日志有记录但文档未标记”。
- 中断恢复时以 `round-004-audit-progress-log.md` 为唯一进度基线继续。

## 审读策略（建议）
- 先审根入口与治理文档（core/engineering），再审模块文档簇。
- high 严重度问题优先闭环：入口冲突、分工串写、追溯断链、验收命令失效、站点断链。
- 统一使用最小验收命令，避免“文档结论正确但无法复现”。

## 通用验收命令
- `test -f doc/core/reviews/consistency-review-round-004.md`
- `test -f doc/core/reviews/round-004-reviewed-files.md`
- `test -f doc/core/reviews/round-004-doc-design-quality-worklist.md`
- `test -f doc/core/reviews/round-004-audit-progress-log.md`
- `rg -n "D4-|A4-" doc/core/reviews/consistency-review-round-004.md`
- `./scripts/doc-governance-check.sh`
