> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# P2P 文档过期性核查报告（2026-02-16）

## 核查目标
- 对 `doc/p2p` 中首轮迁移后的文档做二轮“文档-代码一致性”核查。
- 识别技术口径已过期文档并归档到 `doc/p2p/archive`。

## 核查范围
- `doc/p2p` 下非 `archive` 的 `.md` 文档。
- 核查时间：2026-02-16。

## 核查方法
1. 提取文档中的反引号路径引用（`crates/`、`doc/`、`scripts/`、`tools/`、绝对路径）。
2. 将引用路径与当前仓库文件做存在性比对。
3. 对命中以下规则的文档判定为“过期候选”：
   - 依赖已删除核心路径（重点：`crates/agent_world/src/runtime/distributed*`、`crates/agent_world/src/runtime/distributed_membership_sync*`）。
   - 且该能力已在 split crate 中落地（`agent_world_consensus` / `agent_world_net` / `agent_world_distfs`）。
4. 对候选文档的设计文档与项目管理文档成对归档，防止“设计/项目文档状态不一致”。

## 核查结果
- 首轮迁移总数：132。
- 二轮追加归档：74。
- 当前归档总数：79。
- 当前活跃文档：53。

## 归档结论
- 以 `distributed-consensus*` 与 `distributed-consensus-membership*` 为主的一批文档，虽然历史上有效，但其代码依赖点已从 `agent_world` 运行时目录迁移/拆分，原文档技术落点过期，已归档。
- 归档文档均补充了统一过期标识：
  - `> [!WARNING]`
  - `> 该文档已过期，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-16`

## 保留说明
- 保留文档主要为当前节点主循环、PoS 接线、DistFS 路径索引、runtime bridge 等仍可对应现有 crate 结构的文档。
- 个别历史路径以“历史说明”形式保留（非当前依赖）。

## 追踪文件
- 最终迁移映射：`doc/p2p/archive/migration-map.md`
- 治理设计文档：`doc/p2p/p2p-doc-consolidation.md`
- 治理项目文档：`doc/p2p/p2p-doc-consolidation.project.md`
