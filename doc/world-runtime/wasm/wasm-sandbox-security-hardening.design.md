# oasis7 Runtime：WASM 沙箱安全补强（设计文档）设计

- 对应需求文档: `doc/world-runtime/wasm/wasm-sandbox-security-hardening.prd.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-sandbox-security-hardening.project.md`

## 1. 设计定位
定义 WASM 沙箱安全补强设计，收敛权限面、资源隔离与高风险宿主调用防护。

## 2. 设计结构
- 权限最小化层：限制模块可见 host 能力与资源访问。
- 隔离保护层：对内存、文件、网络与系统交互实施隔离。
- 异常阻断层：对越权、逃逸尝试和资源滥用建立阻断。
- 审计验证层：输出安全事件与沙箱回归证据。

## 3. 关键接口 / 入口
- 沙箱权限配置
- host capability allowlist
- 安全事件记录
- 沙箱回归用例

## 4. 约束与边界
- 安全边界优先于功能便利性。
- 高风险宿主能力必须显式 allowlist。
- 不在本专题引入完整外部 TEE。

## 5. 设计演进计划
- 先收敛权限面。
- 再补隔离与阻断。
- 最后固化安全回归。
