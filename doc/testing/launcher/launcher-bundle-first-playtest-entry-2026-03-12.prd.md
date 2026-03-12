# Agent World：启动器 bundle-first 试玩入口收敛（2026-03-12）

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 当前 `testing-manual.md` 与 `scripts/run-game-test.sh` 都把 `world_game_launcher` 说成默认 Web 闭环入口，但没有明确区分“源码直接运行”与“打包后 bundle 产物运行”。这会让制作人试玩、发布前人工验收和开发回归混用同一口径，导致操作者更容易直接走 `cargo run`，偏离真实交付物体验，也放大静态资源、执行目录与本地状态混杂带来的误判。
- Proposed Solution: 将启动器试玩入口明确收敛为“双模”策略：`bundle-first` 作为制作人试玩、发布前人工验收和对外交付样张的默认入口；`scripts/run-game-test.sh` 保留，但降级为开发回归 bootstrap，并新增 `--bundle-dir` 以直接消费打包产物。进一步提供 `scripts/run-producer-playtest.sh` 作为制作人一键入口，自动准备 bundle 后再进入 bundle 模式启动。
- Success Criteria:
  - SC-1: 手册、启动器人工测试清单和脚本帮助文本都明确区分 `bundle` 验收入口与源码回归入口。
  - SC-2: `scripts/run-game-test.sh` 支持 `--bundle-dir <bundle>`，可直接通过 `<bundle>/run-game.sh` 启动游戏。
  - SC-3: `scripts/run-game-test-ab.sh` 无需额外适配即可透传 `--bundle-dir`，并在帮助口径中明确其作为 bundle 自动化哨兵的用法。
  - SC-4: 至少完成一轮 bundle 构建 + `run-game-test-ab.sh --bundle-dir <bundle>` 闭环验证，并输出明确的通过或阻断证据，避免口径停留在纸面。
  - SC-5: 提供单命令制作人试玩入口，默认可复用或自动构建本地 bundle，再进入 bundle 模式启动。
  - SC-6: 单命令入口支持可选的 `--open-headed`，在 URL 就绪后自动打开 headed 浏览器，避免制作人再手动复制 URL。
  - SC-7: 通过 `--open-headed` 拉起的浏览器会话在脚本退出时自动关闭，避免遗留残窗或脏会话。

## 2. User Experience & Functionality
- User Personas:
  - `producer_system_designer`：需要以真实交付物口径亲自试玩，而不是落回源码态调试链路。
  - `qa_engineer`：需要一条稳定、可审计的 bundle 验收入口，避免把开发态结果误当发布结论。
  - `viewer_engineer`：需要保留源码态快速回归入口，便于调试协议、静态资源与 Viewer Web 问题。
- User Scenarios & Frequency:
  - 每次制作人试玩、发布前人工验收、对外样张抽检时执行 bundle-first 入口。
  - 每次脚本开发调试、端口/协议问题排查时，允许继续使用源码态 bootstrap。
- User Stories:
  - PRD-TESTING-LAUNCHER-BUNDLE-001: As a `producer_system_designer`, I want manual playtests to start from packaged launcher artifacts, so that my verdict reflects the real product handoff.
  - PRD-TESTING-LAUNCHER-BUNDLE-002: As a `qa_engineer`, I want the existing `run-game-test.sh` bootstrap to consume a bundle via one flag, so that automation and manual validation share one script surface.
- Critical User Flows:
  1. Flow-LBFP-001: `run-producer-playtest.sh --open-headed -> 自动准备/复用 bundle -> run-game-test.sh --bundle-dir -> 自动打开 headed agent-browser -> 人工游玩 -> 脚本退出时自动关闭该浏览器会话 -> 记录人工结论`
  2. Flow-LBFP-002: `run-game-test.sh --bundle-dir <bundle> -> 输出 URL/日志 -> run-game-test-ab.sh 采样`
  3. Flow-LBFP-003: `run-game-test.sh (源码模式) -> 开发者快速复现 -> 不作为发布结论`

## 3. Scope & Acceptance
- In Scope:
  - `scripts/run-game-test.sh` 增加 bundle 入口并保留源码 fallback。
  - `scripts/run-producer-playtest.sh` 提供制作人一键入口。
  - `scripts/run-game-test-ab.sh`、`testing-manual.md`、启动器人工测试清单同步 bundle-first 口径。
  - `doc/testing/project.md`、`doc/testing/prd.index.md`、`doc/testing/README.md`、`doc/devlog/2026-03-12.md` 回写追溯。
- Out of Scope:
  - 不移除 `run-game-test.sh` 源码模式。
  - 不改造 `world_game_launcher` / `world_web_launcher` 参数协议。
  - 不新建独立的第三套 bundle 专用自动化脚本。
- Acceptance Criteria:
  - AC-1: `./scripts/run-game-test.sh --help` 明确 `--bundle-dir` 的定位与 bundle-first 推荐口径。
  - AC-2: 传入 `--bundle-dir` 时，脚本使用 `<bundle>/run-game.sh` 而不是 `cargo run` 拉起游戏。
  - AC-3: `testing-manual.md` 明确“制作人试玩 / 发布前人工验收默认走 bundle-first；源码模式仅用于开发回归”。
  - AC-4: `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.prd.md` 将 bundle-first 写为执行说明的一部分。

## 4. Technical Specifications
- Architecture Overview: 维持现有 `run-game-test.sh -> URL/日志/端口就绪 -> agent-browser` 的外部契约不变，仅把启动执行器从单一 `cargo run world_game_launcher` 扩展为 `bundle mode` 与 `source mode` 两条分支，其中 `bundle mode` 优先供人工验收和自动化哨兵使用。
- Integration Points:
  - `scripts/run-game-test.sh`
  - `scripts/run-game-test-ab.sh`
  - `scripts/run-producer-playtest.sh`
  - `scripts/build-game-launcher-bundle.sh`
  - `testing-manual.md`
  - `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.prd.md`
- Edge Cases & Error Handling:
  - `--bundle-dir` 指向不存在目录：脚本必须 fail-fast 并提示目录路径。
  - `--bundle-dir` 缺少 `run-game.sh`：脚本必须明确提示 bundle 不完整。
  - bundle 模式下未显式传 `--viewer-static-dir`：默认使用 bundle 自带 `web/`，不再偷偷 fresh build 源码目录。
  - `--headless` 若命中 `SwiftShader` / software renderer：必须按浏览器环境阻断快失败，并输出可操作提示，不得误判成 fresh Web 构建或玩法回归。
  - 开发者仍需源码回归：保留现有 `cargo run` 分支，但帮助文本与手册必须标注其仅供开发排障。
- Non-Functional Requirements:
  - NFR-LBFP-1: 新增 bundle 模式不能破坏现有 `run-game-test-ab.sh` 透传契约。
  - NFR-LBFP-2: 脚本帮助、手册和人工清单的口径必须 0 冲突。
  - NFR-LBFP-3: bundle-first 入口的失败必须在一次执行中能定位到“目录错误 / 产物不完整 / 端口冲突 / 运行时失败”中的至少一类。
  - NFR-LBFP-4: `--headless` 下若浏览器退化到 `SwiftShader`，自动化必须给出环境级阻断，而不是返回模糊的 `connecting` 超时。
- Security & Privacy: 不新增敏感数据采集，仅调整启动入口与文档口径。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (LBFP-1): 建立专题 PRD / design / project，并更新 testing 主索引。
  - v1.1 (LBFP-2): 为 `run-game-test.sh` 增加 `--bundle-dir` 并同步帮助文本。
  - v1.2 (LBFP-3): 更新主手册与人工清单，将 bundle-first 明确为人工验收默认入口。
  - v1.3 (LBFP-4): 完成 bundle 实机闭环验证与 devlog 收口。
- Technical Risks:
  - 风险-1: 若只改文档不改脚本，使用者仍会被现成 `cargo run` 入口带回源码模式。
  - 风险-2: 若直接删除源码模式，会影响开发期问题复现效率。
  - 风险-3: 若 bundle 模式继续沿用源码静态目录 fresh build 逻辑，会再次混淆“真实产物”与“开发态修补”的边界。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LAUNCHER-BUNDLE-001 | LBFP-1/3/6/7/8 | `test_tier_required` | 手册、人工清单、README、索引互链审阅 | 人工验收口径、一键试玩入口说明 |
| PRD-TESTING-LAUNCHER-BUNDLE-002 | LBFP-2/4/5/6/7/8 | `test_tier_required` | `bash -n` + `--help` + bundle 构建 + `run-producer-playtest.sh --open-headed` + 退出后确认不残留对应 headed 浏览器进程/窗口 + `run-game-test-ab.sh --bundle-dir`，记录通过或阻断证据 | 启动脚本 bootstrap、bundle 试玩闭环 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LBFP-001 | 保留 `run-game-test.sh`，但切换到 bundle-first 解释与用法 | 直接删除脚本 | 现有自动化依赖较多，立即删除会破坏回归面。 |
| DEC-LBFP-002 | 用 `--bundle-dir` 把 bundle 入口并入现有脚本 | 新建独立 bundle 脚本 | 共享端口、日志、URL 输出与 agent-browser 接口，更易维护。 |
| DEC-LBFP-003 | 制作人试玩/发布前验收默认走 bundle | 继续默认源码态 `cargo run` | 前者更贴近真实交付物，也更能暴露 bundle 级问题。 |
| DEC-LBFP-004 | 提供 `run-producer-playtest.sh` 作为 bundle-first 一键入口 | 要求制作人手动执行 build + bootstrap 两条命令 | 单命令更符合制作人实际使用路径，也更不容易退回源码模式。 |
| DEC-LBFP-005 | `--open-headed` 做成可选模式 | 默认总是自动打开浏览器 | 让脚本既能服务纯起栈，也能服务制作人直接入场，避免强绑浏览器副作用。 |
| DEC-LBFP-006 | `run-producer-playtest.sh --open-headed` 退出时自动关闭自己拉起的浏览器会话 | 保留 `agent-browser` 会话常驻，要求人工手动关窗 | 制作人单命令试玩默认应无残窗副作用，脚本应负责自己创建资源的收尾。 |
