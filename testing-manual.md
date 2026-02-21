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
  - Web 启动入口：`scripts/run-viewer-web.sh`
  - Web 闭环采样：Playwright CLI（`AGENTS.md` 标准流程）

### 分布式与共识子系统
- Node：`crates/agent_world_node`
- Net：`crates/agent_world_net`
- Consensus：`crates/agent_world_consensus`
- DistFS：`crates/agent_world_distfs`
- 这些子系统有独立测试集，但当前 `scripts/ci-tests.sh` 只覆盖了其中一部分（见下文“CI 现状与缺口”）。

### 场景系统
- 场景定义：`crates/agent_world/src/simulator/scenario.rs`
- 场景矩阵设计：`doc/world-simulator/scenario-files.md`
- 场景是 UI 闭环、协议闭环、压力回归的统一输入源。

## CI 现状与缺口（事实口径）

### 当前 CI/脚本已覆盖
- 入口 A：`scripts/ci-tests.sh`（主流程）
- `required`：
  - `cargo fmt --check`
  - `cargo test -p agent_world --tests --features test_tier_required`
  - `cargo test -p agent_world_viewer`
  - `cargo check -p agent_world_viewer --target wasm32-unknown-unknown`
- `full`：
  - `required` 全部
  - `cargo test -p agent_world --tests --features test_tier_full,wasmtime,viewer_live_integration`
  - `cargo test -p agent_world --features wasmtime --lib --bins`
  - `cargo test -p agent_world_net --features libp2p --lib`
- 入口 B：`.github/workflows/rust.yml`（required-gate）
  - `CI_VERBOSE=1 ./scripts/ci-tests.sh required`
  - `./scripts/viewer-visual-baseline.sh`
- 入口 C：`.github/workflows/builtin-wasm-m1-multi-runner.yml`（构建 hash 链路独立 gate）
  - runner 矩阵：`ubuntu-24.04 (linux-x86_64)` + `macos-14 (darwin-arm64)`
  - 每个 runner 仅执行：`./scripts/ci-m1-wasm-summary.sh --runner-label ... --out ...`
  - 汇总 job 对账：`./scripts/ci-verify-m1-wasm-summaries.py --summary-dir ... --expected-runners linux-x86_64,darwin-arm64`

### 当前 CI 未直接覆盖（需手册补齐）
- `agent_world_node` 独立测试集。
- `agent_world_consensus` 独立测试集。
- `agent_world_distfs` 独立测试集。
- Web UI Playwright 闭环（现为手动/agent 流程，不在 CI 默认路径中）。
- `m4/m5` builtin wasm hash 校验（`scripts/ci-tests.sh` 已移除 `sync-m4/m5 --check`）。

结论：
- `required/full` 是“核心链路测试层”的主入口（已包含 `agent_world_viewer` 单测 + wasm check）；
- `required-gate` 已补充 viewer 视觉基线脚本（snapshot 基线 + 定向测试）；
- `builtin-wasm-m1-multi-runner` 负责 `m1` hash 链路独立 gate；
- 若目标是“整应用充分测试”，必须在其上叠加分布式子系统与 UI 闭环层。

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
- 默认：`run-viewer-web.sh + Playwright`。
- native 抓图：仅 fallback（Web 无法复现或 native 图形链路问题）。

### L5 长稳与压力层
- 目标：验证在长时运行/高事件量下系统退化策略和稳定性。
- 入口：`viewer-owr4-stress.sh`、`llm-longrun-stress.sh`。

## 测试套件目录（S0~S8）

### S0：基础门禁套件（L0）
```bash
env -u RUSTC_WRAPPER cargo fmt --all -- --check
env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown
```
- 可选（按需执行 builtin wasm hash 校验）：
```bash
./scripts/sync-m1-builtin-wasm-artifacts.sh --check
./scripts/sync-m4-builtin-wasm-artifacts.sh --check
./scripts/sync-m5-builtin-wasm-artifacts.sh --check
```

### S1：核心 required 套件（L1）
```bash
./scripts/ci-tests.sh required
```
- 覆盖重点：
  - runtime/simulator 大量单元与集成测试
  - `world_viewer_live` 二进制测试
  - viewer offline integration
  - `agent_world_viewer` 全量单测 + wasm 编译检查

### S2：核心 full 套件（L1 + L2）
```bash
./scripts/ci-tests.sh full
```
- 相对 S1 增量：
  - `test_tier_full`
  - `wasmtime` 路径
  - `viewer_live_integration`
  - `agent_world_net` 的 `libp2p` 路径

### S3：应用主链定向套件（L1 + L2）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required simulator::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required viewer::live::tests:: -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required viewer::web_bridge::tests:: -- --nocapture
```
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
1) 启动 live server（含 bridge）：
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300
```
2) 启动 web viewer：
```bash
env -u NO_COLOR ./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173
```
3) Playwright 采样：
```bash
source "$HOME/.nvm/nvm.sh"
nvm use 24
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export PWCLI="$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh"
mkdir -p output/playwright/viewer
bash "$PWCLI" open "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011"
bash "$PWCLI" snapshot
bash "$PWCLI" console
bash "$PWCLI" screenshot --filename output/playwright/viewer/viewer-web.png
bash "$PWCLI" close
```
4) 最小通过标准：
- `snapshot` 可见 `canvas`
- `console error = 0`
- 至少 1 张截图在 `output/playwright/viewer/`

#### S6 补充约定（迁移自 `AGENTS.md`）
- 默认链路：
  - Web 闭环为默认，不以 native 抓图链路替代。
- Fallback（仅 native 链路问题）：
  - 当问题只在 native 图形链路出现，或 Web 端无法复现时，再使用：
    - `./scripts/capture-viewer-frame.sh`
  - 该链路定位为历史兼容/应急，不作为默认闭环流程。
- 推荐约定：
  - Web 闭环产物统一放在 `output/playwright/`。
  - 每次调试结束清理 `run-viewer-web.sh` 后台进程，避免端口冲突。
  - 若页面首帧空白，优先排查：
    - `trunk` 是否完成首轮编译。
    - 访问地址是否与脚本端口一致。
    - 浏览器控制台是否有 wasm 初始化错误。

### S7：场景矩阵回归套件（L1 + L4）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required scenario_specs_match_ids -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required scenarios_are_stable -- --nocapture
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required world_init_demo_runs_ -- --nocapture
```
- 配套文档：`doc/world-simulator/scenario-files.md` 的“场景测试覆盖矩阵”。

### S8：长稳与压力套件（L5）
- Viewer 压测：
```bash
./scripts/viewer-owr4-stress.sh --duration-secs 45 --tick-ms 120 --scenarios triad_region_bootstrap,llm_bootstrap
```
- LLM 长稳：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240
```
- LLM 玩法覆盖门禁（按场景声明关键动作）：
```bash
./scripts/llm-longrun-stress.sh \
  --scenario llm_bootstrap \
  --ticks 240 \
  --min-action-kinds 5 \
  --require-action-kind harvest_radiation:1 \
  --require-action-kind mine_compound:1 \
  --require-action-kind refine_compound:1 \
  --require-action-kind build_factory:1 \
  --require-action-kind schedule_recipe:1
```
- LLM 发行口径（启用默认覆盖门禁）：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --release-gate --release-gate-profile hybrid
```
- LLM 发行口径（按 profile 拆分）：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --release-gate --release-gate-profile industrial
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --release-gate --release-gate-profile gameplay
```
- LLM 游戏发展测试 prompt（非强制动作链）：
```bash
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --prompt-pack story_balanced
./scripts/llm-longrun-stress.sh --scenario llm_bootstrap --ticks 240 --prompt-pack civic_operator
```
- 说明：
  - `viewer-owr4-stress` 在无 `OPENAI_API_KEY` 时对 `llm_bootstrap` 会退化为 script_fallback；
  - `llm-longrun-stress.sh` 新增覆盖参数：
    - `--min-action-kinds <n>`：断言动作种类数下限；
    - `--require-action-kind <kind>:<min_count>`：断言关键动作计数下限（可重复）；
    - `--release-gate`：启用发行默认门禁；
    - `--release-gate-profile <industrial|gameplay|hybrid>`：
      - `industrial`：工业闭环 5 动作（`harvest_radiation/mine_compound/refine_compound/build_factory/schedule_recipe`）。
      - `gameplay`：玩法闭环 4 动作（`open_governance_proposal/cast_governance_vote/resolve_crisis/grant_meta_progress`）。
      - `hybrid`（默认）：工业 + gameplay 全覆盖（9 动作）。
    - `--prompt-pack <story_balanced|frontier_builder|civic_operator|resilience_drill>`：
      - `story_balanced`：默认推荐，按“稳定 -> 生产 -> 治理/韧性”阶段推进，并自动注入中途目标切换。
      - `frontier_builder`：偏探索与基础设施扩张。
      - `civic_operator`：偏治理协同与组织秩序。
      - `resilience_drill`：偏危机恢复与经济协作抗压。
  - 若覆盖门禁失败，脚本会输出缺失项与当前 `action_kind_counts`，用于快速定位玩法漏覆盖；
  - 压测结果需保留 CSV/summary/log 产物。

## 改动路径 -> 必跑套件矩阵（针对性执行）

| 改动路径 | 必跑 | 推荐追加 |
|---|---|---|
| `crates/agent_world/src/runtime/**` | S0 + S1 | S2 + S3 + S7 |
| `crates/agent_world/src/simulator/**` | S0 + S1 | S2 + S3 + S7 + S8 |
| `crates/agent_world/src/viewer/**` 或 `src/bin/world_viewer_live/**` | S0 + S1 + S6 | S2 + S3 + S5 |
| `crates/agent_world_viewer/**` | S0 + S5 + S6 | S2 + S8 |
| `crates/agent_world_node/**` | S0 + S4（node） | S2 + S3 |
| `crates/agent_world_net/**` | S0 + S4（net） | S2 + runtime_bridge 变体 |
| `crates/agent_world_consensus/**` | S0 + S4（consensus） | S2 |
| `crates/agent_world_distfs/**` | S0 + S4（distfs） | S2 + S8（若影响长稳） |
| `scripts/ci-tests.sh` / `.github/workflows/rust.yml` | S0 + S1 + `./scripts/viewer-visual-baseline.sh` | S2 + S4 + S6（抽样） |
| `scripts/ci-m1-wasm-summary.sh` / `scripts/ci-verify-m1-wasm-summaries.py` / `.github/workflows/builtin-wasm-m1-multi-runner.yml` | `S0` + `./scripts/ci-m1-wasm-summary.sh --runner-label darwin-arm64 --out output/ci/m1-wasm-summary/darwin-arm64.json` + `./scripts/ci-verify-m1-wasm-summaries.py --summary-dir output/ci/m1-wasm-summary --expected-runners darwin-arm64` | `workflow_dispatch` 触发双 runner（`linux-x86_64,darwin-arm64`）对账 |
| `scripts/run-viewer-web.sh` / `scripts/capture-viewer-frame.sh` | S0 + S6 | S5 + S8 |

## Human/AI 共用执行剧本

### 阶段 A：确定测试范围
1. 识别改动路径命中哪一行“矩阵”。
2. 生成本次要跑的套件列表（至少含“必跑”列）。
3. 在日志中写清“为什么跑这些、不跑哪些”。

### 阶段 B：先跑低层，后跑高层
1. 先执行 S0。
2. 再执行对应的 L1/L2/L3 套件（S1/S2/S3/S4/S5）。
3. 最后执行 UI 闭环与压力（S6/S8）。
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
- 建议通过：S8 至少一条压力脚本

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

### 结果记录模板
```md
- 目标变更：
- 触发路径：
- 执行者（Human/AI）：
- 套件清单（S0~S8）：
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
5. L4 失败：先判定是否环境问题（端口、Node、trunk、wasm 初始化），再判定 UI 回归。
6. L5 失败：判定是否性能退化、资源泄漏、长时状态累计问题。

## TODO（待收口）
- [ ] TODO-1：修正 S7 场景矩阵回归命令的覆盖口径。
  - 问题：当前 `world_init_demo_runs_` 在 `test_tier_required` 下只会命中 1 条用例，无法代表场景矩阵回归充分度。
  - 动作：将 S7 中 `world_init_demo_runs_` 的执行档位调整为 `test_tier_full`（或在文档中明确 required/full 的期望命中数差异与适用场景）。
  - 验收：`env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full world_init_demo_runs_ -- --nocapture` 命中多场景用例（非 1 条）。

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
  - 缓解：高风险改动或发布前至少执行一条 S8。

## 里程碑
- T1：完成基于仓库现状的分层模型与套件目录。
- T2：完成改动路径触发矩阵与 Human/AI 共用剧本。
- T3：完成充分度标准、证据规范、失败分诊规则。
- T4：后续按真实缺陷复盘持续调整各层用例配额与命令清单。
