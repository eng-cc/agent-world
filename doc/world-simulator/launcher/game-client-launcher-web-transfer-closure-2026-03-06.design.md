# 启动器 Web 链上转账闭环设计（2026-03-06）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.project.md`

## 1. 设计定位
定义 wasm 启动器通过 `world_web_launcher` 代理链路完成链上转账提交、错误反馈与状态展示的最小闭环。

## 2. 设计结构
- wasm 提交层：转账窗口收集字段并通过 web 控制面发起提交。
- 控制面代理层：`/api/chain/transfer` 透传到 `world_chain_runtime` 的转账接口。
- 状态反馈层：成功返回 `action_id`，失败返回 `error_code/error`，并保留 in-flight 门控。
- 链状态保护层：链未就绪或未启用时前置阻断，不进入无效提交。

## 3. 关键接口 / 入口
- `POST /api/chain/transfer`
- `/v1/chain/transfer/submit`
- `transfer_window.rs`
- `app_process_web.rs`

## 4. 约束与边界
- 控制面只做代理，不复制账本业务规则。
- Web 与 native 必须共用成功/拒绝/失败三态语义。
- 链不可达时要快速失败并给出结构化错误。
- 本轮不实现钱包托管、多签或跨链能力。

## 5. 设计演进计划
- 先接通控制面代理和 wasm 提交入口。
- 再补错误展示、链状态阻断与 in-flight 门控。
- 最后通过 required 回归验证 web 转账闭环稳定可用。
