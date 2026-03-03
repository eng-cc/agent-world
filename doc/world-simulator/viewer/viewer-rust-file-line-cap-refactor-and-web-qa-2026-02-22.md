# Viewer Rust 文件行数上限重构与 Web 闭环对比（2026-02-22）

## 目标
- 将 `crates/agent_world_viewer/src` 中超过 1200 行的 Rust 文件拆分到不超过 1200 行，且行为不变。
- 在重构后执行一轮 Web Playwright 闭环（S6）验证，输出可操作性对比结论（重构前基线 vs 重构后实测）。

## 范围
- 代码范围：`crates/agent_world_viewer/src/**`。
- 文档范围：`doc/world-simulator/` 设计与项目管理文档、`doc/devlog/2026-02-22.md`。
- 验证范围：
  - viewer crate 单测与 wasm check。
  - `testing-manual.md` S6 Web 闭环（live server + web viewer + Playwright 语义步骤 + screenshot + console）。
- 非范围：
  - 不改变 viewer 协议字段和业务语义。
  - 不引入新的 UI 功能。

## 接口/数据
- 重构策略：
  - 保持现有函数签名与调用链，优先采用“子模块下沉 + 根模块 re-export”。
  - 测试块优先拆到独立 `*_tests.rs` 文件。
- 关键约束：
  - 单个 Rust 文件 `<= 1200` 行。
  - 现有测试语义不变。
- 闭环证据：
  - Playwright `snapshot`、`console`、`screenshot`。
  - 语义接口 `window.__AW_TEST__` 的状态与动作执行结果。

## 里程碑
- M1：建档完成并确认超限文件清单。
- M2：完成第一批文件拆分与回归（低风险拆分）。
- M3：完成 `main.rs`/聊天面板等高体量文件拆分与回归。
- M4：完成 Web Playwright 闭环，产出截图与运行证据。
- M5：完成文档回写、devlog 记录和结论收口。

## 里程碑状态（2026-02-22 收口）
- [x] M1：完成。
- [x] M2：完成。
- [x] M3：完成。
- [x] M4：完成。
- [x] M5：完成。

## 可操作性对比结论（重构前 vs 重构后）
- 代码结构：
  - 重构前：`agent_world_viewer` 存在 7 个超限文件（最大 `main.rs = 2231` 行，`egui_right_panel_chat.rs = 1904` 行）。
  - 重构后：超限文件清零，关键文件降至 `main.rs = 915`、`egui_right_panel_chat.rs = 935`，其余均 `<= 1200`。
- Web 闭环可操作性：
  - 重构前基线（语义目标）：需要保持此前已修复的窄屏可操作、连接容错与聊天链路行为不回退。
  - 重构后实测（Playwright）：
    - `window.__AW_TEST__` 可用；
    - `runSteps(mode=3d;focus=first_location;zoom=0.85;select=first_agent)` 可执行；
    - `getState()` 返回 `connectionStatus=connected`、`selectedKind=agent`、`selectedId=agent-0`；
    - console `Errors = 0`，无新增运行时错误。
- 结论：
  - 本次重构主要改善可维护性与可测试性（文件体量显著下降），未引入可操作性回退。
  - 用户可见交互链路（连接、选择、控制、状态读取）与重构前保持一致。

## 风险
- 大文件拆分时容易出现可见性（module visibility）与 re-export 漏洞，导致编译/测试失败。
- `main.rs` 拆分涉及系统调度函数，若符号迁移不完整会影响启动链路。
- Web 闭环依赖本地端口和浏览器环境，需处理端口占用与进程清理。
