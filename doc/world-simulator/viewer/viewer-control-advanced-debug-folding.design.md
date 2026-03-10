# Viewer 控制区高级调试折叠设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-control-advanced-debug-folding.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-control-advanced-debug-folding.project.md`

## 1. 设计定位
定义控制区从多按钮常驻模式收敛到“播放/暂停”主按钮 + “高级调试”折叠区的交互结构，降低日常 chat 驱动玩法下的误触和认知负担。

## 2. 设计结构
- 主控制层：将 `播放/暂停` 收敛为单一切换按钮。
- 高级调试层：把 `单步` 与 `跳转 0` 收入可展开区域。
- 本地状态层：维护“当前是否播放”和“是否展开高级调试”的 UI 状态。
- 协议复用层：继续走 `ViewerControl::{Play,Pause,Step,Seek}` 协议，不改底层控制语义。

## 3. 关键接口 / 入口
- `ViewerControl::{Play, Pause, Step, Seek}`
- `播放/暂停` 单按钮
- `高级调试` 开关
- 控制区本地 UI 状态

## 4. 约束与边界
- 不修改 viewer 协议与 live server 行为。
- 折叠只影响可见性与默认布局，不影响调试动作可达性。
- 单按钮文案和发送动作必须严格匹配本地运行态。
- 本轮不做大规模控制区信息架构重构。

## 5. 设计演进计划
- 先建立单按钮切换和折叠状态。
- 再补本地运行态更新与测试。
- 最后更新手册并固定简化控制区的日常使用口径。
