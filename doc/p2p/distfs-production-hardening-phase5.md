# Agent World Runtime：DistFS 生产化增强（Phase 5）设计文档

## 目标
- 把 DistFS challenge probe 从硬编码参数升级为运行时可配置参数，支持按网络状态调优挑战强度。
- 将 reward runtime 中 DistFS probe 逻辑模块化，降低主文件复杂度并保持单文件行数安全。
- 增强挑战可观测性：在 epoch 报告中输出 probe 配置与累计 cursor 状态快照。

## 范围

### In Scope
- **DPH5-1：文档与任务拆解**
  - 输出 Phase 5 设计文档与项目管理文档。

- **DPH5-2：probe 参数治理化与模块化拆分**
  - 新增 DistFS probe runtime 子模块，承载：
    - probe 配置结构与默认值；
    - probe 状态加载/持久化；
    - probe 执行函数。
  - `world_viewer_live` CLI 增加参数：
    - `--reward-distfs-probe-max-sample-bytes`
    - `--reward-distfs-probe-per-tick`
    - `--reward-distfs-probe-ttl-ms`
    - `--reward-distfs-probe-allowed-clock-skew-ms`
  - 增加参数校验与单元测试。

- **DPH5-3：可观测增强**
  - 在 reward runtime epoch 报告中新增：
    - 当前 probe 配置；
    - probe cursor 状态快照（累计 total/pass/fail 与 failure reasons）。
  - 补充对应测试。

- **DPH5-4：回归与收口**
  - 执行 DistFS 与 world_viewer_live 相关回归。
  - 回写项目状态与 devlog。

### Out of Scope
- 链上治理参数动态下发。
- 跨进程集中式 challenge coordinator。
- PoRep/PoSt 协议级升级。

## 接口 / 数据

### CLI 参数（草案）
```text
--reward-distfs-probe-max-sample-bytes <u32, >0>
--reward-distfs-probe-per-tick <u32, >0>
--reward-distfs-probe-ttl-ms <i64, >0>
--reward-distfs-probe-allowed-clock-skew-ms <i64, >=0>
```

### runtime 配置（草案）
```rust
DistfsProbeRuntimeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
}
```

### 报告新增字段（草案）
```json
{
  "distfs_probe_config": { ... },
  "distfs_probe_cursor_state": { ... },
  "distfs_challenge_report": { ... }
}
```

## 里程碑
- **DPH5-M1**：文档与任务拆解完成。
- **DPH5-M2**：参数治理化 + 模块化拆分完成。
- **DPH5-M3**：报告可观测增强完成。
- **DPH5-M4**：回归与文档收口完成。

## 风险
- 参数暴露后配置错误风险上升；通过强校验与默认值兜底控制。
- 模块拆分若边界不清会导致重复逻辑；通过单一入口函数约束。
- 报告字段增多会增加 I/O 体积；当前字段规模可接受。
