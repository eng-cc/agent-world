# ROUND-005 状态时效与索引一致性执行清单

审计轮次: 5

## 目标
- 专项收敛 ROUND-004 延期项：`I4-006/I4-008/I4-009/I4-010`。
- 形成“字段口径统一 -> 索引规则统一 -> 引用回写 -> 复审收口”的闭环。

## 执行阶段
| 阶段 | 动作 | 产物 | 状态 |
| --- | --- | --- | --- |
| P0 | 建立启动文档与审读清单 | `consistency-review-round-005.md`、`round-005-reviewed-files.md`、`round-005-audit-progress-log.md`、本清单 | done |
| P1 | 按 S5-001~S5-004 审读并细分问题 | I5-* 细分问题与影响范围 | done |
| P2 | 输出整改方案与验收命令 | A5-* 整改项 + 验收命令 | done |
| P3 | 执行整改并回写索引/引用 | 修订文档 + 索引/引用更新 | done |
| P4 | 回写 `审计轮次: 5` 与复审结论 | `S_round005` + ROUND-005 复审结果 | done |

## 并行批次（4 批）
| 批次 | 范围 | 目标问题 | 状态 |
| --- | --- | --- | --- |
| B5-001 | `doc/world-simulator/**` | I5-001/I5-002/I5-003 | done |
| B5-002 | `doc/p2p/**` | I5-001/I5-002/I5-004 | done |
| B5-003 | `doc/site/**` + `doc/playability_test_result/**` | I5-001/I5-002 | done |
| B5-004 | `doc/world-simulator/prd.index.md` + `doc/p2p/prd.index.md` | I5-004（规则统一） | done |

## 执行原则
- 不扩范围：仅处理 `I5-001~I5-004`，其它问题记录到后续轮次。
- 先统一规则再批量回写，避免“改一篇坏一片”。
- 每读完一篇即追加进度日志，保持可中断恢复。

## 通用验收命令
- `test -f doc/core/reviews/consistency-review-round-005.md`
- `test -f doc/core/reviews/round-005-reviewed-files.md`
- `test -f doc/core/reviews/round-005-timeliness-index-worklist.md`
- `test -f doc/core/reviews/round-005-audit-progress-log.md`
- `rg -n "I5-|A5-" doc/core/reviews/consistency-review-round-005.md`
- `./scripts/doc-governance-check.sh`
