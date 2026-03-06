# Agent World: 系统性应用测试手册（Human/AI 通用）

## 目标
- 基于仓库当前实现，提供一套可直接执行的分层测试手册，让人类开发者与 AI Agent 都能对“整应用”做足够充分的测试。
- 解决“只跑一条命令看总绿灯”但无法定位风险层的问题，把测试明确拆成基础门禁、核心逻辑、协议集成、分布式子系统、UI 闭环、压力回归。
- 把 `test_tier_required` 与 `test_tier_full` 放回整体测试体系中：它们是核心层基线，不等于“整应用全覆盖”。
- 统一证据标准（命令、日志、截图、结论），保证测试可复盘、可审计。

## 范围

### In Scope
- 结合当前仓库真实实现给出分层模型与命令清单。
- 给出“改动路径 -> 应跑测试层级”的触发矩阵。
- 给出 Human/AI 共用执行剧本、通过标准、失败分诊与证据规范。
- 明确现有 CI 覆盖能力与手册补充覆盖能力的边界。

### Out of Scope
- 不在本任务修改 CI workflow 或测试脚本行为。
- 不引入新的测试框架或新的业务代码。
- 不做覆盖率百分比硬门槛治理（如行覆盖率 >= N%）。

## 当前实现分布（2026-02-18 基线）

### 应用主链（world + runtime + simulator + viewer 协议）
- 核心 crate：`crates/agent_world`
- 主要测试分布：
  - 运行时：`crates/agent_world/src/runtime/tests/*.rs`
  - 模拟器：`crates/agent_world/src/simulator/tests/*.rs`
  - LLM 行为：`crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
  - Viewer live 服务：`crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests.rs`
  - 端到端集成：`crates/agent_world/tests/*.rs`

### Viewer 客户端（Bevy/egui + wasm）
- crate：`crates/agent_world_viewer`
- 覆盖：
  - UI/相机/事件联动等单测散布在 `src/*.rs` 与 `src/tests_*.rs`
  - 快照基线：`crates/agent_world_viewer/tests/snapshots/*.png`
  - Web 启动入口：`world_game_launcher`（内置静态服务，`run-viewer-web.sh` 仅保留为兼容/排障工具）
  - Web 闭环采样：Playwright CLI（详见 `doc/testing/manual/web-ui-playwright-closure-manual.prd.md`）

### 分布式与共识子系统
- Node：`crates/agent_world_node`
- Net：`crates/agent_world_net`
- Consensus：`crates/agent_world_consensus`
- DistFS：`crates/agent_world_distfs`
- 这些子系统有独立测试集，但当前 `scripts/ci-tests.sh` 只覆盖了其中一部分（见下文“CI 现状与缺口”）。

### 场景系统
- 场景定义：`crates/agent_world/src/simulator/scenario.rs`
- 场景矩阵设计：`doc/world-simulator/scenario/scenario-files.prd.md`
- 场景是 UI 闭环、协议闭环、压力回归的统一输入源。

## CI 现状与缺口（事实口径）

### 当前 CI/脚本已覆盖
- 入口 A：`scripts/ci-tests.sh`（主流程）
- `required`：
  - `./scripts/doc-governance-check.sh`
  - `cargo fmt --check`
  - `cargo test -p agent_world --tests --features test_tier_required`
  - `cargo test -p agent_world_consensus --lib`
  - `cargo test -p agent_world_distfs --lib`
  - `cargo test -p agent_world_viewer`
  - `cargo check -p agent_world_viewer --target wasm32-unknown-unknown`
- `full`：
  - `required` 全部
  - `cargo test -p agent_world --tests --features test_tier_full,wasmtime,viewer_live_integration`
  - `cargo test -p agent_world_node --lib`
  - `cargo test -p agent_world_net --lib`
  - `cargo test -p agent_world_net --features libp2p --lib`
  - `./scripts/llm-baseline-fixture-smoke.sh`
  - `cargo test -p agent_world --features wasmtime --lib --bins`
- 入口 B：`.github/workflows/rust.yml`（required-gate）
  - `CI_VERBOSE=1 ./scripts/ci-tests.sh required`
  - `./scripts/viewer-visual-baseline.sh`
- 入口 C：`.github/workflows/builtin-wasm-m1-multi-runner.yml`（构建 hash 链路独立 gate）
  - runner 矩阵：`ubuntu-24.04 (linux-x86_64)` + `macos-14 (darwin-arm64)`
  - 每个 runner 仅执行：`./scripts/ci-m1-wasm-summary.sh --module-set m1 --runner-label ... --out ...`
  - 汇总 job 对账：`./scripts/ci-verify-m1-wasm-summaries.py --module-set m1 --summary-dir ... --expected-runners linux-x86_64,darwin-arm64`
- 入口 D：`.github/workflows/builtin-wasm-m4-m5-multi-runner.yml`（m4/m5 构建 hash 链路独立 gate）
  - runner 矩阵：`(m4|m5) x (ubuntu-24.04/linux-x86_64, macos-14/darwin-arm64)`
  - 每个 runner 执行：`./scripts/ci-m1-wasm-summary.sh --module-set <m4|m5> --runner-label ... --out ...`
  - 汇总 job 对账：`./scripts/ci-verify-m1-wasm-summaries.py --module-set <m4|m5> --summary-dir ... --expected-runners linux-x86_64,darwin-arm64`

### 当前 CI 未直接覆盖（需手册补齐）
- Web UI Playwright 闭环（现为手动/agent 流程，不在 CI 默认路径中）。
- `m4/m5` builtin wasm hash 校验（`scripts/ci-tests.sh` 已移除 `sync-m4/m5 --check`）。

结论：
- `required/full` 是“核心链路测试层”的主入口（required 含 `agent_world + consensus + distfs + viewer`，full 追加 `node + net/libp2p`）；
- `required-gate` 已补充 viewer 视觉基线脚本（snapshot 基线 + 定向测试）；
- `builtin-wasm-m1-multi-runner` 与 `builtin-wasm-m4-m5-multi-runner` 共同负责 `m1/m4/m5` hash 链路独立 gate；
- 若目标是“整应用充分测试”，仍需在此基础上叠加 UI 闭环层（S6）与压力层（S8）。

## 分层模型（针对当前仓库）

### L0 静态与工件一致性层
- 目标：尽早拦截格式漂移、内置 wasm 工件漂移、构建目标缺失。
- 性质：最快、最确定。

### L1 核心逻辑确定性层（agent_world 主体）
- 目标：覆盖 runtime/simulator/world-model/LLM 行为/viewer 协议主逻辑。
- 入口：`test_tier_required` 与 `test_tier_full`（主要在 `agent_world` crate）。
- 性质：主覆盖层，应承接绝大多数回归风险。

### L2 协议与联机集成层
- 目标：验证 viewer live、web bridge、离线回放链路、wasmtime 路径等跨模块协作。
- 性质：比 L1 慢，但比 UI 端到端稳定。

### L3 分布式子系统层（node/net/consensus/distfs）
- 目标：验证共识、网络、复制、存储一致性与恢复链路。
- 性质：不应缺席；否则“整应用测试”会有明显盲区。

### L4 UI 闭环层（Web 为默认）
- 目标：验证真实用户路径可用性（加载、交互、状态可见、无 console error）。
- 默认：`world_game_launcher + Playwright`。
- native 抓图：仅 fallback（Web 无法复现或 native 图形链路问题）。

### L5 长稳与压力层
- 目标：验证在长时运行/高事件量下系统退化策略和稳定性。
- 入口：`viewer-owr4-stress.sh`、`llm-longrun-stress.sh`。

## 测试套件目录（S0~S10）

### S0：基础门禁套件（L0）
```bash
./scripts/doc-governance-check.sh
env -u RUSTC_WRAPPER cargo fmt --all -- --check
env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown
```
- 可选（按需执行 builtin wasm hash 校验）：
```bash
./scripts/sync-m1-builtin-wasm-artifacts.sh --check
./scripts/sync-m4-builtin-wasm-artifacts.sh --check
./scripts/sync-m5-builtin-wasm-artifacts.sh --check
```
- 本地策略（2026-03-06 起）：
  - 本地仅允许 `--check`；manifest/identity 写入由 CI bot 流程执行。
  - 非 `--check` 写入需同时满足 `CI=true` 且 `AGENT_WORLD_WASM_SYNC_WRITE_ALLOW=ci-bot`。

### S1：核心 required 套件（L1）
```bash
./scripts/ci-tests.sh required
```
- 覆盖重点：
  - runtime/simulator 大量单元与集成测试
  - `world_viewer_live` 二进制测试
  - viewer offline integration
  - 分布式基础子系统（轻量）：`agent_world_consensus`、`agent_world_distfs`
  - `agent_world_viewer` 全量单测 + wasm 编译检查

### S2：核心 full 套件（L1 + L2）
```bash
./scripts/ci-tests.sh full
```
- 相对 S1 增量：
  - `test_tier_full`
  - `wasmtime` 路径
  - `viewer_live_integration`
  - `agent_world_node --lib`、`agent_world_net --lib`
  - `agent_world_net` 的 `libp2p` 路径
  - `llm-baseline-fixture-smoke`（基线加载与离线治理续跑断言）

### S3：应用主链定向套件（L1 + L2）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required simulator::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required viewer::live::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required viewer::web_bridge::tests:: -- --nocapture
```
- 电价/市场机制定向回归（required/full）：
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required simulator::tests::power::power_buy_zero_price_ -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required simulator::tests::power::power_order_ -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full simulator::tests::power:: -- --nocapture
```
- 主链 Token / NodePoints 桥接定向回归（required/full）：
```bash
./scripts/main-token-regression.sh required
./scripts/main-token-regression.sh full
```
- 运行与审计口径补充：
  - 设计与运行要点：`doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
  - 发布说明：`doc/p2p/token/mainchain-token-allocation-mechanism.release.md`
- 用途：
  - 快速定位 `agent_world` 内部模块回归，不必每次跑全套 full。

### S4：分布式子系统套件（L3）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world_node
env -u RUSTC_WRAPPER cargo test -p agent_world_distfs
env -u RUSTC_WRAPPER cargo test -p agent_world_consensus
env -u RUSTC_WRAPPER cargo test -p agent_world_net --lib
env -u RUSTC_WRAPPER cargo test -p agent_world_net --features libp2p --lib
```
- 可选增强（涉及 runtime_bridge 改动时）：
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world_net --features runtime_bridge --lib
```

### S5：Viewer crate 单测与 wasm 编译套件（L4 前置）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world_viewer
env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown
```
- 说明：
  - `agent_world_viewer` 内已有大量 UI/相机/交互逻辑测试；
  - 这是 UI 闭环前的稳定性筛网；
  - 该套件已并入 `S1/S2` 的默认 gate。

### S6：Web UI 闭环 smoke 套件（L4）
- S6 详细执行步骤、Playwright 命令、发布门禁与补充约定已拆分到：
  - `doc/testing/manual/web-ui-playwright-closure-manual.prd.md`
- 本手册仅保留分层与触发矩阵，执行时按上述文档操作。
- 防误用约束：
  - `scripts/run-game-test-ab.sh` 仅用于自动化回归哨兵（TTFC/命中率/无进展窗口），不等价于“真实玩家长玩评测”。
  - 发布前结论仍需补充手动长玩与卡片填写（按 `doc/playability_test_result/game-test.prd.md` 执行）。
  - 对外样张链路需使用 strict 语义门禁，不得以 `off` / `soft` 结果作为发布判定证据。
- 快速入口：
```bash
./scripts/viewer-release-qa-loop.sh
./scripts/viewer-release-full-coverage.sh --quick
./scripts/viewer-release-art-baseline.sh
```

### S7：场景矩阵回归套件（L1 + L4）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required scenario_specs_match_ids -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required scenarios_are_stable -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full world_init_demo_runs_ -- --nocapture
```
- 配套文档：`doc/world-simulator/scenario/scenario-files.prd.md` 的“场景测试覆盖矩阵”。

### S8：长稳与压力套件（L5）
- Viewer 压测：
```bash
./scripts/viewer-owr4-stress.sh --duration-secs 45 --scenarios triad_region_bootstrap,llm_bootstrap
```
- LLM 长稳：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240
```
- LLM 覆盖门禁（发行口径）：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --release-gate --release-gate-profile hybrid
```
- LLM gameplay 对照（bridge 开/关）：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --prompt-pack story_balanced --runtime-gameplay-bridge
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --prompt-pack story_balanced --no-runtime-gameplay-bridge
```
- git 跟踪基线 fixture smoke（`test_tier_full`）：
```bash
./scripts/llm-baseline-fixture-smoke.sh
```
- Prompt 切换覆盖对比（定向排障）：
```bash
./scripts/llm-switch-coverage-diff.sh --log <run.log> --switch-tick 24
```
- 说明：
  - 详细参数与 profile 组合请以 `./scripts/llm-longrun-stress.sh --help` 为准；
  - `viewer-owr4-stress` 在无 `OPENAI_API_KEY` 时，`llm_bootstrap` 会退化为 script_fallback；
  - `scripts/ci-tests.sh full` 已接入 `./scripts/llm-baseline-fixture-smoke.sh`；
  - 压测结果需保留 CSV/summary/log 产物。

### S9：P2P/存储/共识在线长跑套件（L5）
- 当前状态（2026-02-28）：`scripts/p2p-longrun-soak.sh` 已恢复为可执行脚本，底座为多进程 `world_chain_runtime`。
- 建议命令（smoke）：
```bash
./scripts/p2p-longrun-soak.sh --profile soak_smoke --topologies triad --duration-secs 600 --no-prewarm
```
- 建议命令（endurance + chaos）：
```bash
./scripts/p2p-longrun-soak.sh --profile soak_endurance --topologies triad_distributed --chaos-continuous-enable --chaos-continuous-interval-secs 30 --chaos-continuous-max-events 60
```
- 建议命令（endurance + chaos + feedback）：
```bash
./scripts/p2p-longrun-soak.sh --profile soak_endurance --topologies triad_distributed --duration-secs 900 --chaos-continuous-enable --chaos-continuous-interval-secs 30 --chaos-continuous-max-events 30 --feedback-events-enable --feedback-events-start-sec 30 --feedback-events-interval-secs 60 --feedback-events-max-events 12
```
- 发布门禁基线命令（2026-02-28，300s）：
```bash
./scripts/p2p-longrun-soak.sh --profile soak_release --topologies triad_distributed --duration-secs 300 --no-prewarm --max-stall-secs 240 --max-lag-p95 50 --max-distfs-failure-ratio 0.1 --chaos-continuous-enable --chaos-continuous-interval-secs 30 --chaos-continuous-start-sec 30 --chaos-continuous-max-events 8 --chaos-continuous-actions restart,pause --chaos-continuous-seed 1772284566 --chaos-continuous-restart-down-secs 1 --chaos-continuous-pause-duration-secs 2 --out-dir .tmp/release_gate_p2p
```
- 通过标准：
  - 命令返回 `rc=0`；
  - `summary.json` 中 `overall_status == "ok"` 且 `totals.topology_failed_count == 0`；
  - `soak_release` 档位下 `topologies[].metric_gate.status` 必须为 `pass`（`insufficient_data` 会转失败）；
  - `topologies[].metrics.consensus_hash_consistent` 必须为 `true`，且 `consensus_hash_mismatch_count == 0`（若失败需检查 `topology/.consensus_hash_mismatch.tsv`）；
  - 如启用 chaos，`chaos_events.log` 与 `summary.json.totals.chaos_events_total` 一致。
  - 如启用 feedback events，`summary.json.totals.feedback_events_total == summary.json.totals.feedback_events_success_total + summary.json.totals.feedback_events_failed_total`，且 `feedback_events.log` 中 `phase=completed/failed` 事件数量与 `feedback_events_total` 一致。
- 漂移定位/回滚演练门禁（TASK-GAME-014）：
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::persistence::rollback_with_reconciliation_recovers_from_detected_tick_consensus_drift -- --nocapture
```
- 演练通过标准：
  - 能定位 `mismatch_tick`；
  - `rollback_to_snapshot_with_reconciliation` 后 `first_tick_consensus_drift() == None`；
  - `verify_tick_consensus_chain()` 通过。
- 参考文档：`doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md`、`doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md`。
- 反作弊/反女巫证据链门禁（TASK-GAME-015）：
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::governance::governance_identity_penalty_ -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required governance_identity_penalty_and_appeal_drive_vote_rights -- --nocapture
```
- 通过标准：
  - 同目标主体 + 同证据哈希的惩罚重放被拒绝（incident 指纹不重复通过）。
  - 惩罚 -> 申诉 -> 复核后 `evidence_chain_hash` 逐阶段变化且 `appeal_evidence_hash/resolution_evidence_hash` 非空。
  - `governance_identity_penalty_monitor_stats` 输出误伤率与高风险未闭环数量。

### S10：五节点真实游戏数据在线长跑套件（L5）
- 当前状态（2026-02-28）：`scripts/s10-five-node-game-soak.sh` 已恢复为可执行脚本，底座为五进程 `world_chain_runtime`。
- 当前状态补充（2026-03-01）：reward worker 在空存储时会自动写入 distfs probe seed blob，发布基线下 `distfs_total_checks` 应为正数。
- 建议命令（smoke）：
```bash
./scripts/s10-five-node-game-soak.sh --duration-secs 600 --no-prewarm
```
- 建议命令（默认长窗）：
```bash
./scripts/s10-five-node-game-soak.sh
```
- 发布门禁基线命令（2026-02-28，300s）：
```bash
./scripts/s10-five-node-game-soak.sh --duration-secs 300 --no-prewarm --max-stall-secs 240 --max-lag-p95 50 --out-dir .tmp/release_gate_s10
```
- 通过标准：
  - 命令返回 `rc=0`；
  - `summary.json` 中 `run.status == "ok"`，并产出 `timeline.csv`；
  - `summary.json` 中 `run.metric_gate.status == "pass"`（一般告警通过 `run.metric_gate.notes` 留痕，不应降级为 `insufficient_data`）；
  - 若失败，必须保留 `failures.md` 作为分诊依据。
- 参考文档：`doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md`、`doc/testing/longrun/s10-five-node-real-game-soak.prd.md`。

### 发布门禁一键收口（S0 + S1 + S6 + S9 + S10）
```bash
./scripts/release-gate.sh
./scripts/release-gate.sh --quick
./scripts/release-gate.sh --dry-run
```
- 默认串行执行：`ci-tests full`、`sync-m1/m4/m5 --check`、Web strict、S9/S10。
- `--quick` 用于缩短 S9/S10 时长并关闭 Web visual baseline。
- `--dry-run` 用于门禁编排冒烟，不执行真实命令。

## 改动路径 -> 必跑套件矩阵（针对性执行）

| 改动路径 | 必跑 | 推荐追加 |
|---|---|---|
| `crates/agent_world/src/runtime/**` | S0 + S1 | S2 + S3 + S7 |
| `crates/agent_world/src/simulator/**` | S0 + S1 | S2 + S3 + S7 + S8 |
| `crates/agent_world/src/viewer/**` 或 `src/bin/world_viewer_live/**` | S0 + S1 + S6 | S2 + S3 + S5 |
| `crates/agent_world_viewer/**` | S0 + S5 + S6 | S2 + S8 |
| `crates/agent_world_node/**` | S0 + S4（node） + S9/S10（按改动面至少一条） | S2 + S3 + S8 + 另一条在线长跑（S9 或 S10） |
| `crates/agent_world_net/**` | S0 + S4（net） + S9/S10（按改动面至少一条） | S2 + runtime_bridge 变体 + S8 + 另一条在线长跑（S9 或 S10） |
| `crates/agent_world_consensus/**` | S0 + S4（consensus） + S9/S10（按改动面至少一条） | S2 + S8 + 另一条在线长跑（S9 或 S10） |
| `crates/agent_world_distfs/**` | S0 + S4（distfs） + S9/S10（按改动面至少一条） | S2 + S8 + 另一条在线长跑（S9 或 S10） |
| `scripts/ci-tests.sh` / `.github/workflows/rust.yml` | S0（含 `./scripts/doc-governance-check.sh`） + S1 + `./scripts/viewer-visual-baseline.sh` + （full）`./scripts/llm-baseline-fixture-smoke.sh` | S2 + S4 + S6（抽样） |
| `scripts/release-gate.sh` / `.github/workflows/release-packages.yml` | `./scripts/ci-tests.sh full` + `sync-m1/m4/m5 --check` + Web strict + S9 + S10 | `./scripts/release-gate.sh --quick` / `--dry-run` |
| `scripts/ci-m1-wasm-summary.sh` / `scripts/ci-verify-m1-wasm-summaries.py` / `.github/workflows/builtin-wasm-m1-multi-runner.yml` / `.github/workflows/builtin-wasm-m4-m5-multi-runner.yml` | `S0` + `./scripts/ci-m1-wasm-summary.sh --module-set m4 --runner-label darwin-arm64 --out output/ci/m4-wasm-summary/darwin-arm64.json` + `./scripts/ci-verify-m1-wasm-summaries.py --module-set m4 --summary-dir output/ci/m4-wasm-summary --expected-runners darwin-arm64` | `workflow_dispatch` 触发 m1/m4/m5 双 runner（`linux-x86_64,darwin-arm64`）对账 |
| `scripts/run-viewer-web.sh` / `scripts/capture-viewer-frame.sh` | S0 + S6 | S5 + S8 |
| `scripts/p2p-longrun-soak.sh` / `doc/testing/p2p-storage-consensus-longrun-online-stability-2026-02-24*` | S0 + S9 smoke（含 summary/timeline 校验） | S9 endurance（含 chaos） |
| `scripts/s10-five-node-game-soak.sh` / `doc/testing/s10-five-node-real-game-soak*` | S0 + S10 smoke（含 summary/timeline 校验） | S10 默认长窗（30min+） |

## Human/AI 共用执行剧本

### 阶段 A：确定测试范围
1. 识别改动路径命中哪一行“矩阵”。
2. 生成本次要跑的套件列表（至少含“必跑”列）。
3. 在日志中写清“为什么跑这些、不跑哪些”。

### 阶段 B：先跑低层，后跑高层
1. 先执行 S0。
2. 再执行对应的 L1/L2/L3 套件（S1/S2/S3/S4/S5）。
3. 最后执行 UI 闭环与压力（S6/S8；分布式改动需补 S9 或 S10）。
4. 任意层失败立即停止上层，先定位并修复。

### 阶段 C：记录结论
1. 对每个套件记录：命令、结果、失败点、是否复跑。
2. 记录证据路径（截图、console、CSV、关键日志）。
3. 给出“是否达到本次任务充分度标准”的结论。

## 充分度标准（按任务风险分级）

### 日常改动（低风险）
- 必须通过：S0 + S1
- 若触达 Viewer/UI：追加 S6

### 功能改动（中风险）
- 必须通过：S0 + S1 + 对应路径必跑矩阵
- 至少 1 条 S6 Web 闭环 smoke

### 高风险改动（协议/共识/分布式/发布前）
- 必须通过：S0 + S2 + S4 + S6
- 建议通过：S8 至少一条压力脚本；并执行至少一条 S9 或 S10 在线长跑。

## 证据规范

### 必备证据
- 命令执行记录（终端或 CI 日志）。
- 失败堆栈或关键断言信息。
- UI 闭环截图与 console 结果（若执行 S6）。

### 推荐证据目录
- `output/playwright/viewer/*.png`
- `.playwright-cli/console-*.log`
- `.tmp/viewer_owr4_stress/<timestamp>/`
- `.tmp/llm_stress/`
- `.tmp/p2p_longrun/<timestamp>/`
- `.tmp/s10_game_longrun/<timestamp>/`

### 结果记录模板
```md
- 目标变更：
- 触发路径：
- 执行者（Human/AI）：
- 套件清单（S0~S10）：
  - Sx: 命令 / 结果 / 证据路径
- 失败分诊：
  - 层级（L0~L5）：
  - 原因分类（确定性/环境/flaky）：
  - 处理结论：
- 最终结论：
- 遗留事项：
```

## 失败分诊（按层）
1. L0 失败：优先修复格式、工件、目标安装问题。
2. L1 失败：优先定位业务逻辑回归或断言漂移。
3. L2 失败：优先检查协议兼容、连接时序、桥接参数。
4. L3 失败：优先检查分布式状态恢复、签名校验、网络行为。
5. L4 失败：先判定是否环境问题（端口、launcher 进程、wasm 初始化），再判定 UI 回归。
6. L5 失败：判定是否性能退化、资源泄漏、长时状态累计问题。

## TODO（待收口）
- [x] TODO-1：修正 S7 场景矩阵回归命令的覆盖口径。
  - 处理结果（2026-03-05）：S7 的 `world_init_demo_runs_` 已切换到 `test_tier_full` 执行档位。
  - 验收记录：`env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full world_init_demo_runs_ -- --nocapture` 命中多场景用例（非 1 条）。

- [x] TODO-2：修复 S5 `agent_world_viewer` 测试编译阻塞。
  - 处理结果（2026-02-21）：`agent_world_viewer` 测试集已恢复可编译可执行，并已纳入 `scripts/ci-tests.sh` 的 `required/full` 默认 gate。
  - 验收记录：`env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过，且 `required-gate` 增加 `./scripts/viewer-visual-baseline.sh`。

## 风险
- 风险 1：把 `required/full` 当作整应用全覆盖。
  - 缓解：按本手册补齐 S4/S5/S6/S8。
- 风险 2：UI 闭环只看截图，不看状态与 console。
  - 缓解：S6 强制 `console error = 0` + 可见状态判断。
- 风险 3：分布式子系统改动未触发对应 crate 测试。
  - 缓解：必须使用“改动路径矩阵”决策套件。
- 风险 4：压力回归长期缺失，问题只在长跑暴露。
  - 缓解：高风险改动或发布前至少执行一条 S8，并执行一条 S9 或 S10 在线长跑。

## 里程碑
- T1：完成基于仓库现状的分层模型与套件目录。
- T2：完成改动路径触发矩阵与 Human/AI 共用剧本。
- T3：完成充分度标准、证据规范、失败分诊规则。
- T4：后续按真实缺陷复盘持续调整各层用例配额与命令清单。
