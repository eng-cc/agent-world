# Core 重复/合并审查记录（第002轮，启动稿）

## 目的
- 为 `TASK-CORE-005` 提供 ROUND-002 审查入口，聚焦“文档重复内容识别与可合并评估”。
- 本轮主线-1：识别跨文档重复（目标、范围、规格矩阵、验收条款、操作步骤）。
- 本轮主线-2：对可合并文档给出“保留文档 + 合并策略 + 替代链 + 索引回写”并执行。

## 轮次信息
- 轮次编号: `ROUND-002`
- 轮次状态: `in_progress` (`not_started` | `in_progress` | `completed`)
- 审查时间窗: 2026-03-05 ~ 进行中
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
rg -n "^审计轮次:\\s*2$" doc --glob '*.md'
```

## 启动范围（重复高风险分区）
- A: `world-simulator/viewer`（约 96 份 `.prd.md`，日期专题密集）
- B: `p2p/node + p2p/distfs + p2p/observer`（约 46 份 `.prd.md`）
- C: `testing/ci + scripts/precommit`（测试门禁专题口径复用高）
- D: `site/github-pages + site/manual`（站点发布专题易出现流程重复）
- E: `readme/gap + game/gameplay`（治理/玩法总述文档语义重叠）

## 受审文件清单（S_round002）
- 清单文件：`doc/core/reviews/round-002-reviewed-files.md`
- 生成规则：`rg -l "^审计轮次:\\s*2$" doc --glob '*.md' | sort`
- 当前基线（2026-03-05 启动）：`0` 份文档
- 用途：作为 `A2-006` 的统计分母（仅对纳入本轮清单的文档判定“已审读/未审读”）。

## 一致性问题（重复/合并维度，初始登记）
| 编号 | 问题描述 | 影响范围 | 严重度 |
| --- | --- | --- | --- |
| I2-001 | Viewer 日期专题文档簇存在高重复“验收流程与命令模板”，维护成本高。 | `doc/world-simulator/viewer/*` | high |
| I2-002 | P2P 节点/DistFS 连续阶段文档存在阶段间“目标/范围/验收”重复块。 | `doc/p2p/node/*`、`doc/p2p/distfs/*` | high |
| I2-003 | CI 分层专题与 pre-commit 专题存在规则描述重复，容易双处漂移。 | `doc/testing/ci/*`、`doc/scripts/precommit/*` | medium |
| I2-004 | Site 手册/发布专题存在步骤级重复，入口索引可读性下降。 | `doc/site/manual/*`、`doc/site/github-pages/*` | medium |
| I2-005 | README gap 与 gameplay 总述文档存在“资源/治理叙述”重合，存在合并或主从化空间。 | `doc/readme/gap/*`、`doc/game/gameplay/*` | medium |

## 合并候选批次（启动版）
| 编号 | 候选文档簇 | 候选主文档 | 合并策略 | 状态 |
| --- | --- | --- | --- | --- |
| C2-001 | `viewer-gameplay-release-immersion-phase*.prd*.md` | 待定（建议保留最终阶段 + 汇总页） | 同类阶段文档“主从化 + 历史专题分组” | open |
| C2-002 | `viewer-live-full-event-driven-phase8~11*.prd*.md` | 待定 | 统一验收口径并压缩重复段落 | open |
| C2-003 | `node-redeemable-power-asset*.prd*.md` 系列 | 待定 | 规范“主文档 + 增量专题”边界 | open |
| C2-004 | `testing/ci` 分层专题与 `pre-commit` 专题 | `ci-tiered-execution.prd.md` + `pre-commit.prd.md` | 固定规则归属，去除重复说明 | open |
| C2-005 | `site/manual` 日期专题 | 待定 | 转“历史专题 + 替代链”并收敛到模块主入口 | open |
| C2-006 | `readme/gap` 与 `gameplay` 总述 | 待定 | 统一术语主入口，保留单一权威口径 | open |

## 整改项
| 编号 | 整改动作 | 责任人 | 截止时间 | 状态 |
| --- | --- | --- | --- | --- |
| A2-001 | 建立 ROUND-002 重复识别与合并执行台账（本文件 + 工作清单 + 审读清单） | core 维护者 | 2026-03-05 | done |
| A2-002 | 完成分区 A/B 的重复簇盘点并给出“可合并/保留”判定 | world-simulator + p2p 维护者 | 2026-03-07 | open |
| A2-003 | 完成分区 C 的规则归属收口（CI/pre-commit 去重） | testing + scripts 维护者 | 2026-03-07 | open |
| A2-004 | 完成分区 D/E 的主入口收口与历史专题分组策略 | site + readme + game 维护者 | 2026-03-08 | open |
| A2-005 | 执行首批合并/主从化迁移并回写替代链、索引、redirect | core + 各模块维护者 | 2026-03-09 | open |
| A2-006 | 对已完成 ROUND-002 审读的文档回写 `审计轮次: 2`，并以 `S_round002` 为统计分母 | 各模块维护者 | 2026-03-09 | open |

## 特殊情况备注（仅在无需合并时填写）
| 编号 | 原因 | 风险 | 临时缓解 | 复审日期 | 评审人 |
| --- | --- | --- | --- | --- | --- |

## 复审结果
- 复审时间：
- 复审结论：
