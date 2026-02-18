# README 对齐收口：P0/P1 项目管理文档

## 任务拆解
- [x] T0：输出设计文档（`/Users/scc/ccwork/agent-world/doc/readme-p0-p1-closure.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1（P0）：扩展 node commit/gossip/replication 执行哈希绑定并补齐测试
- [x] T2（P1-A）：实现 viewer live 共识执行高度门控（默认开启）并补齐测试
- [x] T3（P1-B）：新增 runtime 模块部署/安装动作闭环并补齐测试
- [x] T4：全量回归（`cargo check` + 相关测试）与文档收口

## 依赖
- T1 为 T2/T3 的可信基线（执行哈希绑定）。
- T2 依赖 `world_viewer_live` 与 node runtime 共享高度观测。
- T3 依赖 runtime 既有治理闭环（propose/shadow/approve/apply）。

## 状态
- 当前阶段：已完成（T0~T4 全部完成）。
- 阻塞项：无。
- 下一步：无（等待新需求）。
