# Non-Viewer 链上鉴权协议重构（项目管理文档）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/nonviewer-onchain-auth-protocol-hardening.md`
- [x] 新建项目管理文档：`doc/nonviewer-onchain-auth-protocol-hardening.project.md`

### T1 协议与鉴权内核
- [x] 扩展 `agent_world_proto::viewer` 请求结构，新增 `PlayerAuthProof`
- [x] 在 `agent_world::viewer` 实现 canonical payload + ed25519 签名/验签工具
- [x] 补协议与鉴权内核测试（roundtrip + tamper）

### T2 Live 服务端强制鉴权 + 防重放
- [ ] live 请求链路强制 `auth proof` 校验，补全错误码
- [ ] `WorldModel` 持久化 `player_auth_last_nonce` 并实现 nonce 消费校验
- [ ] 补 live/persist 测试（重放拒绝、snapshot 保留）

### T3 客户端接入与收口
- [ ] `agent_world_viewer` 非视觉发送链路接入签名 proof
- [ ] 跑定向回归 + required tier 测试
- [ ] 回写设计/项目文档状态与 devlog 收口

## 依赖
- T2 依赖 T1（先冻结协议与签名负载，后接入 live 验签）。
- T3 依赖 T2（服务端校验上线后再接入客户端签名）。

## 状态
- 当前状态：进行中
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
