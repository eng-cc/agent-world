# Agent World：P2P/存储/共识在线长跑稳定性测试（项目管理文档）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] S9SOAK-1 (PRD-TESTING-LONGRUN-S9SOAK-001/002): 完成方案与项目管理文档建档。
- [x] S9SOAK-2 (PRD-TESTING-LONGRUN-S9SOAK-001/002): 实现 `scripts/p2p-longrun-soak.sh` 最小闭环（启动/停止/超时/清理/目录结构），支持 `triad/triad_distributed`。
- [x] S9SOAK-3 (PRD-TESTING-LONGRUN-S9SOAK-002/003): 实现 epoch JSON 聚合与门禁判定（stall/lag/distfs/invariant），输出 `timeline/summary/failures`。
- [x] S9SOAK-4 (PRD-TESTING-LONGRUN-S9SOAK-003): 支持 `--chaos-plan` 注入并输出 `chaos_events.log` 联动判定。
- [x] S9SOAK-5 (PRD-TESTING-LONGRUN-S9SOAK-001/002): 在 `testing-manual.md` 接入 S9 套件、执行剧本与触发矩阵。
- [x] S9SOAK-6 (PRD-TESTING-LONGRUN-S9SOAK-002/003): 完成 `soak_smoke` 与 `soak_endurance` 样本验证并收口。
- [x] S9SOAK-7 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md
- `testing-manual.md`
- `scripts/p2p-longrun-soak.sh`
- `scripts/ci-tests.sh`
- `scripts/viewer-owr4-stress.sh`
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world/src/bin/world_viewer_live/distfs_probe_runtime.rs`
- `crates/agent_world/src/bin/world_viewer_live/distfs_challenge_network.rs`
- `crates/agent_world_node/src/types.rs`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：按 S9 在夜间/发布前执行默认时长长跑（`soak_smoke 20~30 分钟`，`soak_endurance 180+ 分钟`）
