# Agent World Runtime：节点密钥 config.toml 自举（项目管理文档）

## 任务拆解
- [x] NKEY-1：设计文档与项目管理文档落地。
- [x] NKEY-2：实现启动阶段读取/生成/写回 `config.toml` 的节点密钥逻辑。
- [x] NKEY-3：补齐单元测试并完成 `agent_world` 回归。
- [x] NKEY-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `config.example.toml`
- `crates/node`

## 状态
- 当前阶段：NKEY-1~NKEY-4 全部完成。
- 下一步：如需对接真实签名链路，可在 node/runtime 启动阶段消费该密钥并接入 Action/Head 签名。
- 最近更新：2026-02-16。
