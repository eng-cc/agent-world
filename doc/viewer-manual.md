# Agent World Viewer 使用说明书

## 目标
- 提供一份可直接操作的 Viewer 使用手册，覆盖启动、交互、自动聚焦与 Web 闭环。
- 统一人工调试与脚本闭环的命令入口，减少重复沟通成本。

## 适用范围
- 可视化客户端：`crates/agent_world_viewer`
- 联调服务端：`crates/agent_world --bin world_viewer_live`
- Web 闭环入口：`scripts/run-viewer-web.sh` + Playwright CLI
- native fallback 脚本：`scripts/capture-viewer-frame.sh`

## 快速开始

### 1）启动 live server（推荐）
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --llm --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300
```
如需让 Agent 决策走 LLM，可额外加 `--llm`（需要先配置 LLM key）。

### 2）启动 viewer
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

### 3）离线模式（仅查看本地 UI，不连服务端）
```bash
AGENT_WORLD_VIEWER_OFFLINE=1 env -u RUSTC_WRAPPER cargo run -p agent_world_viewer
```

### 4）浏览器模式（Bevy + wasm）
```bash
env -u NO_COLOR ./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173
```

- 打开浏览器访问：`http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011`
- Web 端通过 `world_viewer_live --web-bind` 提供的 WebSocket bridge 在线连接 live server。
- 首次运行前需安装：
  - `trunk`（`cargo install trunk`）
  - `wasm32-unknown-unknown`（`rustup target add wasm32-unknown-unknown`）

## 常用交互
- 鼠标拖拽：旋转/平移观察视角。
- 滚轮：缩放。
- `W/A/S/D`：移动相机视角（平移 `focus`，2D/3D 均可用；仅在光标位于 3D 视口且未占用文本输入时生效）。
- 右侧面板：查看状态、事件、分块、诊断等信息。
- `2D/3D` 切换：在顶部按钮切换视角模式。
- `F`：对“当前选中对象”执行聚焦（适合人工巡检细节）。

## 自动聚焦（Auto Focus）

### 启动时自动聚焦（环境变量）
- `AGENT_WORLD_VIEWER_AUTO_FOCUS=1`
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_TARGET=<target>`
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_FORCE_3D=1|0`（默认 `1`）
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_RADIUS=<number>`（可选）

支持目标：
- `first_fragment`
- `first_location`
- `first_agent`
- `location:<id>`
- `agent:<id>`

示例：
```bash
AGENT_WORLD_VIEWER_AUTO_FOCUS=1 \
AGENT_WORLD_VIEWER_AUTO_FOCUS_TARGET=first_fragment \
AGENT_WORLD_VIEWER_AUTO_FOCUS_RADIUS=18 \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

## Web 闭环（默认，推荐调试/回归）

### 前置要求
- Node.js 20+（需 `npx` 可用）
- `trunk`（`cargo install trunk`）
- `wasm32-unknown-unknown`（`rustup target add wasm32-unknown-unknown`）

### 标准流程
1) 启动 live server（终端 A）
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300
```

2) 启动 Web Viewer（终端 B）
```bash
env -u NO_COLOR ./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173
```

3) 执行 Playwright 闭环采样（终端 C）
```bash
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export PWCLI="$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh"
mkdir -p output/playwright/viewer
bash "$PWCLI" open "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011"
bash "$PWCLI" snapshot
bash "$PWCLI" console
bash "$PWCLI" screenshot --filename output/playwright/viewer/viewer-web.png
bash "$PWCLI" close
```

### 输出目录
- `output/playwright/viewer/*.png`
- `.playwright-cli/console-*.log`（或 CLI 控制台输出）

### 最小验收口径
- 页面加载成功（`snapshot` 可见 `canvas`）。
- `console error = 0`。
- 至少产出 1 张截图。

### native fallback（仅在 Web 无法复现或排查图形链路）
```bash
./scripts/capture-viewer-frame.sh --scenario asteroid_fragment_detail_bootstrap --addr 127.0.0.1:5131 --tick-ms 300 --viewer-wait 12 --auto-focus-target first_fragment --auto-focus-radius 18
```

## 推荐调试场景
- 细粒度 location 渲染观察：`asteroid_fragment_detail_bootstrap`
- 常规联调：`llm_bootstrap`
- 双区域对比：`twin_region_bootstrap`

## 开采损耗可视化
- 当 location 含有 `fragment_budget` 时，Viewer 会按剩余质量比例缩放体量（体积比例映射到半径立方根）。
- 剩余越少，location 视觉半径越小；为避免完全不可见，存在最小可视半径保护。
- 详情面板会显示：`Fragment Depletion: mined=<x>% remaining=<a>/<b>`。

## 常见问题排查
- Web 页面空白：等待 `trunk` 首轮编译完成，确认访问端口与 `run-viewer-web.sh` 参数一致。
- Playwright 启动失败：先检查 `node --version`、`npm --version`、`npx` 是否可用。
- Console 有 wasm 报错：先修复运行时错误再看视觉问题，避免误判为渲染缺陷。
- 看不到细节：切换 3D，放大并移动视角；必要时使用 `F` 聚焦目标。
- 自动聚焦无效：确认 target 存在，或先使用 `first_fragment` 排除 ID 输入问题。
- 连接失败：检查 `world_viewer_live` 是否运行、端口与 viewer 地址是否一致。

## 参考文档
- `doc/world-simulator/viewer-location-fine-grained-rendering.md`
- `doc/world-simulator/viewer-auto-focus-capture.md`
- `doc/world-simulator/viewer-web-closure-testing-policy.md`
- `doc/scripts/capture-viewer-frame.md`（native fallback）

## Fragment 元素分块渲染（默认开启）
- 目标：把 location 的 fragment 分块默认显示出来，并按主导元素显示不同颜色。
- 当前行为：不再渲染 location 外层几何与标签，仅保留逻辑锚点；frag 分块始终渲染。
- 选择交互：点击 frag 后，详情面板会显示所属 `location`（ID 与名称）。
- 配置说明：已移除 frag 渲染开关与对应环境变量，不再支持按开关隐藏 frag。
