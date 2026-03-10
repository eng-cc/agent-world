# Fix Pre-commit（预提交失败修复脚本）设计

- 对应需求文档: `doc/scripts/precommit/precommit-remediation-playbook.prd.md`
- 对应项目管理文档: `doc/scripts/precommit/precommit-remediation-playbook.project.md`

## 1. 设计定位
定义 precommit / 提交前门禁脚本专题设计，统一本地检查、修复流程与 required 门禁口径。

## 2. 设计结构
- 本地门禁层：定义提交前执行的格式化、测试与治理检查。
- 修复辅助层：提供失败后的 remediation/playbook 脚本入口。
- 口径对齐层：将本地 precommit 与 CI required 门禁对齐。
- 维护回写层：沉淀脚本更新与失败签名。

## 3. 关键接口 / 入口
- pre-commit 脚本入口
- remediation/playbook 工具
- required 门禁矩阵
- 失败签名与维护说明

## 4. 约束与边界
- 本地门禁不得与 CI 口径长期漂移。
- 修复脚本必须服务可重复故障。
- 不在本专题扩展新的 CI 平台。

## 5. 设计演进计划
- 先冻结 precommit 范围。
- 再补修复/对齐链路。
- 最后固化维护说明。
