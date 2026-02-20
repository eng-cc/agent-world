# Doc 目录结构与内容时效复核（Round 2，2026-02-20）项目管理文档

## 任务拆解
- [x] R0：新增 round2 设计文档与项目管理文档，冻结复核口径。
- [x] R1：读取并扫描 `doc/**/*.md`，生成结构/引用/代码路径一致性清单。
- [ ] R2：逐目录人工复核（文档 + 关联文档 + 当前代码），确定归档与保留修正集合。
- [ ] R3：执行归档迁移与引用修复（如有），输出结果文档与回归测试。

## 依赖
- 文档目录：`doc/**/*.md`
- 代码目录：`crates/`、`scripts/`、`site/`、`tools/`
- 校验命令：`env -u RUSTC_WRAPPER cargo check -p agent_world --lib`

## 状态
- 当前阶段：R0-R1 已完成，R2 进行中。
- R1 产出：
  - `output/doc-structure-freshness-review-round2-2026-02-20.json`
  - 全量覆盖 `529` 篇文档（`active=394`、`archive=117`、`devlog=18`）。
  - 活跃文档无 markdown 断链，进入人工复核阶段确认“阶段性旧文档是否应归档”。
- 最近更新：2026-02-20。
