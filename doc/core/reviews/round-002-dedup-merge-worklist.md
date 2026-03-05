# ROUND-002 重复识别与合并执行清单

## 目标
- 在 ROUND-002 中将“重复内容识别 -> 合并决策 -> 索引回写 -> redirect 落地”形成可执行闭环。
- 严格区分三类决策：`merge`（合并）、`master-slave`（主从化）、`keep`（保留不合并）。

## 执行阶段
| 阶段 | 动作 | 产物 | 状态 |
| --- | --- | --- | --- |
| P0 | 建立启动文档与审读清单 | `consistency-review-round-002.md`、`round-002-reviewed-files.md`、本清单 | done |
| P1 | 分区盘点重复簇（A~E）并形成判定草案 | 子代理审计快照 + 候选簇判定（C2-*） | done |
| P2 | 批次执行合并/主从化并回写索引 | 模块 `prd.index.md` + 文档历史状态/替代链 | in_progress |
| P3 | 回写 `审计轮次: 2` 与复审结论 | `S_round002` + ROUND-002 复审结果 | in_progress |

## P1 盘点结果（2026-03-05）
| 分区 | 盘点结果 |
| --- | --- |
| A (`viewer`) | 8 个簇：1 个 `merge`、5 个 `master-slave`、2 个 `keep` |
| B (`p2p`) | 6 个簇：以 `master-slave` 为主（phase/base 主入口） |
| C (`testing/ci + precommit`) | 6 个簇：规则归属固定到 CI 主文档，precommit 侧去定义化 |
| D/E (`site + readme/gap + gameplay`) | 9 个簇：以 `master-slave` 为主，模板段建议抽离 |

## P2 已执行批次
| 批次ID | 范围 | 决策 | 已落地产物 | 状态 |
| --- | --- | --- | --- | --- |
| B1-C2-007 | `viewer-chat-agent-prompt-default-values-inline-input` / `prefill`（PRD + project） | `merge`（prefill 为主，inline-input 降级历史） | 4 份专题文档历史状态回写 + `doc/world-simulator/prd.index.md` 替代链回写 | done |

## 待执行批次（优先级）
| 批次 | 范围 | 目标 | 验收命令 |
| --- | --- | --- | --- |
| B2 | `doc/testing/ci/*` + `doc/scripts/precommit/*` | 固定规则归属并消除双处口径维护（C2-004） | `rg -n "required|full|ci-tests\.sh" doc/testing/ci doc/scripts/precommit -g '*.md'` |
| B3 | `doc/p2p/node/*` + `doc/p2p/distfs/*` + `doc/p2p/observer/*` | 收敛阶段型专题重复块（C2-003/C2-008/C2-009） | `find doc/p2p/node doc/p2p/distfs doc/p2p/observer -maxdepth 1 -type f -name '*.prd.md' | wc -l` |
| B4 | `doc/site/manual/*` + `doc/site/github-pages/*` | 收敛重复流程并保留单一入口（C2-005） | `find doc/site/manual doc/site/github-pages -maxdepth 1 -type f -name '*.prd.md' | wc -l` |
| B5 | `doc/readme/gap/*` + `doc/game/gameplay/*` | 统一治理/玩法总述口径主入口（C2-006） | `find doc/readme/gap doc/game/gameplay -maxdepth 1 -type f -name '*.md' | wc -l` |

## 决策记录模板（每簇必填）
| 字段 | 说明 |
| --- | --- |
| 候选簇ID | 如 `C2-001` |
| 判定 | `merge` / `master-slave` / `keep` |
| 主文档 | 保留为权威口径的文档路径 |
| 被合并/降级文档 | 受影响文档列表 |
| 替代链 | 新旧入口映射 |
| 索引回写 | 需修改的 `prd.index.md` 列表 |
| 风险 | 链接断裂、历史引用丢失、语义冲突 |
| 验收命令 | 最小可复现验证命令 |

## 通用验收命令
- `test -f doc/core/reviews/consistency-review-round-002.md`
- `test -f doc/core/reviews/round-002-reviewed-files.md`
- `test -f doc/core/reviews/round-002-dedup-merge-worklist.md`
- `rg -n "I2-|C2-|A2-" doc/core/reviews/consistency-review-round-002.md`
- `./scripts/doc-governance-check.sh`
