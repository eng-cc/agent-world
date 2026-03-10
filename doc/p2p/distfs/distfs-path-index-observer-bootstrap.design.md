# Agent World Runtime：Observer/Bootstrap 路径索引读取接入设计

- 对应需求文档: `doc/p2p/distfs/distfs-path-index-observer-bootstrap.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-path-index-observer-bootstrap.project.md`

## 1. 设计定位
定义 Observer/Bootstrap 路径索引读取接入设计，让路径索引成为 observer 与 bootstrap 获取 DistFS 路径信息的统一入口。

## 2. 设计结构
- 路径索引层：提供可读取的 DistFS 路径索引与元数据。
- observer 接入层：让 observer 同步策略可直接消费路径索引。
- bootstrap 接入层：在启动/追平阶段复用同一索引来源。
- 一致性校验层：保证索引更新、读取与缺失处理可审计。

## 3. 关键接口 / 入口
- 路径索引读取入口
- observer 路径消费点
- bootstrap 路径消费点
- 索引一致性校验

## 4. 约束与边界
- 路径索引必须作为统一入口，避免多份真相。
- 索引缺失或损坏时需要明确失败信号。
- 不在本专题重构完整 DistFS 元数据系统。

## 5. 设计演进计划
- 先接 observer 路径读取。
- 再贯通 bootstrap 消费路径。
- 最后固化索引校验与回归。
