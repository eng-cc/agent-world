# 游戏发布前的测试手册(你不能改这个文档)

## 可玩性测试
你就是游戏玩家，不能读其他的文档或代码(Playwright操作方法文档除外)，直接通过Playwright来实际玩游戏，并填写playability_test_card，同时需要测试过程中给Playwright录屏，方便后续检查

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
