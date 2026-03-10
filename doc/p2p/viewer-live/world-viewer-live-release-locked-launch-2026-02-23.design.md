# world_viewer_live 发行锁定启动（P2P）设计文档（2026-02-23）设计

- 对应需求文档: `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.project.md`

## 1. 设计定位
定义 `world_viewer_live` 发行锁定启动设计，确保 P2P 发行版本的启动参数、能力开关与锁定口径保持一致。

## 2. 设计结构
- 发行锁定层：定义 release locked 启动配置与不可变默认值。
- 能力裁剪层：只暴露发行允许的能力与入口。
- 启动校验层：在启动时校验锁定配置、环境与依赖。
- 发布收口层：把锁定策略同步到发布文档与回归。

## 3. 关键接口 / 入口
- release locked 启动配置
- 能力开关白名单
- 启动校验入口
- 发行回归用例

## 4. 约束与边界
- 锁定版本不得被非预期参数绕开。
- 能力裁剪需与发布口径一致。
- 不在本专题扩展新的发行渠道。

## 5. 设计演进计划
- 先冻结锁定启动策略。
- 再补校验与能力裁剪。
- 最后回写发布与回归文档。
