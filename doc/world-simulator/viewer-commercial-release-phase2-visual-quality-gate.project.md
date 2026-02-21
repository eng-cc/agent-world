# Viewer 商业化发行缺口收敛 Phase 2（项目管理）

## 任务拆解
- [x] VCR2-0 文档建档：设计文档 + 项目管理文档
- [x] VCR2-1 新增视觉基线脚本（snapshot 基线校验 + 定向测试）
- [ ] VCR2-2 接入 CI：`ci-tests.sh` 与 `rust.yml` 增加 viewer 相关门禁
- [ ] VCR2-3 更新测试手册与项目状态，补 devlog 收口

## 依赖
- `scripts/viewer-visual-baseline.sh`
- `scripts/ci-tests.sh`
- `.github/workflows/rust.yml`
- `testing-manual.md`

## 状态
- 当前阶段：VCR2-0、VCR2-1 已完成，VCR2-2 进行中。
- 下一步：将 viewer 单测/wasm gate 与视觉基线脚本接入 CI required gate。
- 最近更新：2026-02-21。
