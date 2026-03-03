# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 8）设计文档

## 目标
- 把 membership 与 sequencer 的 ed25519 signer 公钥白名单校验/规范化逻辑抽到共享模块，消除重复实现。
- 统一 `signature` 验签返回的 signer 公钥口径（小写规范化 hex），减少调用侧重复 normalize。
- 在不改协议字段的前提下，提高签名治理代码的一致性与可维护性。

## 范围

### In Scope
- **HP8-1：共享公钥治理工具模块**
  - 新增 `ed25519` 公钥治理工具（crate 内部）：
    - 单个公钥：trim + 非空 + hex 校验 + 32-byte 长度校验 + 小写规范化。
    - 白名单集合：逐项规范化与去重，重复项 fail-fast。
  - 统一错误口径，保留字段级别可定位信息（例如 `accepted_*[index]`）。

- **HP8-2：调用侧接线统一**
  - `membership_logic` 改用共享工具，移除本地重复 normalize/allowlist 实现。
  - `sequencer_mainloop` 改用共享工具，移除本地重复 normalize/allowlist 实现。
  - `signature` 验签返回规范化 signer 公钥（小写 hex），与策略比较口径一致。

- **HP8-3：测试与回归**
  - 复用并验证既有 membership/sequencer allowlist 测试在新共享实现下全部通过。
  - 增加共享工具模块的最小单测（非法 key、重复 key、大小写规范化）。
  - 执行 `agent_world_consensus` 与 `agent_world` `test_tier_required` 回归。

### Out of Scope
- CA/证书链、公钥托管、轮换审批流程。
- 线协议和结构体字段变更（`signature` 字符串格式保持不变）。
- 引入新的签名算法或多签方案。

## 接口 / 数据

### 新增共享模块（crate 内部）
```rust
normalize_ed25519_public_key_hex(...)
normalize_ed25519_public_key_allowlist(...)
```

### 行为约束
- 公钥统一规范化为小写 64 hex 字符串（对应 32-byte）。
- allowlist 非空时按规范化集合比较 signer 公钥，大小写无关。

## 里程碑
- **HP8-M0**：设计文档 + 项目管理文档。
- **HP8-M1**：共享模块 + membership/sequencer/signature 接线。
- **HP8-M2**：测试回归与文档收口。

## 风险
- 共享化后若错误口径变化过大，可能影响已有测试断言与运维检索关键词。
- 验签返回值规范化可能影响依赖原始大小写字符串的边缘逻辑，需要确认调用侧仅做比较语义。
- 若共享工具实现过度抽象，会降低可读性；本期保持函数粒度小且用途明确。
