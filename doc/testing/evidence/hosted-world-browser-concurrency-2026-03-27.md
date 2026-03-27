# hosted world 浏览器并发接入证据（2026-03-27）

审计轮次: 1

## Meta
- 关联专题: `PRD-P2P-023-B/C/E`
- 关联任务: `TASK-P2P-041-B/C/E`
- 责任角色: `qa_engineer`
- 协作角色: `viewer_engineer`
- 当前结论: `pass`
- 目标: 固化 hosted public join 在真实双浏览器会话下的 `web_bind` 并发接入真值，确认第二个页面不再长期停在 `debug_viewer:detached`。

## 最终结论
- `ViewerWebBridge::run()` 的并发 accept 改动已经在真实 hosted 栈生效。
- 同一 `oasis7_game_launcher --deployment-mode hosted_public_join --chain-disable` 栈下，两份独立 `agent-browser` session 同时加载同一个 `web_bind` 时，都能稳定进入:
  - `software_safe`
  - `connected`
  - `debugViewer=debug_viewer:subscribed`
  - `controlProfile=live`
- 两个页面都能同时看到 5 个 seeded agents 与完整 snapshot，不再复现“第一页正常、第二页长期 detached/空 snapshot”的 backlog 症状。
- 本次验证额外确认了一个执行约束:
  - `cargo run -p oasis7 --bin oasis7_game_launcher` 不会自动重编它运行的 sibling child binary `target/debug/oasis7_viewer_live`
  - 若要验证 `web_bridge.rs` 这类只影响 child binary 的改动，必须先显式执行 `env -u RUSTC_WRAPPER cargo build -p oasis7 --bin oasis7_viewer_live`，否则本地真实验证会继续跑旧版桥接逻辑

## 执行命令
- 显式重编 child binary:
  - `env -u RUSTC_WRAPPER cargo build -p oasis7 --bin oasis7_viewer_live`
- 定向回归:
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 --lib bridge_run_accepts_second_websocket_while_first_stays_open -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 --lib runtime_live_run_accepts_probe_while_viewer_session_is_open -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_viewer_live -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_game_launcher -- --nocapture`
- 本地 hosted 栈:
  - `env OASIS7_HOSTED_STRONG_AUTH_PUBLIC_KEY=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa OASIS7_HOSTED_STRONG_AUTH_PRIVATE_KEY=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb OASIS7_HOSTED_STRONG_AUTH_APPROVAL_CODE=preview-code env -u RUSTC_WRAPPER cargo run -q -p oasis7 --bin oasis7_game_launcher -- --deployment-mode hosted_public_join --viewer-static-dir crates/oasis7_viewer --viewer-host 127.0.0.1 --viewer-port 6101 --web-bind 127.0.0.1:6102 --live-bind 127.0.0.1:6103 --chain-disable --no-open-browser`
- 双浏览器会话:
  - `agent-browser --session hosted-concurrent-c open 'http://127.0.0.1:6101/?ws=ws://127.0.0.1:6102'`
  - `agent-browser --session hosted-concurrent-d open 'http://127.0.0.1:6101/?ws=ws://127.0.0.1:6102'`
  - `agent-browser --session hosted-concurrent-c get text body`
  - `agent-browser --session hosted-concurrent-d get text body`

## 浏览器证据
### 1. Session C
- 页面文本包含:
  - `software_safe`
  - `connected`
  - `debugViewer=debug_viewer:subscribed`
  - `controlProfile=live`
  - `agent-0` ~ `agent-4`
- `Snapshot Summary.counts` 返回:
  - `agents=5`
  - `locations=5`

### 2. Session D
- 页面文本同样包含:
  - `software_safe`
  - `connected`
  - `debugViewer=debug_viewer:subscribed`
  - `controlProfile=live`
  - `agent-0` ~ `agent-4`
- `Snapshot Summary.counts` 同样返回:
  - `agents=5`
  - `locations=5`

### 3. socket 侧佐证
- 双页加载后，`ss -tnp state established '( sport = :6102 or dport = :6102 )'` 显示 `oasis7_viewer_live` 在 `127.0.0.1:6102` 上同时持有多条已 accept 的 browser 连接，而不是像旧版那样只 accept 第一条、把第二条留在 backlog。

## 风险与剩余项
- 本证据只关闭了 `web_bind` 并发接入瓶颈，不代表 hosted player-session / strong-auth 全链路已经完成。
- 当前剩余主 blocker 没有回到 transport 层，而是产品链路证据层:
  - 还缺一份“真实 attached player session + backend strong-auth grant + prompt_control success”完整证据
  - `main_token_transfer` 继续保持 `blocked_until_strong_auth`
