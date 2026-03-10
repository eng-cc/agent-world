# readme 文档索引

审计轮次: 6

## 入口
- PRD: `doc/readme/prd.md`
- 设计总览: `doc/readme/design.md`
- 标准执行入口: `doc/readme/project.md`
- 兼容执行入口: `doc/readme/project.md`
- 文件级索引: doc/readme/prd.index.md

## 模块职责
- 统一仓库对外说明口径与文档入口。
- 跟踪 README 与设计/实现的一致性缺口。

## 主题文档
- `gap/`：README 与实现/流程间差距闭环。
- `production/`：生产口径与发布收口专题。
- `governance/`：规则层与资源模型治理专题。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `gap/production/governance`。

## 维护约定
- 对外口径变更需同步 `README.md` 与本模块文档。
- `doc/readme/gap/readme-gap-distributed-prod-hardening-gap12345.design.md`
- `doc/readme/gap/readme-gap-infra-exec-compiler-sandbox.design.md`
- `doc/readme/gap/readme-gap-wasm-live-persistence-instance-upgrade.design.md`
- `doc/readme/gap/readme-gap12-consensus-market-lifecycle-closure.design.md`
- `doc/readme/gap/readme-gap12-market-closure.design.md`
- `doc/readme/gap/readme-gap123-runtime-consensus-metering.design.md`
- `doc/readme/gap/readme-gap2-llm-wasm-lifecycle.design.md`
- `doc/readme/gap/readme-gap3-install-target-infrastructure.design.md`
- `doc/readme/gap/readme-gap34-lifecycle-orderbook-closure.design.md`
- `doc/readme/governance/readme-resource-model-layering.design.md`
- `doc/readme/governance/readme-world-rules-consolidation.design.md`
- `doc/readme/production/readme-llm-p1p2-production-closure.design.md`
- `doc/readme/production/readme-p0-p1-closure.design.md`
- `doc/readme/production/readme-prod-closure-llm-distfs-consensus.design.md`
- `doc/readme/production/readme-prod-gap1245-wasm-repl-topology-player.design.md`
