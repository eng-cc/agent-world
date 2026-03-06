# 游戏发布前的测试手册(你不能改这个文档)

审计轮次: 5

- 对应项目管理文档: doc/playability_test_result/game-test.prd.project.md

## 可玩性测试
你就是游戏玩家，不能读其他的文档或代码(Playwright操作方法文档除外)，直接通过Playwright来实际玩游戏，并填写playability_test_card，同时需要测试过程中给Playwright录屏，方便后续检查

## 一键启动（避免参数错误）
统一使用脚本启动，不要手工拼接 `world_viewer_live` 与 `run-viewer-web.sh` 参数：

```bash
./scripts/run-game-test.sh
```

脚本会自动完成：
- 启动 `world_game_launcher`（默认托管游戏进程、Web 静态服务与 WebSocket bridge）
- 按当前控制面参数启动内置 Web viewer 服务（不再单独启动 `run-viewer-web.sh` 进程）
- 端口与主页就绪检查
- 输出可直接用于 Playwright 的 URL（包含 `test_api=1`）
- `Ctrl+C` 时自动清理 launcher 托管进程

## Playwright 进入游戏
```bash
PLAYWRIGHT_CLI_SESSION=game-test-open \
"${CODEX_HOME:-$HOME/.codex}/skills/playwright/scripts/playwright_cli.sh" open "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011&test_api=1" --headed
```
