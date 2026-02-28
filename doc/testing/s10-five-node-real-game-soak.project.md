# Agent World：S10 五节点真实游戏数据在线长跑套件（项目管理文档）

## 任务拆解
- [x] T0：完成 S10 设计文档与项目管理文档建档。
  - [x] `doc/testing/s10-five-node-real-game-soak.md`
  - [x] `doc/testing/s10-five-node-real-game-soak.project.md`
- [x] T1：实现 `scripts/s10-five-node-game-soak.sh` 五节点编排脚本。
  - [x] 五节点启动/停止与输出目录管理。
  - [x] 指标聚合与 `summary.json`/`summary.md` 生成。
  - [x] `--dry-run` 与 `--help` 支持。
- [x] T2：接入 `testing-manual.md` 的 S10 章节与触发矩阵说明。
- [x] T3：执行脚本级验证（语法/帮助/dry-run）并收口文档状态与 devlog。
- [x] T4：增加可控奖励 epoch 时长（测试/长跑门禁可用性修复）。
  - [x] `world_viewer_live` 增加 `--reward-runtime-epoch-duration-secs <n>`。
  - [x] `scripts/s10-five-node-game-soak.sh` 接线并默认使用短 epoch（60s）。
  - [x] 执行真实短跑验证，确认稳定产生 `epoch-*.json`（`no_epoch_reports` 问题消失）。
- [x] T5：修复 `minted_records_empty` 门禁（结算发布与 mint 前置条件）。
  - [x] 结算发布判定支持“已存在 committed checkpoint（高度>0）”的 Pending 状态，避免发布链路长期跳过。
  - [x] 奖励运行时预绑定并补齐 settlement 涉及节点身份，确保 mint 签名校验路径可执行。
  - [x] 执行真实短跑验证，确认 `minted_records` 样本出现且门禁通过。
- [x] T6：修复 no-LLM 长跑 `committed_height` 长时间不增长（stall）问题。
  - [x] `world_viewer_live` 单节点模式仅在 sequencer 角色启用执行 hook 与执行强约束，避免非 sequencer 同步提交时触发 contiguous-height 错误导致共识卡死。
  - [x] `scripts/s10-five-node-game-soak.sh` 增加节点状态隔离（默认搬迁历史 `output/node-distfs/s10-*`），避免跨 run 污染导致恢复到旧 pending 提案。
  - [x] `scripts/s10-five-node-game-soak.sh` 增加可控 auto-attest 策略（默认 `sequencer_only`，可切 `all/off`），在五节点 no-LLM 长跑中保持持续推进且避免多节点高度分叉。
  - [x] 执行真实短跑回归（`260s` + `max_stall=120`），确认门禁通过且高度持续增长。

## 依赖
- `scripts/p2p-longrun-soak.sh`（复用指标口径与产物约定）
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `testing-manual.md`

## 状态
- 当前阶段：已完成（T0~T6）。
- 阻塞项：无。
- 最近更新：2026-02-28。
