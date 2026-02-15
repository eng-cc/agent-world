# World Runtime：Builtin Wasm DistFS 存储与提交前校验

## 目标
- 将内置 builtin wasm 二进制从 git 跟踪中移出，避免仓库长期携带二进制大文件。
- 保留并强化可追踪的一致性基线：`module_id -> wasm_hash` 清单继续由 git 跟踪。
- 将 wasm 校验前移到提交前（pre-commit）执行，尽早拦截“源码已变更但 hash 未更新”问题。
- 运行时装载路径从 `include_bytes!` 切换为 DistFS 本地存储读取，并做 hash 校验。

## 范围
- In Scope：
  - `scripts/sync-m1-builtin-wasm-artifacts.sh` / `scripts/sync-m4-builtin-wasm-artifacts.sh` 改造为“构建 + hash 校验 + DistFS 落盘”。
  - `scripts/pre-commit.sh` 增加 builtin wasm 一致性校验步骤。
  - runtime builtin artifact 加载逻辑改造：按 hash 清单从 DistFS 本地目录读取。
  - `.gitignore` 更新与历史 wasm 文件移出 git 追踪。
- Out of Scope：
  - 远端 DistFS 分发、网络拉取、签名治理。
  - module hash 算法切换（仍使用现有 SHA-256 作为 wasm_hash）。

## 接口 / 数据
- git 跟踪：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_module_ids.txt`
- git 不跟踪：
  - `.distfs/builtin_wasm/blobs/<sha256>.blob`
- 脚本行为：
  - `--check`：构建 wasm，校验 built hash 与 git 清单一致，并将产物写入 DistFS 本地存储（用于后续测试运行时读取）。
  - 默认（非 `--check`）：构建 wasm，刷新 hash 清单并写入 DistFS 本地存储。
- 运行时行为：
  - 通过模块 id 查询 hash 清单。
  - 从 DistFS 本地目录读取 `<sha256>.blob`。
  - 读取后再次计算 SHA-256，必须与清单一致。

## 里程碑
- M1：文档与任务拆解完成。
- M2：脚本与 pre-commit 改造完成。
- M3：runtime 加载改造完成，移除 git 跟踪 wasm 二进制。
- M4：required tier 回归、文档与 devlog 收口。

## 风险
- 本地未执行同步脚本时，运行时读取 DistFS 可能报缺失。
- 运行时从文件读取替代静态嵌入后，错误从“编译期”变为“运行期”，需要清晰报错与提示。
- `--check` 带副作用（写入 DistFS 本地目录），需在文档中明确这是预期行为。
