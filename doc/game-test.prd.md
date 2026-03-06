# Legacy Redirect: game-test.prd

审计轮次: 4

## 目标
- 将根目录 `doc/game-test.prd.md` 的 redirect 入口迁移到 `.prd` 命名，保持入口语义一致。
- 明确活跃主入口为 `doc/playability_test_result/game-test.prd.md`。

## 范围
- In Scope:
  - 根目录 redirect 入口维护与路径指引。
  - 与模块主入口的一致性说明。
- Out of Scope:
  - 不承载真实玩家测试流程正文。
  - 不替代 `doc/playability_test_result/` 下的活跃手册。

## 接口 / 数据
- 根目录 redirect：`doc/game-test.prd.md`
- 根目录项目管理 redirect：`doc/game-test.prd.project.md`
- 活跃主入口：`doc/playability_test_result/game-test.prd.md`
- 活跃项目管理：`doc/playability_test_result/game-test.prd.project.md`

## 里程碑
- M1（2026-03-03）：根目录 game-test redirect 文档切换到 `.prd` 命名。

## 风险
- 旧路径残留引用可能导致跳转断链，需要在迁移提交内一并回写。
