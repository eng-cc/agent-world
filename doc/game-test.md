# 游戏发布前的测试手册

## 目标
- 以真实玩家视角完成 Web 闭环游玩验证，确认当前构建在“可进入、可操作、可持续推进”三个层面可用。
- 统一测试产物口径（测试卡片 + 录屏 + 截图 + 控制台日志），支持问题复盘与回归对比。

## 范围

### In Scope
- 使用 Playwright 执行真实玩家游玩流程。
- 填写 `doc/playability_test_card.md` 并落盘到 `doc/playability_test_result/`。
- 保留测试过程录屏及关键截图。

### Out of Scope
- 不覆盖代码级单元测试（由 `testing-manual.md` 其他套件负责）。
- 不替代性能压测或长稳压测。

## 接口/数据
- 启动接口：`./scripts/run-game-test.sh`
- A/B 量化复测接口：`./scripts/run-game-test-ab.sh`
- 浏览器自动化接口：`./.codex/skills/playwright/scripts/playwright_cli.sh`
- 测试卡片模板：`doc/playability_test_card.md`
- 结果目录：
  - 测试卡片：`doc/playability_test_result/`
  - 录屏/截图：`output/playwright/playability/`

## 里程碑
- M1：启动链路可一键拉起并进入可交互页面。
- M2：完成单轮游玩并产出完整证据（卡片 + 录屏 + 日志）。
- M3：按发布节奏执行多轮复测并追踪缺陷收敛。

## 风险
- WebGL/wasm 在特定平台下可能出现兼容性或渲染崩溃问题。
- WS 链路异常可能导致“连接成功但 tick 不推进”的假可用状态。
- 人工执行步骤较多时易出现参数错误，需统一走脚本入口。

## 可玩性测试
你就是游戏玩家，不能读其他的文档或代码（Playwright 操作方法文档除外），直接通过 Playwright 来实际玩游戏，并填写 playability_test_card，同时需要测试过程中给 Playwright 录屏，方便后续检查。

## 一键启动（避免参数错误）
统一使用脚本启动，不要手工拼接 `world_viewer_live` 与 `run-viewer-web.sh` 参数：

```bash
./scripts/run-game-test.sh
```

脚本会自动完成：
- 启动 `world_viewer_live`（固定使用 `--web-bind`，避免 WS 握手参数错误）
- 启动 Web viewer（`run-viewer-web.sh`）
- 端口与主页就绪检查
- 输出可直接用于 Playwright 的 URL（包含 `test_api=1`）
- `Ctrl+C` 时自动清理两个进程

## Playwright 进入游戏
```bash
PLAYWRIGHT_CLI_SESSION=game-test-open \
./.codex/skills/playwright/scripts/playwright_cli.sh open "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011&test_api=1" --headed
```

## 推荐：A/B 分段 + 量化复测
优先使用以下脚本执行可复现闭环，降低“命令漂移”与“手工漏步骤”风险：

```bash
./scripts/run-game-test-ab.sh
```

脚本会自动完成：
- 启动/复用测试栈并打开 Playwright 会话
- 稳定截图（固定文件名）与录屏
- A 段：`play -> 观察 -> pause`
- B 段：`step/seek` 控制探针 + 无效动作探针
- 产出量化指标：
  - `TTFC`（首次可控时间，ms）
  - `有效控制命中率`（有效推进控制次数 / 预期推进控制次数）
  - `无进展窗口时长`（connected 下 tick 不变最长窗口）
- 输出结果：
  - `ab_metrics.json`
  - `ab_metrics.md`
  - `card_quant_metrics.md`（可直接粘贴到卡片）
