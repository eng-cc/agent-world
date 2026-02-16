## 开发工作流
1. 拿到仓库时先阅读开发日志和项目管理文档，了解现状
  1.1 设计文档写在当前设计涉及的最外层文件夹的 doc 文件夹下，格式为 md
  1.2 项目管理文档也写在当前设计涉及的最外层文件夹的 doc 文件夹下，文件名为 设计文档名称.project.md
  1.3 设计文档至少包含:目标、范围、接口/数据、里程碑、风险（可简要）
  1.4 项目管理文档至少包含:任务拆解、依赖、状态（可简要）
2. 如果是新功能要先写设计文档，再根据设计文档拆解项目管理文档(拆成具体任务)
3. 根据拆解好的任务写代码、跑测试，每次完成一个任务
  3.1 所有代码和功能(包括UI)应该都是可以被测试的，单元测试或者模拟闭环测试都可以
  3.2 所有测试都分test_tier_required或者test_tier_full
4. 每次改完代码必须回顾设计文档和项目管理文档，并更新项目管理文档
5. 检查单个rust代码文件不能超过1200行，单个md文档不能超过500行，需要择机拆分模块和文件夹
6. 每个任务(指项目文档里拆解的任务)完成后都需要写任务日志，跑测试
  6.1 任务日志写在当前设计涉及的最外层文件夹的 doc/devlog 文件夹下,文件名为年-月-日.md
  6.2 任务日志应包含:时刻(需要每次任务确认下当前时刻)、完成内容、遗留事项；日志需要保障符合当天实际，后续不能修改，没有行数限制；每天一个日志文件，无需拆分
7. 每个任务(写文档也算一个任务)一个commit，无需询问，及时提交
8. 只要当前项目管理文档中还有后续任务就不要中断，开始下一个任务即可

## 工程架构
- 本仓库内所有新功能必须包含:设计文档、项目管理文档、代码
- 主crate是agent_world其他子模块各自闭环基础模块功能
- third_party下面的代码只可读，不能写
- 执行cargo命令需要如下形式 env -u RUSTC_WRAPPER cargo check
- 使用手册都放在site/doc(cn/en)，可作为静态站内容

## Agent 专用：UI Web 闭环调试（给 Codex 用，Playwright 优先）
- 目标：在无法直接“看实时窗口”时，完成 `启动 Web Viewer -> 自动化交互/取证 -> 继续调试` 的闭环。
- 适用场景：`agent_world_viewer` 可视化问题定位（黑屏、布局、相机交互、文本状态等）。
- 说明：此流程主要给 agent 使用，人类开发者可忽略。

### 标准流程（Web 默认）
1) 启动 live server（含 WebSocket bridge）：
   `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011 --tick-ms 300`
2) 启动 Web Viewer：
   `env -u NO_COLOR ./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173`
3) 使用 Playwright CLI 执行闭环采样（推荐 skill wrapper）：
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
4) 最小验收口径：
   - 页面可加载（snapshot 可见 `canvas`）。
   - `console error = 0`。
   - `output/playwright/` 至少有 1 张截图产物。

### Fallback（仅 native 链路问题）
- 当问题只在 native 图形链路出现，或 Web 端无法复现时，再使用：
  - `./scripts/capture-viewer-frame.sh`
- 该链路定位为历史兼容/应急，不作为默认闭环流程。

### 推荐约定
- Web 闭环产物统一放在 `output/playwright/`。
- 每次调试结束清理 `run-viewer-web.sh` 后台进程，避免端口冲突。
- 若页面首帧空白，优先排查：
  - `trunk` 是否完成首轮编译。
  - 访问地址是否与脚本端口一致。
  - 浏览器控制台是否有 wasm 初始化错误。
