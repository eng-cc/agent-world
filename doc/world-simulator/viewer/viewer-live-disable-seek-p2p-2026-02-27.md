# Viewer Live 禁用 Seek（P2P 不可回退）2026-02-27

## 目标
- 在 viewer live 模式中禁用 `seek` 控制，确保 P2P 实时链路不存在“回退/跳时”语义。
- 保持世界演化单调前进，避免“测试接口可 seek”与“P2P 共识不可回退”之间的行为冲突。

## 范围

### In Scope
- `agent_world` live 请求处理：`ViewerControl::Seek` 在 live 模式不再生效。
- `agent_world_viewer` 的玩家/测试控制入口收敛：不再把 `seek` 当作可用控制动作。
- 同步更新 live 相关测试与文档记录。

### Out of Scope
- 不删除协议层 `ViewerControl::Seek`（保留与非 live 场景兼容）。
- 不修改非 live viewer server 的历史浏览/游标语义。
- 不做 timeline 模块的全量重构。

## 接口/数据
- live 模式控制入口语义：
  - 允许：`play`、`pause`、`step`
  - 禁用：`seek`（以及 Web Test API 的 `seek_event`）
- 反馈语义：被禁用控制不会驱动世界回退；测试接口白名单不再暴露 seek 系动作。

## 里程碑
1. M1：建档并确认“live 无 seek”边界。
2. M2：代码收敛（live 控制处理 + web test api/玩家入口动作集合）。
3. M3：测试回归 + 项目文档/devlog 收口。

## 风险
- 既有 A/B 脚本若仍依赖 `seek`，将出现行为回归（需调整为无 seek 探针）。
- 协议仍保留 `Seek` 枚举，若调用端未同步策略，可能出现“发送成功但 live 不执行”的认知偏差。
- 需要避免把“live 禁用 seek”误扩散到非 live 服务。
