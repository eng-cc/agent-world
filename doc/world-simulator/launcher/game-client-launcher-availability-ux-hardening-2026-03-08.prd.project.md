# 客户端启动器可用性与体验硬化（2026-03-08）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-availability-ux-hardening-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-027) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-027) [test_tier_required]: 落地启动器可用性与 UX 硬化修复（静态目录回退、wasm 禁用原因提示、查询参数编码、stop no-op 状态语义、移动端布局、favicon 噪声）并完成跨端回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_web_launcher/runtime_paths.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- `crates/agent_world_client_launcher/src/platform_ops.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/launcher_core.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/index.html`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: in_progress
- 当前任务: T1（启动器可用性与体验硬化修复）
- 备注: T0 已完成，T1 待实现与回归。
