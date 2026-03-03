# world-runtime 文档索引

## 入口
- PRD: `doc/world-runtime/prd.md`
- 项目管理: `doc/world-runtime/prd.project.md`

## 主题文档
- `runtime/`：运行时主链路与数值/稳定性专题。
- `wasm/`：WASM 接口、执行器与沙箱治理。
- `module/`：模块生命周期、存储、订阅与治理。
- `governance/`：治理、审计与收据安全专题。
- `integration/`：跨模块桥接专题。
- `testing/`：运行时专用测试分册。
- `archive/`：历史运行时演进文档。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`prd.project.md`。
- 其余专题文档按主题下沉到 `runtime/wasm/module/governance/integration/testing`。

## 根目录 legacy
- `doc/world-runtime.md`
- `doc/world-runtime.project.md`
- 历史完整总览归档：`doc/archive/root-history/world-runtime-root-entry-legacy-2026-03-03.prd.md`
- 历史完整项目归档：`doc/archive/root-history/world-runtime-root-entry-legacy-2026-03-03.prd.project.md`

上述两个根目录文件仅保留为兼容跳转入口；当前主入口以本目录 `prd.md`/`prd.project.md` 为准。
