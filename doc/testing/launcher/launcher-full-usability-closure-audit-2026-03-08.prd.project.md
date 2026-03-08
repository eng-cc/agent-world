# Agent World: 启动器全功能可用性审查与闭环验收（2026-03-08）（项目管理）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] LAUNCHREV-1 (PRD-TESTING-LAUNCHER-REVIEW-001): 完成专题 PRD 与项目管理文档建档，明确审查范围/分级标准/追溯口径。
- [x] LAUNCHREV-2 (PRD-TESTING-LAUNCHER-REVIEW-001/002/003): 执行启动器定向回归与脚本行为审查（迁移入口 + 阻断入口 + 参数兼容）。
- [x] LAUNCHREV-3 (PRD-TESTING-LAUNCHER-REVIEW-002): 执行真实 Web 闭环（`world_game_launcher + Playwright`）并归档证据。
- [ ] LAUNCHREV-4 (PRD-TESTING-LAUNCHER-REVIEW-001/003): 输出可用性分级结论、风险项与后续动作，完成文档/devlog 收口。

## 依赖
- doc/testing/launcher/launcher-full-usability-closure-audit-2026-03-08.prd.md
- `testing-manual.md`
- `doc/testing/manual/web-ui-playwright-closure-manual.prd.md`
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `scripts/run-game-test.sh`
- `scripts/viewer-release-qa-loop.sh`
- `scripts/s10-five-node-game-soak.sh`
- `scripts/p2p-longrun-soak.sh`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-08
- 当前阶段：进行中（已完成 LAUNCHREV-1/2/3）
- 阻塞项：无
- 下一步：执行 LAUNCHREV-4（输出审查结论与风险分级）
