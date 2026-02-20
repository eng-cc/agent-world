# Doc 目录分层整理（2026-02-20）项目管理文档

## 任务拆解
- [x] T0：输出设计文档与项目管理文档。
- [ ] T1：新建分层目录并迁移 `readme-*`、`github-pages-*`、`m4-*` 文档（含 `.project.md`）。
- [ ] T2：修复非 `doc/devlog` 文档中的迁移路径引用，并做引用扫描校验。
- [ ] T3：新增 `doc/README.md` 目录入口，完成测试与收口回写。

## 依赖
- `doc/` 下现有设计文档与项目管理文档文件路径。
- 仓库内对上述路径的交叉引用（`README.md`、`doc/**/*.md`、`testing-manual.md`）。
- 检查命令：`env -u RUSTC_WRAPPER cargo check -p agent_world --lib`。

## 状态
- 当前阶段：T0 已完成，T1 进行中。
- 最近更新：2026-02-20。
