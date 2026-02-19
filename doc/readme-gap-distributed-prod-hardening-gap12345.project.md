# README 分布式计算与存储生产级收口（Gap 1/2/3/4/5）项目管理文档

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap-distributed-prod-hardening-gap12345.md`）与项目管理文档（本文件）
- [x] T1：`agent_world_node` 复用 `agent_world_consensus` PoS 内核（提议者选择/阈值判定）并补测试（Gap 1）
- [ ] T2：存储挑战门控升级为多样本网络验证与匹配阈值，补齐回归（Gap 2）
- [ ] T3：gap sync 升级为分高度重试与错误可观测，补齐回归（Gap 3）
- [ ] T4：DistFS sidecar 恢复失败审计落盘，保持 JSON 兜底并补测试（Gap 4）
- [ ] T5：`triad_distributed` 最小引导 + gossip 自动发现，补齐 CLI/启动测试（Gap 5）
- [ ] T6：执行 `env -u RUSTC_WRAPPER cargo check` + required-tier 定向测试并收口文档/devlog

## 依赖
- T2 依赖 T1（共识判定路径先稳定，避免门控误判放大）。
- T3 依赖 T2（先确定复制门控语义，再做补洞重试策略）。
- T5 与 T1~T4 弱耦合，可并行，但在 T6 统一回归。
- T6 依赖 T1~T5 全部完成。

## 状态
- 当前阶段：进行中（T0/T1 完成，T2~T6 待执行）
- 阻塞项：无
- 下一步：执行 T2（存储挑战门控多样本验证与阈值收口）
