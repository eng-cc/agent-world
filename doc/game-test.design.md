# Legacy Redirect: game-test.design

- 对应需求文档: `doc/game-test.prd.md`
- 对应项目管理文档: `doc/game-test.project.md`

## 1. 设计定位
定义根目录 `game-test` redirect 入口设计，保证历史路径统一指向活跃的 `playability_test_result` 主入口。

## 2. 设计结构
- redirect 入口层：保留根目录历史路径，承接旧引用。
- 主入口指向层：明确唯一业务主入口，避免正文散落在 redirect 文件。
- 兼容维护层：在迁移期间维持旧路径可达与说明可读。

## 3. 关键接口 / 入口
- redirect PRD：`doc/game-test.prd.md`
- redirect Project：`doc/game-test.project.md`
- 当前主入口：`doc/playability_test_result/game-test.prd.md` / `doc/playability_test_result/game-test.project.md`

## 4. 约束与边界
- redirect 文档只承担导航与兼容说明，不承载业务设计正文。
- 历史路径与主入口映射必须稳定、单向且可追溯。
- 不在本专题扩展额外归档结构。

## 5. 设计演进计划
- 先补齐 redirect design 与互链。
- 再持续清理仓内旧路径引用。
- 最终将历史入口压缩为长期兼容层。
