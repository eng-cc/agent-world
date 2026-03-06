# core PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-CORE-001 (PRD-CORE-001) [test_tier_required]: 完成 core PRD 改写，固化跨模块治理基线。
- [x] TASK-CORE-002 (PRD-CORE-001/002/003) [test_tier_required]: 将 core PRD 扩展为项目全局总览入口（模块地图/关键链路/关键分册导航）。
- [x] TASK-CORE-003 (PRD-CORE-001/002) [test_tier_required]: 建立跨模块变更影响检查清单（设计/代码/测试/发布），并固化 N/A、整改项与特殊备注机制。
  - 产物文件: `doc/core/checklists/cross-module-impact-checklist.md`
  - 验收命令 (`test_tier_required`):
    - `test -f doc/core/checklists/cross-module-impact-checklist.md`
    - `rg -n "适用范围|输入|检查项|阻断条件|回滚策略" doc/core/checklists/cross-module-impact-checklist.md`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-CORE-004 (PRD-CORE-002/003) [test_tier_required]: 建立仓库级 PRD-ID 到测试证据映射模板，固化字段定义、层级口径与最小审查清单。
  - 产物文件: `doc/core/templates/prd-id-test-evidence-mapping.md`
  - 验收命令 (`test_tier_required`):
    - `test -f doc/core/templates/prd-id-test-evidence-mapping.md`
    - `rg -n "PRD-ID|任务ID|测试层级|命令|证据路径|结论" doc/core/templates/prd-id-test-evidence-mapping.md`
    - `./scripts/doc-governance-check.sh`
- [ ] TASK-CORE-005 (PRD-CORE-003) [test_tier_required]: 对模块 PRD 按轮次进行一致性审查并形成审查记录（含轮次状态与文档级审计轮次字段，缺省按 0 处理）。
  - 产物文件: `doc/core/reviews/consistency-review-round-001.md`
  - 验收命令 (`test_tier_required`):
    - `ls doc/core/reviews/consistency-review-round-*.md`
    - `rg -n "轮次编号|轮次状态|审计轮次|缺省=0|抽样范围|一致性问题|整改项|责任人|截止时间|复审结果" doc/core/reviews/consistency-review-round-*.md`
    - `./scripts/doc-governance-check.sh`
  - ROUND-002 启动产物（2026-03-05）:
    - `doc/core/reviews/consistency-review-round-002.md`
    - `doc/core/reviews/round-002-reviewed-files.md`
    - `doc/core/reviews/round-002-dedup-merge-worklist.md`
  - ROUND-003 启动产物（2026-03-05）:
    - `doc/core/reviews/consistency-review-round-003.md`
    - `doc/core/reviews/round-003-reviewed-files.md`
    - `doc/core/reviews/round-003-filename-semantic-worklist.md`
  - ROUND-004 启动产物（2026-03-06）:
    - `doc/core/reviews/consistency-review-round-004.md`
    - `doc/core/reviews/round-004-reviewed-files.md`
    - `doc/core/reviews/round-004-doc-design-quality-worklist.md`
  - ROUND-002 进展（2026-03-05）:
    - 已完成 A/B/C/D-E 分区重复簇盘点并回写到 ROUND-002 台账。
    - 已完成首批执行 `C2-007`：`viewer-chat-agent-prompt-default-values-inline-input` 并入 `prefill` 且旧文档已删除，替代链与索引已回写。
    - 已完成第二批执行 `C2-004`：CI/precommit 规则归属收口（CI 文档链定义规则，precommit/precommit-remediation-playbook 保留入口与修复流程）。
    - 已完成第三批子簇 `B3-C2-009-S1`：observer sync source 主从化（`source-mode` 主文档 + `source-dht-mode` 增量子文档）。
    - 已完成并行批次 `B3-C2-009-S2/C2-010/C2-011`：observer sync-mode、node-contribution、distfs-self-healing 三簇主从化收口并回写审计轮次。
    - 已完成并行批次 `B3-C2-003/C2-008-S1/C2-008-S2`：node-redeemable-power-asset 与 distfs-production-hardening（phase1~9）主从化收口并回写审计轮次。
    - 已完成并行批次 `B4-C2-005-S1/S2`：site/manual 与 github-pages 主从化收口并回写审计轮次。
    - 已完成并行批次 `B5-C2-006-S1/S2`：readme/gap 与 game/gameplay 主从化收口并回写审计轮次。
    - 已完成并行批次 `B6-C2-001/C2-002`：viewer phase8~10 簇收口并回写审计轮次，ROUND-002 进入 completed。
    - 已完成补充批次 `B7-C2-001/C2-002`：viewer phase8~10 物理合并并回写历史入口/索引。
  - ROUND-003 进展（2026-03-05）:
    - 已完成全量命名审读并回写 `审计轮次: 3`，`S_round003` 清单已生成。
    - 已登记命名问题 `I3-001~I3-011`（viewer/launcher/p2p/engineering/headless-runtime/scripts/site 命名不一致），并已完成更名与引用回写。
    - ROUND-003 已于 2026-03-06 收口为 `completed`（复审结论已落档）。
  - ROUND-004 进展（2026-03-06）:
    - 已完成 ROUND-004 启动台账，聚焦文档设计质量八项维度（信息架构、分工边界、追溯闭环、可执行性、权威源、状态时效、术语一致性、发布可达性）。
    - 已启动 6 个子代理并行分区审读（core/engineering、world-simulator、p2p、testing/scripts/playability、site/readme/game、world-runtime/headless-runtime）。
    - 已补充“逐文档即时回写”机制：每篇文档审读完成即回写 `审计轮次: 4` 并登记 `round-004-audit-progress-log.md`，用于中断恢复追溯。
    - 已闭环工作失误 `E4-001`：纠偏后采用“`doc/**/*.md` 排除 `doc/devlog/**`”作为 ROUND-004 分母口径。
    - 已完成 `F4-001~F4-004`：补审 `doc/world-simulator/**` 至 `314/314`，并将 `S_round004` 更新到 `788`（非 devlog 全覆盖），`A4-002/A4-006` 已恢复 `done`。
    - 已执行首批整改：完成 `A4-009/A4-010`（清理重复行内审计字段、修正 `site/site/doc` 路径、修复 pre-commit 占位命令口径并更新验收命令）。
- [x] TASK-CORE-006 (PRD-CORE-001/002) [test_tier_required]: 收敛 `doc/` 根目录 legacy redirect 入口并更新总导航。
- [x] TASK-CORE-007 (PRD-CORE-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- doc/core/prd.index.md
- `AGENTS.md`
- `doc/README.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`
- 各模块 `doc/<module>/prd.md` 与 `doc/<module>/prd.project.md`

## 状态
- 更新日期: 2026-03-06
- 当前状态: active
- 下一任务: TASK-CORE-005
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 core 设计执行状态；过程记录在 `doc/devlog/2026-03-06.md`（历史记录见同目录其他日期文件）。
