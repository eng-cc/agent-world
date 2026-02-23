# P2P/区块链链路安全硬化（2026-02-23，项目管理文档）

## 任务拆解
- [x] T0：完成设计文档与项目管理文档建档。
- [ ] T1：实现 replication guard 原子化语义，并补“失败不污染状态”测试。
- [ ] T2：实现 signed consensus 模式下 signer 绑定完整性强制，并完成 `world_viewer_live` signer 映射接线。
- [ ] T3：实现 replication 远端 writer allowlist 授权校验。
- [ ] T4：实现 fetch-commit/fetch-blob 请求签名鉴权（客户端签名 + 服务端验签）。
- [ ] T5：实现网络订阅有界缓存与 membership DHT 恢复默认策略收紧。
- [ ] T6：补齐定向/回归测试，回写文档状态与 devlog。

## 依赖
- T2 依赖 T1（先稳定 replication 基础状态语义，再收紧身份校验链路）。
- T3 依赖 T2（writer 授权依赖 signer 绑定与键值规范）。
- T4 依赖 T3（fetch 鉴权复用 writer/signer 授权语义）。
- T5 可与 T4 并行，但建议串行以减少回归噪音。
- T6 依赖 T1~T5 全部完成。

## 状态
- 当前阶段：T1 进行中。
- 阻塞项：无。
- 最近更新：2026-02-23。
