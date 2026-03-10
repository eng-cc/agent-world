# 分布式能力彻底拆分 Phase 7 设计

- 对应需求文档: `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
- 对应项目管理文档: `doc/p2p/distributed/distributed-hard-split-phase7.project.md`

## 1. 设计定位
定义分布式能力从 `agent_world` 向 `agent_world_net / consensus / distfs / proto` 四个基础 crate 的彻底拆分方案，使 `agent_world` 仅保留世界内核与模拟层。

## 2. 设计结构
- crate 归位层：net/consensus/distfs/proto 承担分布式主线实现。
- facade 收敛层：`agent_world` 移除分布式导出，只保留领域边界。
- 协议迁移层：Viewer 协议迁入 `agent_world_proto`。
- 工程治理层：WASM ABI 边界收敛与超长文件拆分并行推进。

## 3. 关键接口 / 入口
- `agent_world_net` / `agent_world_consensus` / `agent_world_distfs` / `agent_world_proto`
- Viewer 协议模块
- WASM ABI 类型来源
- 超长文件拆分目标

## 4. 约束与边界
- 不新增新的分布式协议能力。
- 拆分目标是归位与边界清晰，不是算法改写。
- 大规模路径调整必须伴随分批编译和定向测试。
- server/viewer 协议迁移要保证双端版本一致。

## 5. 设计演进计划
- 先落 distfs crate 与路径切换。
- 再收敛 facade 和 viewer 协议。
- 最后处理 WASM ABI 边界与超长文件拆分。
