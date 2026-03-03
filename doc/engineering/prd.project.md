# engineering PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-ENGINEERING-001 (PRD-ENGINEERING-001): 完成 engineering PRD 改写，建立工程治理主文档。
- [x] TASK-ENGINEERING-002 (PRD-ENGINEERING-001/002): 补齐高频违规（超行数/超文档长度/文档平铺新增）自动诊断与门禁建议。
- [x] TASK-ENGINEERING-005 (PRD-ENGINEERING-001/002): 执行文档平铺存量迁移批次（world-simulator/p2p），并更新 allowlist 与引用路径。
- [x] TASK-ENGINEERING-006 (PRD-ENGINEERING-001/002): 执行文档平铺存量迁移批次（world-runtime/testing/site/readme/scripts/game/headless-runtime），并更新 allowlist 与引用路径。
- [ ] TASK-ENGINEERING-003 (PRD-ENGINEERING-002/003): 建立工程门禁趋势统计（违规率、修复时长）。
- [ ] TASK-ENGINEERING-004 (PRD-ENGINEERING-003): 增加工程规范季度审查流程与记录模板。
- [x] TASK-ENGINEERING-007 (PRD-ENGINEERING-001/002/003): 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-ENGINEERING-008 (PRD-ENGINEERING-004): 按“逐篇阅读 + 人工重写”完成老格式文档迁移试点（`oversized-rust-file-splitting-round3-2026-02-23`）并补齐内容保真映射。
- [ ] TASK-ENGINEERING-009 (PRD-ENGINEERING-004): 按模块分批推进活跃老格式文档逐篇人工迁移并持续回写 PRD-ID / project / devlog。
- [x] TASK-ENGINEERING-010 (PRD-ENGINEERING-005/006/007): 产出四人并行迁移协作方案与 2026-03-03 待迁移清单快照，冻结迁移原则和目录责任域。
- [x] TASK-ENGINEERING-011 (PRD-ENGINEERING-006): Owner-A 迁移 `doc/world-simulator/**` 待迁移文档（161 篇）。
- [x] TASK-ENGINEERING-012 (PRD-ENGINEERING-006): Owner-B 迁移 `doc/p2p/**` 待迁移文档（115 篇）。
- [x] TASK-ENGINEERING-013 (PRD-ENGINEERING-006): Owner-C 迁移 `doc/world-runtime/**`、`doc/headless-runtime/**`、`doc/archive/root-history/**` 待迁移文档（52 篇）。
- [x] TASK-ENGINEERING-013A (PRD-ENGINEERING-006): Owner-C Batch-C1 迁移 `doc/archive/root-history/**` 待迁移文档（7 篇）。
- [x] TASK-ENGINEERING-013B (PRD-ENGINEERING-006): Owner-C Batch-C2 迁移 `doc/headless-runtime/**` 待迁移文档（6 篇）。
- [x] TASK-ENGINEERING-013C (PRD-ENGINEERING-006): Owner-C Batch-C3 迁移 `doc/world-runtime/archive/**`、`doc/world-runtime/governance/**`、`doc/world-runtime/module/**`、`doc/world-runtime/wasm/**` 待迁移文档（23 篇）。
- [x] TASK-ENGINEERING-013D (PRD-ENGINEERING-006): Owner-C Batch-C4 迁移 `doc/world-runtime/runtime/**` 待迁移文档（16 篇）。
- [x] TASK-ENGINEERING-014 (PRD-ENGINEERING-006): Owner-D 迁移 `doc/site/**`、`doc/readme/**`、`doc/scripts/**`、`doc/game/**`、`doc/engineering/**` 与根入口遗留文档（63 篇，D1/D2 已完成）。
- [x] TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006): Owner-D 已完成非根入口 60 篇 legacy 迁移（`*.md/*.project.md -> *.prd.md/*.prd.project.md`）。
- [x] TASK-ENGINEERING-014-D2 (PRD-ENGINEERING-006): 完成 3 份根入口 redirect project 文档收口（`doc/game-test.prd.project.md`、`doc/world-runtime.prd.project.md`、`doc/world-simulator.prd.project.md`）。
- [ ] TASK-ENGINEERING-015 (PRD-ENGINEERING-007): 执行全量迁移收口复核（命名一致性、引用可达、模块追踪同步、燃尽归零）。
- [ ] TASK-ENGINEERING-016 (PRD-ENGINEERING-008): 为 12 个模块补齐文件级 PRD 索引，并从模块入口文档建立可达引用。
- [ ] TASK-ENGINEERING-017 (PRD-ENGINEERING-009): 在 `scripts/doc-governance-check.sh` 新增专题 `*.prd.md <-> *.prd.project.md` 双向互链门禁。
- [ ] TASK-ENGINEERING-018 (PRD-ENGINEERING-010): 在 12 个模块 `prd.project.md` 的任务项显式标注 `test_tier_required/full`。

## 依赖
- `AGENTS.md`
- `doc/scripts/precommit/pre-commit.prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`
- `doc/engineering/*.md`
- `doc/engineering/oversized-rust-file-splitting-round3-2026-02-23.prd.md`
- `doc/engineering/oversized-rust-file-splitting-round3-2026-02-23.prd.project.md`
- `doc/engineering/doc-migration/legacy-doc-migration-collaboration-2026-03-03.prd.md`
- `doc/engineering/doc-migration/legacy-doc-migration-collaboration-2026-03-03.prd.project.md`
- `doc/engineering/doc-migration/legacy-doc-migration-backlog-2026-03-03.md`
- `scripts/doc-governance-check.sh`
- `doc/*/README.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-ENGINEERING-016（模块文件级索引补齐）
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 并行迁移状态: 分工与快照已冻结（TASK-ENGINEERING-010 完成）；Owner-A 已完成 `doc/world-simulator/**`（161/161），Owner-B 已完成 `doc/p2p/**`（115/115），Owner-C 已完成 `doc/world-runtime/**`、`doc/headless-runtime/**`、`doc/archive/root-history/**`（52/52），Owner-D 责任域 63 篇已全部完成（D1/D2 结项）。
- 当前整改批次: R1（索引闭环 / 互链门禁 / 任务 tier 显式）待执行 TASK-ENGINEERING-016/017/018。
- 说明: 本文档仅维护 engineering 设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
