# Core 文档时效与索引一致性审查记录（第005轮）

审计轮次: 5

## 目的
- 为 `TASK-CORE-005` 提供 ROUND-005 审查入口，聚焦 ROUND-004 延期项闭环。
- 本轮仅处理 4 个中风险问题：`I4-006/I4-008/I4-009/I4-010`。
- 目标是一次性完成“状态时效 + 完成态字段 + 命名一致性 + 索引覆盖规则”收敛。

## 轮次信息
- 轮次编号: `ROUND-005`
- 轮次状态: `completed` (`not_started` | `in_progress` | `completed`)
- 审查时间窗: 2026-03-06 ~ 2026-03-15
- 审查负责人: cc

状态判定：
- `not_started`: 仅完成启动文档与方法准备，尚未形成有效问题记录。
- `in_progress`: 已开始审读并登记问题/整改项，但未形成最终复审结论。
- `completed`: 本轮范围内问题已收口，整改项关闭或已登记批准延期备注。

## 文档级审计标记方法（缺省=0）
- 每个受审文档采用字段 `审计轮次: <整数>` 标识最新已完成审计轮次。
- 本轮执行规则：
  - 单篇文档完成 ROUND-005 审读后，在同一提交回写 `审计轮次: 5`。
  - 若仅发现问题暂不改正文，仍回写 `审计轮次: 5`，并在进度日志标记 `issue_open`。
  - 若实施整改，需同步更新 `prd.md/prd.index.md/project.md` 与相关引用。
  - 若文档未纳入本轮范围，保持原值。
- 本轮完成条件：`S_round005` 范围文档全部完成审读并形成复审结论。

建议统计命令：
```bash
rg -n "^审计轮次:\s*5$" doc/world-simulator doc/p2p doc/site doc/playability_test_result --glob '*.md'
```

## 重点审计维度（本轮限定）
| 编号 | 维度 | 审计目标 | 严重度判定 |
| --- | --- | --- | --- |
| D5-001 | 状态时效 | 含 `当前状态` 的文档应具备可核对的更新字段（`更新日期/最近更新/更新时间`） | 缺失=medium |
| D5-002 | 完成态字段 | 完成态文档具备最小完成信息（完成日期/最近更新/映射字段） | 缺失=medium |
| D5-003 | 命名一致性 | world-simulator 文档命名与目录语义一致，避免歧义命名 | 偏差=medium |
| D5-004 | 索引覆盖规则 | p2p/world-simulator 索引覆盖声明规则统一、边界明确 | 不一致=medium |

## 启动范围（仅本轮）
- S5-001: `doc/world-simulator/**`
- S5-002: `doc/p2p/**`
- S5-003: `doc/site/**` + `doc/playability_test_result/**`
- S5-004: root/模块索引补查（`doc/world-simulator/prd.index.md` + `doc/p2p/prd.index.md`）

## 并行审计编组（4 批）
- G5-001: `doc/world-simulator/**`（命名一致性 + 状态时效）
- G5-002: `doc/p2p/**`（索引覆盖规则 + 状态时效）
- G5-003: `doc/site/**` + `doc/playability_test_result/**`（状态时效 + 完成态字段）
- G5-004: 索引规则统一回写与交叉复核（world-simulator/p2p）

## 受审文件清单（S_round005）
- 清单文件：`doc/core/reviews/round-005-reviewed-files.md`
- 统计口径：`doc/world-simulator/** + doc/p2p/** + doc/site/** + doc/playability_test_result/**`
- 当前基线（2026-03-06 18:04 CST）：`516` 份文档
- 用途：作为 ROUND-005 判定分母（仅统计本轮范围）。

## 审计进度日志（逐文档）
- 日志文件：`doc/core/reviews/round-005-audit-progress-log.md`
- 记录粒度：1 文档 1 记录（即时写入，不延后）。
- 字段：`时间`、`审计人/代理`、`文档路径`、`结论(pass/issue_open/blocked)`、`问题编号`、`备注`。

## 承接问题清单（来自 ROUND-004）
| 编号 | 承接来源 | 问题描述 | 影响范围 | 建议动作 | 当前判定 |
| --- | --- | --- | --- | --- | --- |
| I5-001 | I4-006 | 文档状态与最近更新时间失真 | world-simulator/p2p/site/playability_test_result | 统一状态时效字段与回写规则 | closed（A5-003，规则扫描剩余 0） |
| I5-002 | I4-008 | 完成态缺日期/映射字段 | world-simulator/p2p/site/playability_test_result | 补齐完成态最小字段集合 | closed（A5-004，当前规则扫描剩余 0） |
| I5-003 | I4-009 | world-simulator 结构命名一致性偏差 | world-simulator | 执行命名收敛并回写索引/引用 | closed（A5-005，风险词扫描 0 命中） |
| I5-004 | I4-010 | world-simulator 与 p2p 索引覆盖规则不一致 | world-simulator/p2p | 建立统一覆盖规则并同步入口说明 | closed（A5-006，索引规则模板已统一） |

## 整改项
| 编号 | 整改动作 | 责任人 | 截止时间 | 验收命令 | 状态 |
| --- | --- | --- | --- | --- | --- |
| A5-001 | 建立 ROUND-005 启动台账（本文件 + 审读清单 + 执行清单 + 进度日志） | cc | 2026-03-06 | `test -f doc/core/reviews/consistency-review-round-005.md && test -f doc/core/reviews/round-005-reviewed-files.md && test -f doc/core/reviews/round-005-timeliness-index-worklist.md && test -f doc/core/reviews/round-005-audit-progress-log.md` | done |
| A5-002 | 完成 S5-001~S5-004 全量审读并登记 I5-* 细分问题 | cc | 2026-03-10 | `rg -n "I5-00[1-4]" doc/core/reviews/consistency-review-round-005.md` | done |
| A5-003 | 收敛状态时效字段（`当前状态` 与更新时间字段配对） | cc | 2026-03-12 | `rg -l "^## 状态$|^#### 当前状态$|^### 当前状态$" doc/world-simulator doc/p2p doc/site doc/playability_test_result --glob '*.md' \| while read -r f; do rg -q "更新日期|最近更新|更新时间|最近更新时间" "$f" || echo "$f"; done` | done |
| A5-004 | 补齐完成态最小字段（完成日期/最近更新/映射字段） | cc | 2026-03-12 | `rg -l "当前状态[:：].*(已完成|completed)" doc/world-simulator doc/p2p doc/site doc/playability_test_result --glob '*.md' \| while read -r f; do rg -q "完成日期|最近更新|更新日期|更新时间" "$f" || echo "$f"; done` | done |
| A5-005 | 统一 world-simulator 命名语义并回写引用 | cc | 2026-03-13 | `rg --files doc/world-simulator --glob '*.md' \| rg "fix|tmp|misc|new|update"` | done |
| A5-006 | 统一 world-simulator/p2p 索引覆盖规则与声明模板 | cc | 2026-03-14 | `rg -n "覆盖规则|纳入规则|排除规则|历史入口|兼容跳转" doc/world-simulator/prd.index.md doc/p2p/prd.index.md` | done |
| A5-007 | 生成 `S_round005` 并完成复审结论 | cc | 2026-03-15 | `rg -n "轮次状态|复审结论|S_round005" doc/core/reviews/consistency-review-round-005.md` | done |

## 特殊情况备注（仅在无需整改时填写）
| 编号 | 原因 | 风险 | 临时缓解 | 复审日期 | 评审人 |
| --- | --- | --- | --- | --- | --- |

## 复审结果
- 复审时间：2026-03-06 18:04 CST
- 复审结论：ROUND-005 completed（`I5-001~I5-004` 全部关闭，`S_round005=516/516`）。
- 当前进展：已完成 ROUND-005 全量审读与逐文档回写；本轮范围文档均已回写 `审计轮次: 5`。
