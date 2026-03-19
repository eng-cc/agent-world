# oasis7：P2P/存储/共识在线长跑稳定性测试方案（2026-02-24）设计

- 对应需求文档: `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md`
- 对应项目管理文档: `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.project.md`

## 1. 设计定位
定义长跑、在线稳定性与 chaos 场景专题设计，统一 soak、反馈注入、复制网络和多节点稳定性验证。

## 2. 设计结构
- 场景编排层：定义长跑时长、节点拓扑、注入事件与脚本入口。
- 稳定性观测层：采集在线率、复制、存储与共识健康指标。
- chaos 注入层：按模板执行故障、反馈事件与恢复操作。
- 验收归档层：沉淀长跑结果、失败签名与门禁结论。

## 3. 关键接口 / 入口
- longrun/soak 脚本入口
- chaos 注入模板
- 稳定性指标与探针
- 长跑归档与失败签名

## 4. 约束与边界
- 长跑场景需可重复执行、可比较。
- chaos 注入必须与恢复策略成对设计。
- 不在本专题扩展新的线上编排平台。

## 5. 设计演进计划
- 先固定长跑场景与探针。
- 再补 chaos/反馈注入。
- 最后固化稳定性验收与归档。
