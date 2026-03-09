# 客户端启动器 Native 遗留代码清理（2026-03-06）项目管理文档

审计轮次: 5
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-022) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-022) [test_tier_required]: 落地 native 遗留代码清理（状态字段/常量边界收敛 + 删除未引用旧测试文件）并执行 required 回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/launcher_core.rs`
- `crates/agent_world_client_launcher/src/main_tests.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-06
- 当前阶段: completed
- 当前任务: 无（T0/T1 已完成）
- 备注: native 遗留清理已完成，required 回归通过且功能语义保持不变。
