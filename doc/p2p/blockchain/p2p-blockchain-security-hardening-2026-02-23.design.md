# P2P/区块链链路安全硬化设计

- 对应需求文档: `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.project.md`

## 1. 设计定位
定义 replication、signed consensus、fetch 鉴权和 membership 默认策略的整体安全硬化方案，把复制链路从“基本可跑”提升到“可生产约束”。

## 2. 设计结构
- replication 原子化层：`apply_replication_record` 失败不污染 guard。
- signer 绑定层：signed consensus 模式下强制 validator_signer_public_keys 完整覆盖并与本地节点绑定一致。
- writer 鉴权层：远端 replication writer 需通过签名校验和 allowlist 授权。
- fetch 与订阅层：fetch 请求签名鉴权、NetworkSubscription 有界缓存、membership DHT 默认 require_signature。

## 3. 关键接口 / 入口
- `apply_replication_record`
- `validator_signer_public_keys`
- `NodeReplicationConfig` writer allowlist
- `FetchCommitRequest` / `FetchBlobRequest`

## 4. 约束与边界
- 继续使用当前 ed25519 / HMAC 体系，不改底层 libp2p 握手。
- viewer live signer 映射接线只作为 T2 配套，不扩大 viewer 业务 scope。
- 兼容性收紧后会拒绝旧配置，这是有意识的安全提升。
- 安全增强必须同时配套清晰错误信息和回归测试。

## 5. 设计演进计划
- 先做 replication guard 原子化。
- 再收紧 signer 绑定、writer allowlist 和 fetch 鉴权。
- 最后补订阅容量上限与 membership 默认策略收紧。
