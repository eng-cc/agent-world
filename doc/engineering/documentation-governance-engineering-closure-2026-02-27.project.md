# 文档治理工程化全量优化（2026-02-27）项目管理文档

## 任务拆解
- [x] T0 建档：新增设计文档与项目管理文档
- [x] T1 新增文档治理检查脚本（结构/路径/行数）
- [x] T2 全量修复非归档文档绝对路径为相对路径
- [x] T3 接入 `scripts/ci-tests.sh` required/full 流程
- [x] T4 更新 `testing-manual.md` 文档治理门禁入口
- [x] T5 回归验证、项目文档状态收口、补齐当日 devlog

## 依赖
- `scripts/ci-tests.sh`
- `testing-manual.md`
- `doc/**/*.project.md`（非 archive）
- `doc/**/*.md`（非 archive / 非 devlog）

## 状态
- 当前阶段：已完成（T0~T5）
- 阻塞项：无
- 最近更新：2026-02-27（完成 T5，项目收口）
