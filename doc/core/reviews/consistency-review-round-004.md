# Core 文档设计质量审查记录（第004轮）

审计轮次: 4

## 目的
- 为 `TASK-CORE-005` 提供 ROUND-004 审查入口，聚焦“文档设计质量与可维护性”。
- 本轮主线-1：检查文档体系的信息架构、分工边界、追溯闭环与可执行性。
- 本轮主线-2：识别设计层面的结构性缺陷，输出整改方案并完成回写。

## 轮次信息
- 轮次编号: `ROUND-004`
- 轮次状态: `in_progress` (`not_started` | `in_progress` | `completed`)
- 审查时间窗: 2026-03-06 ~ 2026-03-06
- 审查负责人: cc

状态判定：
- `not_started`: 仅完成启动文档与方法准备，尚未形成有效问题记录。
- `in_progress`: 已开始审读并登记问题/整改项，但未形成最终复审结论。
- `completed`: 设计问题已收口，整改项关闭或已登记批准延期备注。

## 文档级审计标记方法（缺省=0）
- 每个受审文档采用字段 `审计轮次: <整数>` 标识最新已完成审计轮次。
- 本轮执行规则：
  - 单篇文档完成 ROUND-004 审读后，在同一提交回写 `审计轮次: 4`（与是否需要整改解耦）。
  - 若判定“设计合格无需整改”，仍应回写 `审计轮次: 4`，并在本记录登记判定理由。
  - 若实施整改，需同步更新：`prd.md`/`prd.index.md`/`prd.project.md` 与引用路径。
  - 若尚未完成 ROUND-004 审读，则保持原值（缺失按 `0`）。
- 本轮完成条件：纳入 `S_round004` 的文档全部满足 `审计轮次 >= 4`，且复审结论已落档。

### 即时回写机制（抗中断）
- 每读完 1 篇文档，必须立即完成两件事：
  - 回写该文档 `审计轮次: 4`。
  - 在审计进度日志登记 1 条结果记录（含时间、审计人、文档路径、结论、问题编号）。
- 严禁“先读一批、最后统一回写”；若中断，已登记日志应能准确反映已审读范围。
- 若仅发现问题但暂不整改，仍回写 `审计轮次: 4`，并在日志标记 `issue_open` 与 I4-* 编号。
- 若文档因冲突/权限暂无法回写，需先在日志标记 `blocked` 并记录原因，再进入下一个文档。

建议统计命令：
```bash
rg -n "^审计轮次:\s*4$" doc --glob '*.md'
```

## 重点审计维度（文档设计视角）
| 编号 | 维度 | 审计目标 | 严重度判定 |
| --- | --- | --- | --- |
| D4-001 | 信息架构 | `prd.md / prd.index.md / prd.project.md` 入口层级清晰、无多入口冲突 | 入口冲突=high |
| D4-002 | 文档分工边界 | PRD 仅写 Why/What/Done，Project 仅写 How/When/Who，Devlog 仅写过程 | 分工串写=high |
| D4-003 | 可追溯闭环 | `PRD-ID -> TASK -> 验收命令 -> 测试证据` 全链可反查 | 断链=high |
| D4-004 | 可执行性 | 验收命令真实可跑、与正文描述一致、可复现 | 命令失效=high |
| D4-005 | 权威源与去重 | 同一规则/术语仅一处权威定义，其它文档只引用 | 双源漂移=high |
| D4-006 | 状态与时效 | 文档状态与更新时间真实，过时文档不误标 active | 状态失真=medium |
| D4-007 | 术语与规格一致性 | 核心术语、状态机、字段命名跨模块一致 | 语义冲突=medium |
| D4-008 | 发布可达性 | 站点链接、索引可达性、双语映射完整（无 404/孤儿文档） | 断链=high |

## 启动范围（设计风险分区）
- A: `doc/core/*` + `doc/engineering/*`（治理规则、模板与审计台账）
- B: `doc/world-simulator/*` + `doc/p2p/*`（专题密集区，追溯链复杂）
- C: `doc/testing/*` + `doc/scripts/*`（验收命令与门禁口径）
- D: `doc/site/*` + `doc/readme/*` + `doc/game/*`（外部入口与术语一致性）
- E: 其余模块与根入口（补查）

## 并行审计编组（6 子代理）
- G4-001 `Locke`：`doc/core/**` + `doc/engineering/**`
- G4-002 `Aristotle`：`doc/world-simulator/**`
- G4-003 `Beauvoir`：`doc/p2p/**`
- G4-004 `Socrates`：`doc/testing/**` + `doc/scripts/**` + `doc/playability_test_result/**`
- G4-005 `Feynman`：`doc/site/**` + `doc/readme/**` + `doc/game/**`
- G4-006 `Turing`：`doc/world-runtime/**` + `doc/headless-runtime/**` + 根入口补查
- 执行策略：分区并行审读，主代理统一汇总 I4-* 并去重后回写本台账。

## 受审文件清单（S_round004）
- 清单文件：`doc/core/reviews/round-004-reviewed-files.md`
- 生成规则：`rg -l "^审计轮次:\s*4$" doc --glob '*.md' | sort`
- 当前基线（2026-03-06 11:46 CST）：`18` 份文档（增量执行中）
- 用途：作为 ROUND-004 统计分母（仅对纳入本轮清单的文档判定“已审读/未审读”）。

## 审计进度日志（逐文档）
- 日志文件：`doc/core/reviews/round-004-audit-progress-log.md`
- 记录粒度：1 文档 1 记录（即时写入，不延后）。
- 字段：`时间`、`审计人/代理`、`文档路径`、`结论(pass/issue_open/blocked)`、`问题编号`、`备注`。

## 设计问题清单
| 编号 | 问题描述 | 影响范围 | 建议动作 | 严重度 | 当前判定 |
| --- | --- | --- | --- | --- | --- |

## 整改项
| 编号 | 整改动作 | 责任人 | 截止时间 | 状态 |
| --- | --- | --- | --- | --- |
| A4-001 | 建立 ROUND-004 启动台账（本文件 + 审读清单 + 执行清单） | cc | 2026-03-06 | done |
| A4-002 | 完成 D4-001~D4-008 全量审读并登记问题清单 | cc | 2026-03-09 | in_progress |
| A4-003 | 对 high 严重度问题输出“整改方案 + 影响面 + 验收命令” | cc | 2026-03-10 | todo |
| A4-004 | 执行文档整改并回写审计轮次与索引引用 | cc | 2026-03-12 | todo |
| A4-005 | 生成 `S_round004` 清单并完成复审结论 | cc | 2026-03-12 | todo |
| A4-006 | 启动 6 子代理并行审计并回收分区问题清单 | cc | 2026-03-06 | in_progress |
| A4-007 | 落实“逐文档即时回写”机制（审计轮次 + 进度日志）并纳入并行审计流程 | cc | 2026-03-06 | done |
| A4-008 | 验收命令：`rg -n "PRD-ID|验收命令|证据" doc/core/prd.project.md doc/engineering/prd.project.md doc/p2p/prd.project.md doc/world-runtime/prd.project.md doc/headless-runtime/prd.project.md doc/testing/prd.project.md`<br>`rg -n "PRD-ID.*TASK|TASK.*PRD-ID" doc/core/prd.project.md doc/engineering/prd.project.md doc/p2p/prd.project.md doc/world-runtime/prd.project.md doc/headless-runtime/prd.project.md doc/testing/prd.project.md` | cc | 2026-03-10 | todo |
| A4-009 | 验收命令：`rg -n "env -u RUSTC_WRAPPER cargo check|\\./scripts/|bash scripts/" doc/site doc/testing doc/scripts doc/playability_test_result --glob '*.md'`<br>`! rg -n "\\$CODEX_HOME|<staged \\.rs files>|site/site/doc" doc/site doc/testing doc/scripts doc/playability_test_result --glob '*.md'` | cc | 2026-03-10 | todo |
| A4-010 | 验收命令：`./scripts/doc-governance-check.sh`<br>`! rg -n "site/site/doc|doc/world-simulator\\.prd\\.project\\.md" doc --glob '*.md'` | cc | 2026-03-10 | todo |
| A4-011 | 验收命令：`! rg -n "^- 审计轮次:\\s*[0-9]+" doc --glob '*.md'`<br>`rg -n "当前基线|当前已审读文档数" doc/core/reviews/consistency-review-round-004.md doc/core/reviews/round-004-reviewed-files.md` | cc | 2026-03-11 | todo |
| A4-012 | 验收命令：`! rg -n "完成内容|遗留事项|时刻" doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.md doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.md`<br>`rg -n "PRD-ID|任务拆解|验收命令" doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.project.md doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md` | cc | 2026-03-11 | todo |
| A4-013 | 验收命令：`! rg -n "doc/world-runtime\\.prd\\.md|ModuleValidationFailed" doc/world-runtime/testing/testing.md`<br>`rg -n "headless-runtime|nonviewer" doc/headless-runtime/README.md doc/headless-runtime/prd.md doc/headless-runtime/prd.project.md` | cc | 2026-03-11 | todo |
| A4-014 | 验收命令：`rg -n "legacy|快照|豁免|可达性" doc/engineering/doc-migration/legacy-doc-migration-backlog-2026-03-03.md`<br>`./scripts/doc-governance-check.sh` | cc | 2026-03-12 | todo |

## 特殊情况备注（仅在无需整改时填写）
| 编号 | 原因 | 风险 | 临时缓解 | 复审日期 | 评审人 |
| --- | --- | --- | --- | --- | --- |

## 复审结果
- 复审时间：-
- 复审结论：-
- 当前进展：ROUND-004 已启动并进入并行审读阶段（6 子代理分区执行）；已下发“逐文档即时回写”强制规则（读完即写 `审计轮次: 4` + 进度日志），待汇总 I4-* 问题清单。
