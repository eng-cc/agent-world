# P2P Mobile Light Client 权威状态架构（2026-03-06）设计

- 对应需求文档: `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
- 对应项目管理文档: `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.project.md`

## 1. 设计定位
定义移动轻客户端 + 链下权威模拟 + 链上状态承诺/挑战的分层设计，让手机端只承担输入、渲染与最终性消费职责。

## 2. 设计结构
- 轻客户端入口层：客户端只发送带签名的 `intent`，不运行本地权威模拟。
- 权威执行层：权威节点按 tick 排序执行输入并产出 delta、`state_root` 与 `data_root`。
- 承诺挑战层：链上记录批次承诺，watcher 在窗口内复算 challenge 并触发 resolve/slash。
- 最终性恢复层：客户端消费 `pending/confirmed/final` 三段状态，并通过快照+增量完成重连追平。
- 会话安全层：通过 session key、吊销与换钥机制约束移动端写入权限。

## 3. 关键接口 / 入口
- `intent(player_id, session_pubkey, tick, seq, action, payload_hash, sig)`
- `authoritative_batch(batch_id, state_root, data_root, patches)`
- `authoritative_challenge` / `authoritative_challenge_ack` / `error`
- `authoritative_recovery` / `authoritative_recovery_ack` / `error`
- `RequestSnapshot(snapshot_hash, log_cursor, stable_batch_id, reorg_epoch)`

## 4. 约束与边界
- 移动端不得重新承担本地权威模拟职责。
- 只有 `final` 批次可用于资产结算与排行统计。
- challenge 窗口内若存在争议，批次不得进入 final。
- 恢复链路必须支持重组回滚、快照校验与会话吊销拦截。

## 5. 设计演进计划
- 先落地 intent 幂等、权威批次与最终性状态机。
- 再补齐 challenge/resolve/slash 与 recovery 语义。
- 最后固化 required/full 回归矩阵，作为移动轻客户端主路径门禁。
