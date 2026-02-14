# Agent World: 测试覆盖与 CI 扩展（设计文档）

## 目标
- 补齐关键链路测试覆盖：离线回放 viewer server 联测、wasmtime 特性测试。
- 在 CI 中显式运行关键测试目标，确保特性分支可持续验证。
- 保持测试稳定性：独立端到端联测不依赖 UI 与网络外部资源。
- 保障 `egui` snapshot 测试在无可用 `wgpu` 设备的 CI 机器上可降级运行，不阻断整体测试流水线。

## 范围

### In Scope
- Viewer 离线回放端到端联测（snapshot/journal -> server -> client）。
- Viewer 在线联测维持现有 feature gate（viewer_live_integration）。
- CI 增加 `--features wasmtime` 测试步骤。
- CI 运行离线回放联测目标（viewer_offline_integration）。
- 文档补充联测运行方式与 CI 覆盖说明。
- `agent_world_viewer` 的 `egui_kittest` snapshot 测试在 `wgpu` 初始化失败时自动跳过（记录原因）。

### Out of Scope
- CI 的缓存优化与并行矩阵策略。
- UI 真实渲染测试（使用 headless 协议联测即可）。

## 接口 / 数据

### 联测输入
- `world_viewer_demo` 生成的 `snapshot.json` 与 `journal.json`。

### 联测输出
- 验证 `HelloAck` / `Snapshot` / `Event` 消息能被正常接收。

### CI 任务
- 统一测试清单脚本：`scripts/ci-tests.sh`（分级参数：`required` / `full`）。
- 本地与 PR 门禁：`CI_VERBOSE=1 ./scripts/ci-tests.sh required`。
- 每日定时全量：`CI_VERBOSE=1 ./scripts/ci-tests.sh full`。
- `required` 覆盖：
  - `env -u RUSTC_WRAPPER cargo fmt --all -- --check`
  - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required --test module_input_cbor --test module_lifecycle --test module_state --test module_store --test module_subscription_filters --test viewer_offline_integration --test world_init_demo`
- `full` 追加覆盖：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full --test module_input_cbor --test module_lifecycle --test module_state --test module_store --test module_subscription_filters --test viewer_offline_integration --test world_init_demo`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full,wasmtime --test wasm_executor`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full,viewer_live_integration --test viewer_live_integration`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features wasmtime --lib --bins`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_net --features libp2p --lib`
- CI 需安装系统依赖（Wayland/X11/ALSA/UDev）以编译 `agent_world_viewer`。
- `egui` snapshot 测试在渲染器初始化失败时输出 skip 日志并提前返回，不影响其他 `cargo test` 用例。

## 里程碑
- **T1**：新增离线回放联测并保证稳定退出。
- **T2**：CI 增加 wasmtime 特性测试步骤。
- **T3**：文档更新与任务日志落地。
- **T4**：`egui` snapshot 测试支持无 `wgpu` 环境降级。

## 风险
- wasmtime 特性测试耗时与依赖体积增加。
- socket 联测对端口竞争敏感，需要随机端口与超时保护。
- `egui` snapshot 在无 `wgpu` 时会被跳过，可能降低该环境下的视觉回归覆盖；需在具备图形能力环境保留一次完整快照校验。
