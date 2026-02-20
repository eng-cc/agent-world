# Agent World Runtime：PoS 历史设计文档归档收敛（2026-02-20）

## 目标
- 将已经完成且被后续实现替代的早期 Node PoS 分阶段设计文档归档到 `doc/p2p/archive`。
- 保留历史追溯能力，同时避免团队把阶段性文档当作当前实现基线。
- 保持 `doc/p2p` 活跃目录中的 PoS 文档口径与当前代码一致。

## 范围

### In Scope
- 归档以下 3 组 PoS 早期文档及其项目管理文档：
  - `distributed-node-mainloop*`
  - `distributed-node-pos-mainloop*`
  - `distributed-node-pos-gossip*`
- 在归档文档文件头补充过期警示块与归档日期。
- 修复仓库内对上述文档的路径引用（仅路径层面，不重写历史内容）。
- 更新本轮项目管理文档与当日 devlog。

### Out of Scope
- 修改任何运行时代码或共识语义。
- 重写历史方案细节或变更原任务结论。
- 大规模重构其他主题文档。

## 接口 / 数据
- 归档目标目录：`doc/p2p/archive/`
- 归档警示模板：
  - `> [!WARNING]`
  - `> 该文档已过期，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-20`
- 当前 PoS 基线文档：
  - `doc/p2p/consensus-code-consolidation-to-agent-world-consensus.md`
  - `doc/p2p/consensus-code-consolidation-to-agent-world-consensus.project.md`

## 里程碑
- PDA-1：新增归档收口设计/项目管理文档。
- PDA-2：完成 3 组历史 PoS 文档迁移到 `doc/p2p/archive` 并加过期警示。
- PDA-3：完成引用路径修复、回归检查、devlog 收口与提交。

## 风险
- 若遗漏引用修复，会产生活跃文档断链。
- 若误归档仍在迭代中的文档，会影响当前设计基线。
- 归档后需确保团队入口文档仍指向当前有效基线。
