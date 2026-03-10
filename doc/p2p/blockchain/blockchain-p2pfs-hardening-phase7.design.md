# 区块链 + P2PFS 硬改造 Phase 7 设计

- 对应需求文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.project.md`

## 1. 设计定位
定义 sequencer `accepted_action_signer_public_keys` 从原样字符串比较升级到规范化校验的方案，使 action signer 信任根与 membership signer 保持同一治理口径。

## 2. 设计结构
- 策略扩展层：为 sequencer action signer 增加规范化白名单治理。
- 校验一致性层：沿用 membership 同类校验语义，避免两套 signer 口径分叉。
- 错误口径层：对白名单误配和 signer 不命中返回稳定错误。
- 回归保护层：补齐 action signer 大小写、重复和非法值测试。

## 3. 关键接口 / 入口
- `accepted_action_signer_public_keys`
- sequencer signer 校验路径
- 规范化比较语义
- action signer 回归测试

## 4. 约束与边界
- 保持现有协议字段，不新增额外 signer 元数据结构。
- action signer 治理需与 membership signer 尽量同构。
- 白名单为空时维持兼容旧行为。
- 不在本阶段引入更重的 sequencer 身份治理平台。

## 5. 设计演进计划
- 先统一 action signer 白名单口径。
- 再接规范化校验与错误处理。
- 最后通过 targeted 回归冻结 phase7。
