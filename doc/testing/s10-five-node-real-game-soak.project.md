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
- [x] T7：修复 sequencer 执行桥在非连续 committed 高度下的偶发卡死。
  - [x] `execution_bridge` 在 `on_commit` 遇到高度跳变时改为容错续跑（记录告警并继续处理当前高度），不再直接返回 contiguous-height 错误。
  - [x] 增加单元测试覆盖非连续高度提交场景（`height: 1 -> 3`）。
  - [x] 执行真实长跑回归：失败样本 `20260228-165423`（旧二进制，`stall=842s`）与修复后样本 `20260228-172637`（新二进制，`600s` 门禁通过，`max_stall_secs_observed=0`）。
- [x] T8：结算签名错误清零（五节点 `stderr` 无结算验签失败）。
  - [x] `world_viewer_live` 奖励 signer 改为“按 signer node_id 从根密钥派生”并统一用于签名。
  - [x] `reward_runtime_node_identity_bindings` 移除 signer 特殊 root 绑定，所有节点身份统一派生规则。
  - [x] 结算 envelope 网络编码切换为 CBOR（解码兼容 JSON 回退），修复跨节点验签失败。
  - [x] 执行真实短长跑回归：`20260228-180015`（`260s`）五节点 `stderr` 中 `apply settlement envelope failed` / `verify settlement signature failed` / `settlement signer public key mismatch` 计数均为 0。
- [ ] T9：将“结算 envelope 应用失败率”纳入硬门禁。
  - [ ] 在 reward runtime epoch 报表中暴露结算应用尝试/失败累计计数。
  - [ ] `scripts/s10-five-node-game-soak.sh` 增加 `max_settlement_apply_failure_ratio` 阈值与门禁判定。
  - [ ] `summary.json`/`summary.md` 增加结算应用失败率指标输出。
- [ ] T10：执行连续多轮长跑稳定性验证（3x600s + 1x 更长）并收口发布结论。
  - [ ] 连续跑测产物落盘并逐轮核验关键告警为 0。
  - [ ] 汇总门禁结果并更新项目状态为完成。

## 依赖
- `scripts/p2p-longrun-soak.sh`（复用指标口径与产物约定）
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（T8 已完成，执行 T9）。
- 阻塞项：无。
- 最近更新：2026-02-28。
