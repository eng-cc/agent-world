# ROUND-002 重复识别与合并执行清单

## 目标
- 在 ROUND-002 中将“重复内容识别 -> 合并决策 -> 索引回写 -> redirect 落地”形成可执行闭环。
- 严格区分三类决策：`merge`（合并）、`master-slave`（主从化）、`keep`（保留不合并）。

## 执行阶段
| 阶段 | 动作 | 产物 | 状态 |
| --- | --- | --- | --- |
| P0 | 建立启动文档与审读清单 | `consistency-review-round-002.md`、`round-002-reviewed-files.md`、本清单 | done |
| P1 | 分区盘点重复簇（A~E）并形成判定草案 | 子代理审计快照 + 候选簇判定（C2-*） | done |
| P2 | 批次执行合并/主从化并回写索引 | 模块 `prd.index.md` + 文档历史状态/替代链 | done |
| P3 | 回写 `审计轮次: 2` 与复审结论 | `S_round002` + ROUND-002 复审结果 | done |

## P1 盘点结果（2026-03-05）
| 分区 | 盘点结果 |
| --- | --- |
| A (`viewer`) | 8 个簇：3 个 `merge`、4 个 `master-slave`、1 个 `keep` |
| B (`p2p`) | 6 个簇：已全部按 `master-slave` 收口（phase/base 主入口） |
| C (`testing/ci + precommit`) | 6 个簇：规则归属固定到 CI 主文档，precommit 侧去定义化 |
| D/E (`site + readme/gap + gameplay`) | 9 个簇：以 `master-slave` 为主，模板段建议抽离 |

## P2 已执行批次
| 批次ID | 范围 | 决策 | 已落地产物 | 状态 |
| --- | --- | --- | --- | --- |
| B1-C2-007 | `viewer-chat-agent-prompt-default-values-inline-input` / `prefill`（PRD + project） | `merge`（prefill 为主，inline-input 降级历史） | 4 份专题文档历史状态回写 + `doc/world-simulator/prd.index.md` 替代链回写 | done |
| B2-C2-004 | `doc/testing/ci/*` + `doc/scripts/precommit/*` | `master-slave`（CI 文档链定义规则，precommit 文档保留执行入口） | CI 三份主 PRD 增加“口径归属”；precommit/fix-precommit 改为引用主口径并回写审计轮次 | done |
| B3-C2-009-S1 | `observer-sync-source-mode` / `observer-sync-source-dht-mode`（PRD + project） | `master-slave`（`source-mode` 为主，`source-dht-mode` 为 DHT 增量） | 4 份 observer 文档与 `doc/p2p/prd.index.md` 回写主从口径并更新审计轮次 | done |
| B3-C2-009-S2 | `observer-sync-mode-runtime-metrics` / `metrics-runtime-bridge` / `observability`（PRD + project） | `master-slave`（`runtime-metrics` 为主，另外两篇为增量） | 6 份 observer 文档回写主从口径并更新审计轮次 | done |
| B3-C2-010 | `node-contribution-points*`（PRD + project） | `master-slave`（`node-contribution-points` 为主） | 6 份 node 文档回写主从口径并更新审计轮次 | done |
| B3-C2-011 | `distfs-self-healing-*`（PRD + project） | `master-slave`（`control-plane` 为主） | 6 份 distfs 文档回写主从口径并更新审计轮次 | done |
| B3-C2-003 | `node-redeemable-power-asset*`（PRD + project） | `master-slave`（`node-redeemable-power-asset` 为主） | 6 份 node 文档回写主从口径并更新审计轮次 | done |
| B3-C2-008-S1 | `distfs-production-hardening-phase1~5`（PRD + project） | `master-slave`（`phase1` 为主） | 10 份 distfs 文档回写主从口径并更新审计轮次 | done |
| B3-C2-008-S2 | `distfs-production-hardening-phase6~9`（PRD + project） | `master-slave`（`phase1` 为主） | 8 份 distfs 文档回写主从口径并更新审计轮次 | done |
| B4-C2-005-S1 | `site/manual/*`（PRD + project） | `master-slave`（`site-manual-static-docs` 为主） | 4 份 manual 文档 + `doc/site/prd.index.md` 回写主从口径并更新审计轮次 | done |
| B4-C2-005-S2 | `site/github-pages/*`（PRD + project） | `master-slave`（`github-pages-game-engine-reposition-2026-02-25` 为主） | 34 份 github-pages 文档回写主从口径并更新审计轮次 | done |
| B5-C2-006-S1 | `doc/readme/gap/*`（PRD + project） | `master-slave`（`readme-gap-distributed-prod-hardening-gap12345` 为主） | 18 份 gap 文档 + `doc/readme/prd.index.md` 回写主从口径并更新审计轮次 | done |
| B5-C2-006-S2 | `doc/game/gameplay/*`（PRD + project） | `master-slave`（`gameplay-top-level-design` 为主） | 18 份 gameplay 文档 + `doc/game/prd.index.md` 回写主从口径并更新审计轮次 | done |
| B6-C2-001 | `viewer-gameplay-release-experience-overhaul` / `immersion-phase8~10`（PRD + project） | `keep` + `master-slave`（`experience-overhaul` 为主） | 8 份 viewer 文档回写主从口径并更新审计轮次 | done |
| B6-C2-002 | `viewer-live-full-event-driven-phase8~10`（PRD + project） | `master-slave`（`phase10-2026-02-27` 为主） | 6 份 viewer 文档 + `doc/world-simulator/prd.index.md` 回写主从口径并更新审计轮次 | done |
| B7-C2-001/C2-002 | `viewer-gameplay-release-immersion-phase8~10` + `viewer-live-full-event-driven-phase8/9`（PRD + project） | `merge`（物理合并，主入口保留） | 主文档合并增量内容 + 阶段文档历史状态回写 + `doc/world-simulator/prd.index.md` 历史入口更新 | done |

## 待执行批次（优先级）
| 批次 | 范围 | 目标 | 验收命令 |
| --- | --- | --- | --- |
| - | - | ROUND-002 批次执行已完成 | - |

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
