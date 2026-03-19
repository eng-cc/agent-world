# oasis7 Runtime：Node DistFS 复制网络化收敛设计

- 对应需求文档: `doc/p2p/node/node-distfs-replication-network-closure.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-distfs-replication-network-closure.project.md`

## 1. 设计定位
定义 Node 与 DistFS 复制网络的闭环收敛设计，统一节点复制请求、存储交互和网络摄取边界。

## 2. 设计结构
- 复制网络层：统一 node replication 与 distfs 数据交换通道。
- 存储接入层：把复制结果映射到本地/远端存储状态。
- 校验摄取层：对复制块、元数据和来源执行完整性校验。
- 运行闭环层：把复制网络结果反馈到节点运行时与观测面。

## 3. 关键接口 / 入口
- replication network 请求入口
- DistFS 存储交互接口
- 复制校验/摄取入口
- 节点运行时反馈面

## 4. 约束与边界
- 复制语义需与 DistFS 数据一致性口径对齐。
- 校验失败数据不得进入本地状态。
- 不在本专题引入新的存储后端类型。

## 5. 设计演进计划
- 先收敛复制网络接口。
- 再对齐存储与摄取校验。
- 最后通过闭环回归固化。
