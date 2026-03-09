# world_viewer_live 发行锁定启动（P2P）项目管理文档（2026-02-23）（项目管理文档）

审计轮次: 5
> 状态更新（2026-03-08）:
> - 对应功能已从 `world_viewer_live` 下线（`--release-config` 不再支持）。
> - 本项目文档转为历史归档，后续控制面变更归并到 `world_chain_runtime` 路径任务。

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-P2P-MIG-115)：完成设计文档与项目管理文档建档。
- [x] T1 (PRD-P2P-MIG-115)：实现 `--release-config` 锁定参数加载与启动接线（含 TOML `locked_args` 解析）。
- [x] T2 (PRD-P2P-MIG-115)：实现发行锁定模式 CLI 白名单约束，并补参数解析测试（成功/拒绝/覆盖）。
- [x] T3 (PRD-P2P-MIG-115)：补发行配置样例与手册说明，执行 `world_viewer_live` 定向回归并回写文档/devlog 状态。

## 依赖
- doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.md
- T1 依赖 T0（接口与语义冻结后实现）。
- T2 依赖 T1（白名单约束建立在锁定加载路径之上）。
- T3 依赖 T1/T2（样例与手册需对齐最终 CLI 行为）。

## 状态
- 当前阶段：已归档（历史完成记录，当前实现已下线）。
- 阻塞项：无。
- 最近更新：2026-03-08。
