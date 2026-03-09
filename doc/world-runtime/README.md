# world-runtime 文档索引

审计轮次: 6

## 入口
- PRD: `doc/world-runtime/prd.md`
- 设计总览: `doc/world-runtime/design.md`
- 标准执行入口: `doc/world-runtime/project.md`
- 兼容执行入口: `doc/world-runtime/project.md`
- 文件级索引: doc/world-runtime/prd.index.md

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
