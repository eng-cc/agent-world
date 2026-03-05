# Core 重复/合并审查记录（第002轮）

## 目的
- 为 `TASK-CORE-005` 提供 ROUND-002 审查入口，聚焦“文档重复内容识别与可合并评估”。
- 本轮主线-1：识别跨文档重复（目标、范围、规格矩阵、验收条款、操作步骤）。
- 本轮主线-2：对可合并文档给出“保留文档 + 合并策略 + 替代链 + 索引回写”并执行。

## 轮次信息
- 轮次编号: `ROUND-002`
- 轮次状态: `completed` (`not_started` | `in_progress` | `completed`)
- 审查时间窗: 2026-03-05 ~ 2026-03-05
- 审查负责人: cc

状态判定：
- `not_started`: 尚未形成有效“重复问题/合并候选/整改项”记录。
- `in_progress`: 已登记重复问题并启动分批审读，但未形成最终复审结论。
- `completed`: 合并/保留决策已收口，阻断项关闭或已登记批准延期备注。

## 文档级审计标记方法（缺省=0）
- 每个受审文档采用字段 `审计轮次: <整数>` 标识最新已完成审计轮次。
- 本轮执行规则：
  - 单篇文档完成 ROUND-002 审读后，在同一提交回写 `审计轮次: 2`（与是否需要合并整改解耦）。
  - 若判定“保留不合并”，仍应回写 `审计轮次: 2`，并在本记录登记判定理由。
  - 若尚未完成 ROUND-002 审读，则保持原值（缺失按 `0`）。
- 本轮完成条件：纳入 `S_round002` 的文档全部满足 `审计轮次 >= 2`，且复审结论已落档。

建议统计命令：
```bash
rg -n "^审计轮次:\s*2$" doc --glob '*.md'
```

## 启动范围（重复高风险分区）
- A: `world-simulator/viewer`（约 96 份 `.prd.md`，日期专题密集）
- B: `p2p/node + p2p/distfs + p2p/observer`（约 46 份 `.prd.md`）
- C: `testing/ci + scripts/precommit`（测试门禁专题口径复用高）
- D: `site/github-pages + site/manual`（站点发布专题易出现流程重复）
- E: `readme/gap + game/gameplay`（治理/玩法总述文档语义重叠）

## 受审文件清单（S_round002）
- 清单文件：`doc/core/reviews/round-002-reviewed-files.md`
- 生成规则：`rg -l "^审计轮次:\s*2$" doc --glob '*.md' | sort`
- 用途：作为 `A2-006` 的统计分母（仅对纳入本轮清单的文档判定“已审读/未审读”）。

## 子代理审计快照（2026-03-05）
| 分区 | 覆盖范围 | 重复簇数量 | 当前结论 |
| --- | --- | --- | --- |
| A | `doc/world-simulator/viewer` | 8 | 3 簇 `merge`（含 `inline-input/prefill` + phase8~10 物理合并），4 簇 `master-slave`，1 簇 `keep` |
| B | `doc/p2p/node + distfs + observer` | 6 | 6 簇均已按 `master-slave` 收口，主入口固定为 base/phase1 文档 |
| C | `doc/testing/ci + doc/scripts/precommit` | 6 | 规则归属固定到 CI 主文档链；pre-commit 侧去定义化、改引用 |
| D/E | `doc/site/* + doc/readme/gap + doc/game/gameplay` | 9 | 大部分适合 `master-slave`，结构模板段建议抽成统一模板引用 |

## 一致性问题（重复/合并维度）
| 编号 | 问题描述 | 影响范围 | 严重度 | 当前判定 |
| --- | --- | --- | --- | --- |
| I2-001 | Viewer 默认值专题 `inline-input` 与 `prefill` 在目标/范围/验收高度重复。 | `doc/world-simulator/viewer/viewer-chat-agent-prompt-default-values-*` | high | `merge`；已执行首批收口（C2-007） |
| I2-002 | Viewer/P2P 阶段型文档存在阶段间“目标/范围/验收模板”重复。 | `doc/world-simulator/viewer/*phase*`、`doc/p2p/node/*`、`doc/p2p/distfs/*`、`doc/p2p/observer/*` | high | 以 `master-slave` 为主，viewer phase8~10 已物理合并；P2P 侧已完成 `observer sync`、`node-contribution`、`distfs-self-healing`、`node-redeemable-power-asset`、`distfs-production-hardening` 收口 |
| I2-003 | CI 分层专题与 pre-commit 专题存在规则描述重复，易双处漂移。 | `doc/testing/ci/*`、`doc/scripts/precommit/*` | high | 已完成首批收口（C2-004）：规则主源固定在 `testing/ci`，`precommit` 仅保留执行入口 |
| I2-004 | Site 手册与 github-pages 专题存在流程叙事重复。 | `doc/site/manual/*`、`doc/site/github-pages/*` | medium | `master-slave`；保留主叙事文档，日期文档改差异记录 |
| I2-005 | README gap 与 gameplay 总述反复定义术语/模块分层。 | `doc/readme/gap/*`、`doc/game/gameplay/*` | medium | 已按 `master-slave` 收口：`readme-gap12345` 与 `gameplay-top-level-design` 固定为双主入口 |
| I2-006 | 跨目录复用模板段（映射说明、迁移记录）在业务文档重复粘贴。 | 多模块 `.prd.md/.prd.project.md` | medium | 建议模板化（`keep` 文档，收敛模板） |

## 合并候选批次
| 编号 | 候选文档簇 | 候选主文档 | 合并策略 | 状态 |
| --- | --- | --- | --- | --- |
| C2-001 | `viewer-gameplay-release-immersion-phase8~10` | `viewer-gameplay-release-experience-overhaul.prd.md` | `merge`（物理合并，阶段文档归档） | done |
| C2-002 | `viewer-live-full-event-driven-phase8~10` | `viewer-live-full-event-driven-phase10-2026-02-27.prd.md` | `merge`（物理合并，阶段文档归档） | done |
| C2-003 | `node-redeemable-power-asset*` 系列 | `node-redeemable-power-asset.prd.md` | `master-slave` | done |
| C2-004 | `testing/ci` 分层专题与 `pre-commit` 专题 | `ci-tiered-execution.prd.md` + `pre-commit.prd.md` | 固定规则归属，删除重复定义 | done |
| C2-005 | `site/manual` + `site/github-pages` 镜像/叙事专题 | `site-manual-static-docs.prd.md` + `github-pages-game-engine-reposition-2026-02-25.prd.md` | `master-slave` | done |
| C2-006 | `readme/gap` 与 `gameplay` 总述簇 | `readme-gap-distributed-prod-hardening-gap12345.prd.md` + `gameplay-top-level-design.prd.md` | `master-slave` + 模板化 | done |
| C2-007 | `viewer-chat-agent-prompt-default-values-inline-input` vs `prefill` | `viewer-chat-agent-prompt-default-values-prefill.prd.md` | `merge`（inline-input 降级历史） | done |
| C2-008 | `distfs-production-hardening-phase1~9` | `distfs-production-hardening-phase1.prd.md` | `master-slave` | done |
| C2-009 | `observer-sync-source*`/`observer-sync-mode*` | `observer-sync-source-mode.prd.md` + `observer-sync-mode-runtime-metrics.prd.md` | `master-slave` | done |
| C2-010 | `node-contribution-points*` 系列 | `node-contribution-points.prd.md` | `master-slave` | done |
| C2-011 | `distfs-self-healing-*` 系列 | `distfs-self-healing-control-plane-2026-02-23.prd.md` | `master-slave` | done |

## 整改项
| 编号 | 整改动作 | 责任人 | 截止时间 | 状态 |
| --- | --- | --- | --- | --- |
| A2-001 | 建立 ROUND-002 重复识别与合并执行台账（本文件 + 工作清单 + 审读清单） | cc | 2026-03-05 | done |
| A2-002 | 完成分区 A/B 的重复簇盘点并给出“可合并/保留”判定 | cc | 2026-03-07 | done |
| A2-003 | 完成分区 C 的规则归属收口（CI/pre-commit 去重） | cc | 2026-03-07 | done |
| A2-004 | 完成分区 D/E 的主入口收口与历史专题分组策略 | cc | 2026-03-08 | done |
| A2-005 | 执行首批合并/主从化迁移并回写替代链、索引、redirect | cc | 2026-03-09 | done |
| A2-006 | 对已完成 ROUND-002 审读的文档回写 `审计轮次: 2`，并以 `S_round002` 为统计分母 | cc | 2026-03-09 | done |
| A2-007 | 完成 C2-007（viewer 默认值专题）主从化落地并更新模块索引 | cc | 2026-03-05 | done |
| A2-008 | 并行执行 B3 第一批主从化（observer-sync-mode、node-contribution、distfs-self-healing） | cc | 2026-03-05 | done |
| A2-009 | 并行执行 B3 第二批主从化（node-redeemable-power-asset、distfs-production-hardening） | cc | 2026-03-05 | done |
| A2-010 | 并行执行 B4（site/manual + github-pages）主从化并回写模块索引 | cc | 2026-03-05 | done |
| A2-011 | 并行执行 B5（readme/gap + game/gameplay）主从化并回写模块索引 | cc | 2026-03-05 | done |
| A2-012 | 并行执行 B6（viewer phase8~10）主从化并回写模块索引 | cc | 2026-03-05 | done |
| A2-013 | 执行 C2-001/C2-002 物理合并并回写历史入口（替代链/索引/审计备注） | cc | 2026-03-05 | done |

## 特殊情况备注（仅在无需合并时填写）
| 编号 | 原因 | 风险 | 临时缓解 | 复审日期 | 评审人 |
| --- | --- | --- | --- | --- | --- |

## 复审结果
- 复审时间：2026-03-05
- 复审结论：ROUND-002 已完成；候选簇 C2-001~C2-011 全部完成判定与文档回写，并补充 C2-001/C2-002 物理合并记录，`S_round002` 清单已刷新。
- 当前进展：已完成分区盘点，并已落地 C2-001、C2-002、C2-003、C2-004、C2-005、C2-006、C2-007、C2-008、C2-009、C2-010、C2-011。
