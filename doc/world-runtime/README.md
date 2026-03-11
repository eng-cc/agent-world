# world-runtime 文档索引

审计轮次: 6

## 入口
- PRD: `doc/world-runtime/prd.md`
- 设计总览: `doc/world-runtime/design.md`
- 标准执行入口: `doc/world-runtime/project.md`
- 文件级索引: `doc/world-runtime/prd.index.md`

## 模块职责
- 维护运行时主链路、存储治理、WASM 执行与模块发布口径。
- 汇总 runtime / wasm / module / governance / integration / testing 六类专题。
- 承接候选级证据、发布门禁指标与跨模块 runtime 收口事项。

## 主题文档
- `runtime/`：运行时主链路、数值正确性、存储治理与长稳专题。
- `wasm/`：WASM 接口、执行器、SDK 与沙箱治理。
- `module/`：模块生命周期、发布合法性、存储与订阅过滤专题。
- `governance/`：治理、审计与收据安全专题。
- `integration/`：跨模块桥接专题。
- `testing/`：运行时专用测试分册。

## 近期专题
- `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.prd.md`
- `doc/world-runtime/module/online-module-release-legality-closure-2026-03-08.prd.md`
- `doc/world-runtime/module/player-published-entities-2026-03-05.prd.md`
- `doc/world-runtime/governance/zero-trust-governance-receipt-hardening-2026-02-26.prd.md`
- `doc/world-runtime/wasm/wasm-agent-os-alignment-hardening.prd.md`

## 根目录收口
- 模块根目录主入口保留：`README.md`、`prd.md`、`design.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `runtime/`、`wasm/`、`module/`、`governance/`、`integration/`、`testing/`。

## 根目录 legacy
- `doc/world-runtime.prd.md`
- `doc/world-runtime.project.md`

上述两个根目录文件仅保留为兼容跳转入口；当前主入口以本目录 `prd.md` / `project.md` 为准。

## 维护约定
- runtime 行为、发布门禁或候选级证据口径变化时，优先回写 `doc/world-runtime/prd.md`。
- 新增专题后，需同步回写 `doc/world-runtime/prd.index.md` 与本目录索引。
