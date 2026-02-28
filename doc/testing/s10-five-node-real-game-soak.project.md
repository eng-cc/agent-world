# Agent World：S10 五节点真实游戏数据在线长跑套件（项目管理文档）

## 任务拆解
- [x] T0：完成 S10 设计文档与项目管理文档建档。
  - [x] `doc/testing/s10-five-node-real-game-soak.md`
  - [x] `doc/testing/s10-five-node-real-game-soak.project.md`
- [x] T1：实现 `scripts/s10-five-node-game-soak.sh` 五节点编排脚本。
  - [x] 五节点启动/停止与输出目录管理。
  - [x] 指标聚合与 `summary.json`/`summary.md` 生成。
  - [x] `--dry-run` 与 `--help` 支持。
- [ ] T2：接入 `testing-manual.md` 的 S10 章节与触发矩阵说明。
- [ ] T3：执行脚本级验证（语法/帮助/dry-run）并收口文档状态与 devlog。

## 依赖
- `scripts/p2p-longrun-soak.sh`（复用指标口径与产物约定）
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（T0~T1 已完成，执行 T2）。
- 阻塞项：无。
- 最近更新：2026-02-28。
