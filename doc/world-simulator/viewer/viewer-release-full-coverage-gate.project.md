# Viewer 发行全覆盖验收 Gate（项目管理）

## 任务拆解
- [x] RFCG-0 文档建档：设计文档 + 项目管理文档
- [x] RFCG-1 扩展 `viewer-theme-pack-preview.sh`：支持主题包选择（`industrial_v2|industrial_v1`）
- [x] RFCG-2 新增 `viewer-release-full-coverage.sh`：编排可用性/视觉/玩法全覆盖门禁并输出汇总报告
- [x] RFCG-3 更新 `testing-manual.md`：新增全覆盖验收入口、产物与口径说明
- [x] RFCG-4 回归验证与收口：脚本语法检查 + 快速模式冒烟 + 项目状态回写
- [x] RFCG-5 修复视觉门禁假阳性：主题/纹理 gate 强制 `connected + snapshot_ready`，并补抓帧端口就绪等待

## 依赖
- `scripts/viewer-release-qa-loop.sh`
- `scripts/viewer-theme-pack-preview.sh`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`
- `scripts/llm-longrun-stress.sh`
- `scripts/validate-viewer-theme-pack.py`
- `testing-manual.md`

## 状态
- 当前阶段：RFCG 全部完成（RFCG-0 ~ RFCG-5）。
- 下一步：按发布节奏将全覆盖脚本接入 nightly 或 release-candidate 人工门禁。
- 最近更新：2026-02-21（补齐视觉环节 connected 硬门禁并修复默认抓帧断连竞态）。
