# 文档迁移并行协作方案（2026-03-03）项目管理文档

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-ENGINEERING-010 (PRD-ENGINEERING-005): 冻结四人并行迁移原则、目录边界与执行流程，并产出协作主文档。
- [ ] TASK-ENGINEERING-011 (PRD-ENGINEERING-006): Owner-A 迁移 `doc/world-simulator/**` 待迁移文档（161 篇）。
- [x] TASK-ENGINEERING-012 (PRD-ENGINEERING-006): Owner-B 迁移 `doc/p2p/**` 待迁移文档（115 篇）。
- [ ] TASK-ENGINEERING-013 (PRD-ENGINEERING-006): Owner-C 迁移 `doc/world-runtime/**`、`doc/headless-runtime/**`、`doc/archive/root-history/**` 待迁移文档（52 篇）。
- [ ] TASK-ENGINEERING-014 (PRD-ENGINEERING-006): Owner-D 迁移 `doc/site/**`、`doc/readme/**`、`doc/scripts/**`、`doc/game/**`、`doc/engineering/**` 及根入口遗留文档（63 篇）。
- [ ] TASK-ENGINEERING-015 (PRD-ENGINEERING-007): 执行全量收口复核（命名一致性、引用可达、模块追踪同步、燃尽归零）。

## 依赖
- `doc/engineering/doc-migration/legacy-doc-migration-collaboration-2026-03-03.prd.md`
- `doc/engineering/doc-migration/legacy-doc-migration-backlog-2026-03-03.md`
- `doc/engineering/prd.md`
- `doc/engineering/prd.project.md`
- `doc/devlog/2026-03-03.md`
- `./scripts/doc-governance-check.sh`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 当前完成: 2 / 6（完成协作入口与 Owner-B 目录迁移）
- 下一任务: TASK-ENGINEERING-011 / TASK-ENGINEERING-013 / TASK-ENGINEERING-014（继续并行迁移执行）
- 风险备注: 大目录迁移期间需每日同步燃尽，防止 Owner 负载失衡。
