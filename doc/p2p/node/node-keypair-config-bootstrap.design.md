# Agent World Runtime：节点密钥 config.toml 自举设计

- 对应需求文档: `doc/p2p/node/node-keypair-config-bootstrap.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-keypair-config-bootstrap.project.md`

## 1. 设计定位
定义节点密钥通过 `config.toml` 完成自举的设计，统一密钥来源、加载顺序与启动校验。

## 2. 设计结构
- 配置加载层：从 `config.toml` 读取节点密钥或引用信息。
- 自举校验层：在启动前完成格式、存在性与权限校验。
- 身份注入层：把密钥注入 node/network/consensus 依赖路径。
- 故障提示层：对缺失、冲突与格式错误给出明确失败信号。

## 3. 关键接口 / 入口
- `config.toml` 密钥字段
- 节点启动自举入口
- 网络/共识身份注入点
- 配置错误提示

## 4. 约束与边界
- 密钥来源顺序必须确定，避免多入口冲突。
- 启动失败要给出明确诊断，不可静默生成替代身份。
- 不在本专题实现完整密钥托管系统。

## 5. 设计演进计划
- 先确定 config 自举字段。
- 再贯通启动与身份注入。
- 最后补齐失败场景回归。
