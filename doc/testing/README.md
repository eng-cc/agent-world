# testing 文档索引

审计轮次: 3

## 入口
- PRD: `doc/testing/prd.md`
- 项目管理: `doc/testing/prd.project.md`
- 文件级索引: doc/testing/prd.index.md

## 关键文档
- 系统测试手册：`testing-manual.md`
- 模块化测试细则：`doc/testing/manual/`
- CI 与门禁专题：`doc/testing/ci/`
- 启动器链路测试：`doc/testing/launcher/`
- 长稳与压力测试：`doc/testing/longrun/`、`doc/testing/performance/`
- 门禁策略与治理：`doc/testing/governance/`、`doc/testing/chaos-plans/`

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`prd.project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `ci/launcher/longrun/performance/manual/governance`。

## 维护约定
- 测试门禁变更需同步 required/full 分层口径与对应脚本。
