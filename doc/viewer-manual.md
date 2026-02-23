# Agent World Viewer 使用说明书

## 目标
- 提供一份可直接操作的 Viewer 使用手册，覆盖启动、交互、自动聚焦、自动步骤与 Web 闭环。
- 统一人工调试与脚本闭环的命令入口，减少重复沟通成本。

## 适用范围
- 可视化客户端：`crates/agent_world_viewer`
- 联调服务端：`crates/agent_world --bin world_viewer_live`
- Web 闭环入口：`scripts/run-viewer-web.sh` + Playwright CLI
- native fallback 脚本：`scripts/capture-viewer-frame.sh`
- 角色边界：Web 端定位为 Viewer（观察/调试/间接控制），不承担完整分布式节点职责；共识与复制由后端节点进程负责。

## 快速开始

### 1）启动 live server（推荐）
```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300
```
`world_viewer_live` 默认使用 LLM 决策（仍需先配置 LLM key）；`--llm` 可保留用于显式声明。
如需临时回退到内置脚本决策，可显式传 `--no-llm`。

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --no-llm --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300
```

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
- Web 端通过 `world_viewer_live --web-bind` 提供的 WebSocket bridge 在线连接 live server（Viewer + 网关路径）。
- Web 端不直接运行 `agent_world_node` 的完整分布式协议栈（不承担 gossip/replication/共识职责）。
- 首次运行前需安装：
  - `trunk`（`cargo install trunk`）
  - `wasm32-unknown-unknown`（`rustup target add wasm32-unknown-unknown`）

## 发行模式（P2P 推荐）

当节点发布后不希望再通过命令行临时调参时，使用 `--release-config <path>` 启动锁定参数文件：

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- \
  --release-config world_viewer_live.release.example.toml \
  --bind 0.0.0.0:5010 \
  --web-bind 0.0.0.0:5011
```

- `world_viewer_live.release.example.toml` 参考根目录样例，核心字段为 `locked_args = [...]`。
- `--release-config` 模式下 CLI 仅允许 `--release-config`、`--bind`、`--web-bind`、`--help`；其余参数会直接拒绝，避免线上节点语义漂移。

## 常用交互
- 鼠标拖拽：旋转/平移观察视角。
- 滚轮：缩放。
- `W/A/S/D`：移动相机视角（平移 `focus`，2D/3D 均可用；仅在光标位于 3D 视口且未占用文本输入时生效）。
- `2D/3D` 切换：在顶部按钮切换视角模式。
- 控制区（观察模式）：默认仅显示 `播放/暂停` 单按钮；点击 `高级调试` 后展开 `单步` 与 `跳转 0`。
- `F`：对“当前选中对象”执行聚焦（适合人工巡检细节）。
- `F8`：循环切换材质变体预设（`default -> matte -> glossy -> default`），用于快速对比 roughness/metallic 观感。
- 右侧综合面板：查看控制、状态、事件、分块、诊断等模块信息。
- 右侧综合面板支持 `隐藏面板/显示面板` 总开关：隐藏后右侧面板与 Chat 面板都不渲染，3D 区域最大化。
- 最右侧 Chat 面板：独立承载 Agent Chat，不与综合面板混排；顶部区域展开时显示，可通过模块可见性中的 `Chat` 开关隐藏。面板内提供可展开的“预设 Prompt”小区域：支持聊天预设编辑并一键填充输入框，同时可编辑 `system prompt`、`短期目标`、`长期目标`，并直接应用到当前目标 Agent。三个字段会直接预填充当前生效值（未设置 override 时为系统默认值），可直接编辑；展开区内容过高时支持内部滚动。
- Chat 输入：输入框聚焦时，`Enter` 直接发送；`Shift+Enter` 换行。

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

## 自动步骤（Auto Select / Automation Steps）
- `--auto-select-target`：启动后自动选中目标（例如 `first_agent`、`agent:agent-0`）。
- `--automation-steps`：执行一组自动步骤（例如 `mode=3d;focus=agent:agent-0;zoom=0.8;select=agent:agent-0`）。
- 常用于截图回归，减少手工定位误差。

示例：
```bash
./scripts/capture-viewer-frame.sh \
  --scenario llm_bootstrap \
  --addr 127.0.0.1:5131 \
  --auto-select-target first_agent \
  --automation-steps "mode=3d;focus=first_agent;zoom=0.8"
```

## 3D 渲染档位与精调（商业化精致度）

### 档位入口
- `AGENT_WORLD_VIEWER_RENDER_PROFILE=debug|balanced|cinematic`
- 默认：`balanced`
- 建议：先选档位，再做单项覆盖（避免一次性改太多参数导致定位困难）。

### 档位差异（默认值）
- `debug`：低几何复杂度 + 关闭 location 壳层 + 偏可读性材质 + 无阴影 + 轻后处理（`Reinhard`、无 Bloom）。
- `balanced`：中等几何复杂度 + 壳层开启 + 可读性材质 + 三点光默认比率 + `TonyMcMapface` + Bloom 默认开启。
- `cinematic`：高几何复杂度 + 质感材质策略 + 阴影开启 + 三点光更强调轮廓 + `BlenderFilmic` + 更强 Bloom 与色彩后处理。

### 资产层（Geometry/Shell）
- `AGENT_WORLD_VIEWER_ASSET_GEOMETRY_TIER=debug|balanced|cinematic`
- `AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED=1|0`
- 外部 mesh 覆盖（可选，未配置时回退到内置基础几何）：
  - `AGENT_WORLD_VIEWER_AGENT_MESH_ASSET=<path#label>`
  - `AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET=<path#label>`
  - `AGENT_WORLD_VIEWER_ASSET_MESH_ASSET=<path#label>`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_MESH_ASSET=<path#label>`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_MESH_ASSET=<path#label>`
- 示例：
```bash
AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET=models/world/location.glb#Mesh0/Primitive0 \
AGENT_WORLD_VIEWER_AGENT_MESH_ASSET=models/agents/worker.glb#Mesh0/Primitive0 \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

### 材质层（PBR/Fragment）
- `AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY=readability|fidelity`
- `AGENT_WORLD_VIEWER_FRAGMENT_UNLIT=1|0`
- `AGENT_WORLD_VIEWER_FRAGMENT_ALPHA=<0.05..1.0>`
- `AGENT_WORLD_VIEWER_FRAGMENT_EMISSIVE_BOOST=<>=0`
- `AGENT_WORLD_VIEWER_MATERIAL_AGENT_ROUGHNESS=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_AGENT_METALLIC=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_AGENT_EMISSIVE_BOOST=<>=0`
- `AGENT_WORLD_VIEWER_MATERIAL_ASSET_ROUGHNESS=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_ASSET_METALLIC=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_ASSET_EMISSIVE_BOOST=<>=0`
- `AGENT_WORLD_VIEWER_MATERIAL_FACILITY_ROUGHNESS=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_FACILITY_METALLIC=<0..1>`
- `AGENT_WORLD_VIEWER_MATERIAL_FACILITY_EMISSIVE_BOOST=<>=0`
- `AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET=default|matte|glossy`（可选，启动时指定材质变体预设；运行中可按 `F8` 切换）
- 外部颜色覆盖（可选，值为严格 `#RRGGBB`，非法值自动回退默认）：
  - `AGENT_WORLD_VIEWER_AGENT_BASE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_AGENT_EMISSIVE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_LOCATION_BASE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_ASSET_BASE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_ASSET_EMISSIVE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_BASE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_COLOR=<#RRGGBB>`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_COLOR=<#RRGGBB>`
- 外部贴图覆盖（可选，值为 `<path#label>`；web/native 需按运行时支持选择贴图格式，如 `png/ktx2`）：
  - 基础色（Albedo/Base Color）：
    - `AGENT_WORLD_VIEWER_AGENT_BASE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_ASSET_BASE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_PLANT_BASE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_TEXTURE_ASSET=<path#label>`
  - 法线（Normal）：
    - `AGENT_WORLD_VIEWER_AGENT_NORMAL_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_ASSET_NORMAL_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_PLANT_NORMAL_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_STORAGE_NORMAL_TEXTURE_ASSET=<path#label>`
  - 金属度/粗糙度（MetallicRoughness，ORM 贴图中的 MR 通道）：
    - `AGENT_WORLD_VIEWER_AGENT_METALLIC_ROUGHNESS_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_ASSET_METALLIC_ROUGHNESS_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_PLANT_METALLIC_ROUGHNESS_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_STORAGE_METALLIC_ROUGHNESS_TEXTURE_ASSET=<path#label>`
  - 自发光（Emissive）：
    - `AGENT_WORLD_VIEWER_AGENT_EMISSIVE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_ASSET_EMISSIVE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_TEXTURE_ASSET=<path#label>`
    - `AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_TEXTURE_ASSET=<path#label>`
  - 说明：任一通道配置即生效；location 在任一贴图通道覆盖时会启用专用 core/halo 材质，避免与 world/chunk 材质联动。
- 示例：
```bash
AGENT_WORLD_VIEWER_AGENT_BASE_COLOR=#FF6A38 \
AGENT_WORLD_VIEWER_AGENT_EMISSIVE_COLOR=#E66230 \
AGENT_WORLD_VIEWER_LOCATION_BASE_COLOR=#4B88D9 \
AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_COLOR=#B8D8FF \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```
- 材质变体预设示例：
```bash
AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET=matte \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```
- 贴图示例：
```bash
AGENT_WORLD_VIEWER_AGENT_BASE_TEXTURE_ASSET=textures/agents/worker_albedo.png \
AGENT_WORLD_VIEWER_AGENT_NORMAL_TEXTURE_ASSET=textures/agents/worker_normal.png \
AGENT_WORLD_VIEWER_AGENT_METALLIC_ROUGHNESS_TEXTURE_ASSET=textures/agents/worker_mr.png \
AGENT_WORLD_VIEWER_AGENT_EMISSIVE_TEXTURE_ASSET=textures/agents/worker_emissive.png \
AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET=textures/world/location_albedo.png \
AGENT_WORLD_VIEWER_ASSET_BASE_TEXTURE_ASSET=textures/world/asset_albedo.png \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

### 光照层（三点光）
- `AGENT_WORLD_VIEWER_SHADOWS_ENABLED=1|0`
- `AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS=<number>`
- `AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO=<>=0`
- `AGENT_WORLD_VIEWER_RIM_LIGHT_RATIO=<>=0`

### 后处理层（Post Process）
- `AGENT_WORLD_VIEWER_TONEMAPPING=none|reinhard|reinhard_luminance|aces|agx|sbdt|tony_mc_mapface|blender_filmic`
- `AGENT_WORLD_VIEWER_DEBAND_DITHER_ENABLED=1|0`
- `AGENT_WORLD_VIEWER_BLOOM_ENABLED=1|0`
- `AGENT_WORLD_VIEWER_BLOOM_INTENSITY=<0..2>`
- `AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE=<-8..8>`
- `AGENT_WORLD_VIEWER_COLOR_GRADING_POST_SATURATION=<0..2>`

### 推荐启动模板
```bash
AGENT_WORLD_VIEWER_RENDER_PROFILE=cinematic \
AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY=fidelity \
AGENT_WORLD_VIEWER_BLOOM_INTENSITY=0.24 \
AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE=0.35 \
AGENT_WORLD_VIEWER_COLOR_GRADING_POST_SATURATION=1.08 \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

## 工业风主题包（industrial_v2，推荐）

### 资产内容
- 路径：`crates/agent_world_viewer/assets/themes/industrial_v2/`
- 包含：
  - 5 类实体 mesh（`*_industrial_v2.gltf + *.bin`）
  - 5 类实体 PBR 贴图（`base/normal/metallic_roughness/emissive`，默认 512x512）
  - 预设文件：`industrial_v2_default.env`、`industrial_v2_matte.env`、`industrial_v2_glossy.env`

### 一键应用（启动前）
```bash
source crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_default.env
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

切换变体（仅替换 preset）：
```bash
source crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_matte.env
```
```bash
source crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_glossy.env
```

### 运行中主题切换（右侧 Theme Runtime）
- 在右侧面板 `控制` 区域可看到 `Theme Runtime`：
  - `Preset`：`Off / industrial_v2 default / matte / glossy / Custom file`
  - `Apply Theme`：立即应用当前 preset
  - `Auto Hot Reload`：打开后自动检测 preset 文件变更并重载
- 适用场景：美术调参与脚本生成资产后实时复验，无需重启 viewer。

### 运行中主题切换（环境变量）
- `AGENT_WORLD_VIEWER_THEME_PRESET=none|industrial_v2_default|industrial_v2_matte|industrial_v2_glossy|custom`
- `AGENT_WORLD_VIEWER_THEME_PRESET_FILE=<path/to/preset.env>`（设置后优先按文件路径加载）
- `AGENT_WORLD_VIEWER_THEME_HOT_RELOAD=1|0`

示例：
```bash
AGENT_WORLD_VIEWER_THEME_PRESET=industrial_v2_default \
AGENT_WORLD_VIEWER_THEME_HOT_RELOAD=1 \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

自定义 preset 示例：
```bash
AGENT_WORLD_VIEWER_THEME_PRESET_FILE=.tmp/custom_theme.env \
AGENT_WORLD_VIEWER_THEME_HOT_RELOAD=1 \
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```

### 主题包校验（提交前）
```bash
python3 scripts/validate-viewer-theme-pack.py \
  --theme-dir crates/agent_world_viewer/assets/themes/industrial_v2 \
  --profile v2
```

## 工业风主题包（industrial_v1，兼容）
- 路径：`crates/agent_world_viewer/assets/themes/industrial_v1/`
- 预设：`industrial_default.env`、`industrial_matte.env`、`industrial_glossy.env`
- 校验：
```bash
python3 scripts/validate-viewer-theme-pack.py \
  --theme-dir crates/agent_world_viewer/assets/themes/industrial_v1 \
  --profile v1
```

### 批量预览（default/matte/glossy）
```bash
./scripts/viewer-theme-pack-preview.sh \
  --scenario llm_bootstrap \
  --theme-pack industrial_v2 \
  --variants all \
  --base-port 5423
```

输出目录：
- `output/theme_preview/<timestamp>/default/viewer.png`
- `output/theme_preview/<timestamp>/matte/viewer.png`
- `output/theme_preview/<timestamp>/glossy/viewer.png`
- 每个变体目录附带 `live_server.log`、`viewer.log`、`capture_status.txt`、`meta.txt`
- `meta.txt` 记录 `theme_pack` 与实际 `preset_file`
- `capture_status.txt` 必须为 `connection_status=connected` 且 `snapshot_ready=1`（否则脚本失败）

常用参数：
- `--theme-pack <industrial_v2|industrial_v1>`：选择主题包（默认 `industrial_v2`，推荐）。

### 资产重生成
```bash
python3 scripts/generate-viewer-industrial-theme-assets.py --quality v2 --out-dir crates/agent_world_viewer/assets/themes/industrial_v2
```
```bash
python3 scripts/generate-viewer-industrial-theme-assets.py --quality v1 --out-dir crates/agent_world_viewer/assets/themes/industrial_v1
```

## 贴图查看器（可截图）

用途：
- 在统一构图下快速检查贴图观感（base/normal/metallic_roughness/emissive）。
- 支持批量实体来源与材质变体，输出可留痕截图目录。

### 基础调用
```bash
./scripts/viewer-texture-inspector.sh \
  --inspect all \
  --variants all \
  --scenario llm_bootstrap
```

### 常用参数
- `--preset-file <path>`：指定主题预设 env 文件（默认 `industrial_default.env`）。
- `--inspect <list>`：贴图来源实体（`agent,location,asset,power_plant,power_storage,all`）。
- `--variants <list>`：`default,matte,glossy,all`。
- `--base-texture/--normal-texture/--mr-texture/--emissive-texture`：临时覆盖贴图路径。
- `--use-source-mesh`：把“来源实体 mesh”作为预览载体（默认关闭，默认使用 location 载体保证构图稳定）。
- `--out-dir <dir>`：输出目录（默认 `output/texture_inspector/<timestamp>`）。

### 输出目录
- `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer.png`
- 同目录附带：`live_server.log`、`viewer.log`、`capture_status.txt`、`meta.txt`
- `capture_status.txt` 必须为 `connection_status=connected` 且 `snapshot_ready=1`（否则脚本失败）

## Web 闭环（默认，推荐调试/回归）

说明：该闭环用于可视化观察与交互取证，不等价于“浏览器作为完整分布式节点运行”。

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
基础调用：
```bash
./scripts/capture-viewer-frame.sh --scenario asteroid_fragment_detail_bootstrap --addr 127.0.0.1:5131 --tick-ms 300 --viewer-wait 12 --auto-focus-target first_fragment --auto-focus-radius 18
```

常用增强参数：
- `--capture-max-wait <sec>`：覆盖内置截图最大等待时间。
- `--no-prewarm`：跳过预热编译。
- `--keep-tmp`：保留 `.tmp/` 产物便于排查。
- `--auto-focus-keep-2d`：自动聚焦时保持 2D，不强制切 3D。

## 右侧综合面板与 Chat 面板显隐
- 综合右侧面板支持按模块单独显示/隐藏：控制、总览、覆盖层、诊断、事件联动、时间轴、状态明细。
- 综合右侧面板顶部提供总开关：`隐藏面板/显示面板`；隐藏时主面板与 Chat 面板均不占右侧宽度，3D 视口扩展到全宽。
- Chat 功能已拆分为独立最右侧面板，不再出现在综合右侧面板内容区。
- `Chat` 可见性开关关闭时，不渲染独立 Chat 面板且不占用右侧宽度。
- 开关状态会落盘并在重启后恢复。
- 默认缓存路径：`$HOME/.agent_world_viewer/right_panel_modules.json`
- 可通过环境变量覆盖：`AGENT_WORLD_VIEWER_MODULE_VISIBILITY_PATH`

## Web 全屏自适应（wasm）
- Web 端 canvas 跟随浏览器父容器尺寸，默认占满可用视口（非固定 `1200x800` 逻辑窗口体验）。
- 右侧面板宽度采用“最小宽度 + 动态上限（随可用宽度变化）”，在大屏上不再受固定像素上限限制。
- 当需要更大观察区域时，优先使用右侧面板总开关收起面板。

## 选中详情面板
- 点击对象后会在详情区显示信息，支持：
  - Agent
  - Location
  - Asset
  - PowerPlant
  - PowerStorage
  - Chunk
- LLM 场景下，Agent 详情可显示最近决策 I/O（输入、输出、错误与 token/时延摘要）。
- 离线或无 LLM trace 时会显示降级提示，不影响基础详情查看。

## 快速定位 Agent
- 入口：右侧 `Event Link / 事件联动` 区域的 `定位 Agent` 按钮。
- 行为：
  - 优先定位当前已选中的 Agent；
  - 否则定位当前场景字典序第一个 Agent。
- 适合对象密集场景下快速回到 Agent 观察位。

## 全览图缩放切换（2D）
- 2D 视角支持“细节态 / 全览图态”自动切换。
- 默认进入细节态，便于看清 Agent 与局部关系。
- 缩放到阈值后自动进入全览图态，显示简化标记并隐藏部分细节几何。
- 切回近景后自动恢复细节显示。

## 文本可选中/复制面板
- 支持打开可选中文本面板，用于复制状态、事件、诊断与详情文本。
- 面板使用系统快捷键复制（macOS `Cmd+C` / Windows/Linux `Ctrl+C`）。
- 若遮挡视图可在顶部控制区切换显示/隐藏。

## UI 语言切换
- Viewer 支持中文/英文 UI。
- 通过顶部语言控件切换后即时生效。
- 若启用本地配置持久化，重启后会保持最近一次选择。
- 语言切换不改变协议字段，仅改变显示文案。

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
- `doc/world-simulator/viewer-selection-details.md`
- `doc/world-simulator/viewer-right-panel-module-visibility.md`
- `doc/world-simulator/viewer-web-fullscreen-panel-toggle.md`
- `doc/world-simulator/viewer-overview-map-zoom.md`
- `doc/world-simulator/viewer-agent-quick-locate.md`
- `doc/world-simulator/viewer-copyable-text.md`
- `doc/scripts/capture-viewer-frame.md`（native fallback）

## Fragment 元素分块渲染（默认开启）
- 目标：把 location 的 fragment 分块默认显示出来，并按主导元素显示不同颜色。
- 当前行为：不再渲染 location 外层几何与标签，仅保留逻辑锚点；frag 分块始终渲染。
- 选择交互：点击 frag 后，详情面板会显示所属 `location`（ID 与名称）。
- 配置说明：已移除 frag 渲染开关与对应环境变量，不再支持按开关隐藏 frag。
