# Viewer 商业化发行缺口收敛 Phase 2：视觉验收基线与门禁

## 目标
- 建立可执行、可追溯的 Viewer 视觉验收基线，减少“主观观感回归”在发布前才暴露的风险。
- 将 Viewer 自身测试（单测 + wasm 编译）纳入统一 CI 流程，补齐当前核心门禁缺口。
- 提供可重复运行的视觉基线脚本，作为后续真实资产和表现层升级的验收入口。

## 范围

### In Scope
- 新增 `viewer` 视觉基线脚本（基于现有 snapshot 测试与基线文件完整性检查）。
- 将 `agent_world_viewer` 的单测与 wasm check 纳入 `scripts/ci-tests.sh`。
- 在 GitHub Actions 主流程中接入视觉基线脚本（required gate）。
- 更新测试手册与相关文档状态。

### Out of Scope
- 本阶段不引入真实美术资源包（模型/贴图/动画）批量导入。
- 本阶段不实现 Playwright 全自动在线链路门禁（保留手动/agent 闭环）。
- 本阶段不调整 Viewer 协议与运行时语义。

## 接口 / 数据
- 新增脚本：`scripts/viewer-visual-baseline.sh`
  - 校验 snapshot 基线文件存在性。
  - 执行 `cargo test -p agent_world_viewer egui_kittest_snapshot_`。
- `scripts/ci-tests.sh` 扩展：
  - `required/full` 都执行 `agent_world_viewer` 单测与 wasm check。
- `.github/workflows/rust.yml` 扩展：
  - `required-gate` job 调用视觉基线脚本。

## 里程碑
- VCR2-0：设计/项目管理文档建档。
- VCR2-1：视觉基线脚本实现。
- VCR2-2：CI 流程接入 viewer 测试与 wasm gate。
- VCR2-3：手册/测试手册/项目状态/devlog 收口。

## 风险
- CI 时长上升：
  - 缓解：仅接入必要的 viewer 套件与轻量 snapshot 子集。
- 无 GPU 环境下 snapshot 不稳定：
  - 缓解：沿用现有 snapshot 测试的“不可用即跳过”策略，并将基线文件完整性作为补充硬检查。
- 脚本门禁与手册口径不一致：
  - 缓解：同批更新 `testing-manual.md` 与项目管理文档状态。
