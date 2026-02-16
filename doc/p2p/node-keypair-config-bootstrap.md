# Agent World Runtime：节点密钥 config.toml 自举（设计文档）

## 目标
- 在节点启动时确保根目录 `config.toml` 存在可用的节点密钥字段（公钥/私钥）。
- 当字段缺失时自动生成一对密钥并写回 `config.toml`，形成“启动即自举”的最小闭环。
- 不改变现有节点主循环行为，仅补配置层能力。

## 范围

### In Scope
- 在 `world_viewer_live` 节点启动路径增加：
  - 读取 `config.toml` 的 `[node]` 区块。
  - 若 `private_key/public_key` 缺失，自动生成 ed25519 密钥对。
  - 将密钥字段写回 `config.toml`。
- 保持已有 CLI 参数和节点启动流程兼容。
- 增加单元测试覆盖：
  - 文件不存在时自动创建并写入密钥。
  - 已有密钥时不重复生成。
  - 私钥存在但公钥缺失时可自动补全公钥。

### Out of Scope
- 生产级密钥托管（HSM/KMS）。
- 密钥轮换、吊销、分层权限策略。
- 跨节点密钥注册与身份治理。

## 接口 / 数据

### config.toml 结构（草案）
```toml
[node]
private_key = "<hex>"
public_key = "<hex>"
```

### 启动流程（草案）
- `start_live_node` 调用 `ensure_node_keypair_in_config(config_path)`。
- 若 `node_enabled=false`，不触发密钥自举。

## 里程碑
- NKEY-1：设计文档与项目管理文档落地。
- NKEY-2：实现读取/生成/写回逻辑并接入启动流程。
- NKEY-3：补齐单元测试并完成回归。
- NKEY-4：文档状态与 devlog 收口。

## 风险
- 回写 `config.toml` 会重新序列化 TOML，可能丢失注释格式。
- 若运行目录不可写，节点启动将报错，需要明确错误提示。
- 已存在非法密钥格式时需要明确失败语义，避免静默覆盖。
