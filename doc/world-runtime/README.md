# world-runtime 文档索引

审计轮次: 6

## 入口
- PRD: `doc/world-runtime/prd.md`
- 设计总览: `doc/world-runtime/design.md`
- 标准执行入口: `doc/world-runtime/project.md`
- 文件级索引: `doc/world-runtime/prd.index.md`

## 主题文档
- `runtime/`：运行时主链路与数值/稳定性专题。
- `wasm/`：WASM 接口、执行器与沙箱治理。
- `module/`：模块生命周期、存储、订阅与治理。
- `governance/`：治理、审计与收据安全专题。
- `integration/`：跨模块桥接专题。
- `testing/`：运行时专用测试分册。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `runtime/wasm/module/governance/integration/testing`。

## 根目录 legacy
- `doc/world-runtime.prd.md`
- `doc/world-runtime.project.md`

上述两个根目录文件仅保留为兼容跳转入口；当前主入口以本目录 `prd.md`/`project.md` 为准。

- `doc/world-runtime/governance/zero-trust-governance-receipt-hardening-2026-02-26.design.md`
- `doc/world-runtime/module/agent-default-modules.design.md`
- `doc/world-runtime/module/module-storage.design.md`
- `doc/world-runtime/runtime/bootstrap-power-modules.design.md`
- `doc/world-runtime/module/module-subscription-filters.design.md`
- `doc/world-runtime/module/online-module-release-legality-closure-2026-03-08.design.md`
- `doc/world-runtime/module/player-published-entities-2026-03-05.design.md`
- `doc/world-runtime/runtime/runtime-infinite-sequence-rollover.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase1.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase10.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase11.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase12.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase13.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase14.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase15.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase2.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase3.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase4.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase5.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase6.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase7.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase8.design.md`
- `doc/world-runtime/runtime/runtime-numeric-correctness-phase9.design.md`
- `doc/world-runtime/wasm/wasm-agent-os-alignment-hardening.design.md`
- `doc/world-runtime/wasm/wasm-executor.design.md`
- `doc/world-runtime/wasm/wasm-sandbox-security-hardening.design.md`
- `doc/world-runtime/wasm/wasm-sdk-no-std.design.md`
- `doc/world-runtime/wasm/wasm-sdk-wire-types-dedup.design.md`
