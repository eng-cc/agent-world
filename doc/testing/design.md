# testing 模块设计总览

审计轮次: 6

- 对应需求文档: `doc/testing/prd.md`
- 对应项目管理文档: `doc/testing/project.md`
- 对应文件级索引: `doc/testing/prd.index.md`

## 1. 设计定位
`testing` 模块的 `design.md` 负责描述测试分层、验证策略、证据采集与发布门禁的总体设计。

## 2. 阅读顺序
1. `doc/testing/prd.md`
2. `doc/testing/design.md`
3. `doc/testing/project.md`
4. `doc/testing/prd.index.md`
5. 下钻 `ci/`、`governance/`、`launcher/`、`longrun/`、`performance/` 等专题目录

## 3. 设计结构
- 分层层：`test_tier_required` / `test_tier_full` 的职责分工。
- 证据层：测试结果、失败签名、门禁与复审记录。
- 发布层：go/no-go、回归范围与阻断结论。

## 4. 集成点
- `testing-manual.md`
- `doc/playability_test_result/prd.md`
- `doc/core/prd.md`
- `doc/scripts/prd.md`

## 5. 专题导航
- CI 与覆盖进入 `ci/`
- 线上/长时验证进入 `longrun/`
- 发行与治理验证进入 `governance/`、`launcher/`

## 目标
- 提供 `testing` 模块的总体设计入口。

## 范围
- 覆盖模块级结构、主链路、分层与专题导航。
- 不替代专题 `*.design.md` 的细化设计。

## 接口 / 数据
- 需求入口：`doc/testing/prd.md`
- 执行入口：`doc/testing/project.md`
- 兼容执行入口：`doc/testing/project.md`
- 索引入口：`doc/testing/prd.index.md`

## 里程碑
- M1 (2026-03-09): 在 ROUND-006 中补齐模块级 `design.md` 标准入口。
- M2: 按专题继续补齐高复杂度主题的 `*.design.md`。

## 风险
- 若专题级设计未及时补齐，模块级 `design.md` 可能承载过多导航职责。
- 若 legacy `*.project.md` 长期保留，执行入口会继续双轨并存。
