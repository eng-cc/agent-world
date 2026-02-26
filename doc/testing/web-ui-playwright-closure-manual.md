# Agent World: Web UI Playwright 闭环测试手册

> 本文档从 `testing-manual.md` 的 `S6` 拆分，作为 Web UI 闭环与 Playwright 执行细节的唯一入口。

## 目标
- 统一 Web UI 闭环测试的启动、采样、门禁和排障流程。
- 让 Human/AI 在同一套命令下可复现实机浏览器结果。

## S6：Web UI 闭环 smoke 套件（L4）
1) 启动 live server（含 bridge）：
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300 --topology single --viewer-no-consensus-gate
```
2) 启动 web viewer：
```bash
env -u NO_COLOR ./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173
```
2.1) 启动前自检（必做，防止一次性启动失败）：
```bash
# 1) 确认端口监听（viewer=4173, ws=5011）
lsof -iTCP:4173 -sTCP:LISTEN -n -P
lsof -iTCP:5011 -sTCP:LISTEN -n -P

# 2) 确认主页可访问（避免 ERR_CONNECTION_REFUSED）
curl -fsS "http://127.0.0.1:4173/" >/dev/null

# 3) URL 必须带引号（含 `&`，否则 shell 会截断/拆命令）
URL='http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011&test_api=1'
```
3) Playwright 采样：
```bash
source "$HOME/.nvm/nvm.sh"
nvm use 24
export REPO_ROOT="$(pwd)"
export PWCLI="$REPO_ROOT/.codex/skills/playwright/scripts/playwright_cli.sh"
[ -f "$PWCLI" ] || { echo "missing playwright cli wrapper: $PWCLI" >&2; exit 1; }
mkdir -p output/playwright/viewer
bash "$PWCLI" open "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011&test_api=1" --headed
bash "$PWCLI" snapshot
bash "$PWCLI" eval '() => typeof window.__AW_TEST__ === "object"'
bash "$PWCLI" eval '() => window.__AW_TEST__.runSteps("mode=3d;focus=first_location;zoom=0.85;select=first_agent;wait=0.3")'
bash "$PWCLI" eval '() => window.__AW_TEST__.sendControl("pause")'
bash "$PWCLI" eval '() => window.__AW_TEST__.getState()'
bash "$PWCLI" eval '() => { const s = window.__AW_TEST__.getState(); return !!s && typeof s.tick === "number" && typeof s.connectionStatus === "string"; }'
bash "$PWCLI" console
bash "$PWCLI" screenshot --filename output/playwright/viewer/viewer-web.png
bash "$PWCLI" close
```
3.1) Playwright 会话防抖（推荐）：
```bash
# 每轮前清理旧会话，避免残留 daemon/session 干扰
bash "$PWCLI" close-all || true

# 打开页面后先做 fail-fast 检查
bash "$PWCLI" open "$URL" --headed
bash "$PWCLI" snapshot
bash "$PWCLI" eval '() => typeof window.__AW_TEST__ === "object"'
bash "$PWCLI" console warning
```
4) 最小通过标准：
- `snapshot` 可见 `canvas`
- `window.__AW_TEST__` 可用，且 `getState()` 返回 `tick/connectionStatus`
- `console error = 0`
- 至少 1 张截图在 `output/playwright/viewer/`

## 启动失败分级与处置（Fail Fast）
- F1：`open/goto` 报 `ERR_CONNECTION_REFUSED`
  - 结论：`trunk serve` 尚未就绪或进程已退出。
  - 处置：回看 `run-viewer-web.sh` 输出，确认 4173 监听后再重试 `open`。
- F2：页面可开但 console 出现 `copy_deferred_lighting_id_pipeline`、`RuntimeError: unreachable`、`CONTEXT_LOST_WEBGL`
  - 结论：渲染初始化崩溃（非测试脚本问题）。
  - 处置：立即归档 console/screenshot/video，标记本轮 fail，不继续做可玩性判定。
- F3：`__AW_TEST__` 可用但 `connectionStatus=connecting` 且 `tick=0` 长时间不变
  - 结论：语义链路未进入有效推进（连接或主循环异常）。
  - 处置：执行 `play` 后额外观察约 12s；仍不推进则判 fail 并归档证据。
- F4：脚本/人工流程里把 URL 写入 env 文件后 `source` 报 parse error
  - 结论：`&` 未转义或未加引号。
  - 处置：使用 `URL='http://...?...&...'`，或避免 `source` 这类文件。

## 一键发行验收（推荐）
```bash
./scripts/viewer-release-qa-loop.sh
```
- 产物：
  - `output/playwright/viewer/release-qa-summary-*.md`
  - `output/playwright/viewer/release-qa-*.png`
  - `output/playwright/viewer/release-qa-*.log`
- 门禁要点：
  - 视觉基线（`viewer-visual-baseline.sh`）通过；
  - `window.__AW_TEST__` 语义动作链通过；
  - 多缩放贴图观感门禁通过（near/mid/far 三档截图 + 相机半径语义断言 + 截图像素差异指标）；
  - console dump 扫描 Bevy `%cERROR%c`/`[ERROR]`（避免仅依赖浏览器原生 error 计数漏报）。

## 一键全覆盖发行验收（可用性 + 视觉 + 玩法）
```bash
./scripts/viewer-release-full-coverage.sh
```
- 覆盖范围：
  - Web 可用性门禁（`viewer-release-qa-loop.sh`）；
  - 主题包校验 + 主题变体截图 + 纹理通道矩阵截图；
  - 关键玩法链路门禁：
    - 工业：`harvest_radiation/mine_compound/refine_compound/build_factory/schedule_recipe`
    - 治理危机：`open_governance_proposal/cast_governance_vote/resolve_crisis/grant_meta_progress`
    - 经济：`open/accept/settle_economic_contract`
- 产物：
  - `output/playwright/viewer/release_full/<timestamp>/release-full-summary-*.md`
  - 子目录：`web_qa/`、`theme_preview/`、`texture_inspector/`、`gameplay_industrial/`、`gameplay_governance/`
  - 视觉抓帧状态：`theme_preview/*/capture_status.txt`、`texture_inspector/*/*/capture_status.txt`
- 视觉门禁新增硬条件：
  - 主题/纹理截图不仅要求 `viewer.png` 存在；
  - 还要求 `capture_status.txt` 满足 `connection_status=connected` 且 `snapshot_ready=1`；
  - 任一截图断连或无快照时，full coverage 直接 FAIL（避免“有图但不可用”的假通过）。
- 快速冒烟：
```bash
./scripts/viewer-release-full-coverage.sh --quick
```

## 补充约定（迁移自 `AGENTS.md`）
- 默认链路：
  - Web 闭环为默认，不以 native 抓图链路替代。
- Fallback（仅 native 链路问题）：
  - 当问题只在 native 图形链路出现，或 Web 端无法复现时，再使用：
    - `./scripts/capture-viewer-frame.sh`
  - 该链路定位为历史兼容/应急，不作为默认闭环流程。
- 推荐约定：
  - Web 闭环产物统一放在 `output/playwright/`。
  - Playwright 优先通过 `window.__AW_TEST__`（`runSteps/setMode/focus/select/sendControl/getState`）做语义化操作，避免坐标点击脆弱性。
  - 发布验收优先使用 `./scripts/viewer-release-qa-loop.sh` 固化流程；Web UI 渲染性能口径必须使用 GPU 硬件加速（禁止 `SwiftShader/software rendering`），Playwright 需 `open ... --headed`；LLM 场景需等待首个 tick 推进（建议 `play` 后额外观察约 12s）后再判失败。
  - 每次调试结束清理 `run-viewer-web.sh` 后台进程，避免端口冲突。
  - 若页面首帧空白，优先排查：
    - `trunk` 是否完成首轮编译。
    - 访问地址是否与脚本端口一致。
    - 浏览器控制台是否有 wasm 初始化错误。
