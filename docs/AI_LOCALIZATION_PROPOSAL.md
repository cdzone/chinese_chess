# AI 本地化方案（现状说明）

## 文档信息

- **文档版本**: v1.1（scrub）
- **创建日期**: 2026-01-10
- **修订日期**: 2026-10-29
- **状态**: 核心已实施（本文档保留为设计对照，不再作「待讨论」提案）
- **目标**: 将 PvE 的 Alpha-Beta AI 迁到客户端本地运行；在线 AI 作备选

---

## 一、背景（历史）

早期 PvE 由服务端托管 AI（`chess-server` 内调用 `chess-ai`），存在算力占服务器、网络延迟、离线不可用等问题。本地化把搜索放到客户端，服务器只负责联机。

---

## 二、已落地（以代码为准）

### 2.1 模式与入口

| 能力 | 位置 |
|------|------|
| `GameMode::LocalPvE { difficulty }` | `chess-client/src/game/state.rs` |
| `ClientGame::start_local_pve` | 同上 |
| 菜单「单机对战」难度入口 | `chess-client/src/ui/menu.rs` |
| 本地悔棋 / 暂停 / 认输 | `chess-client/src/game/mod.rs`（`GameEvent`） |
| 本地存档读写 | `chess-client/src/storage.rs`、`ui/saved_games_ui.rs` |

`GameMode` 另含 `LocalPvP`、`OnlinePvE`、`OnlinePvP`；本说明只覆盖本地 AI 路径。

### 2.2 本地 AI 运行时

| 能力 | 位置 |
|------|------|
| `LocalAiPlugin`（注册于 `GamePlugin`） | `chess-client/src/game/ai.rs`、`game/mod.rs` |
| `AsyncComputeTaskPool` 后台搜索 | `ai.rs`：`trigger_ai_compute` / `poll_ai_result` |
| `AiEngine::from_difficulty` + `search_with_history` | `chess-ai`（客户端依赖见 `chess-client/Cargo.toml`） |
| 思考态 `AiThinkingState` + UI「AI 思考中...」 | `ai.rs`、`chess-client/src/ui/game_ui.rs` |
| AI 超时保护（设置项 `ai_timeout_secs`） | `ai.rs` + `settings` |

说明：提案草稿曾规划单独的 `local_ai.rs`；实现合并进 `game/ai.rs`，**无** `local_ai.rs`。

### 2.3 依赖

```toml
# chess-client/Cargo.toml（已存在）
chess-ai = { path = "../chess-ai" }
```

默认不启用 LLM。可选 feature `llm`（`chess-ai/llm`）走本机 Ollama 等复盘/分析路径，**不属于**本 LocalPvE 搜索主路径；许可说明见根目录 `THIRD_PARTY.md`。

---

## 三、相对原提案的差异

| 原提案说法 | 现状 |
|------------|------|
| 状态「待讨论」、讨论议题「是否立即实施」 | 主路径已合入；议题作废 |
| 新增 `game/local_ai.rs` | 未拆文件，逻辑在 `game/ai.rs` |
| 统一 `AiProvider` trait / 远程 AI 抽象 | **未做**；本地与在线仍分支处理 |
| Phase 3：运行时改难度、统计面板、AI 提示 | **非目标**（未作为交付项） |
| 运营成本美元测算、天梯防作弊长文 | **删除**；本仓为学习/沙盒玩具，不作运营口径 |

在线 PvE（`OnlinePvE`）代码仍保留为备选，是否继续维护不在本文范围。

---

## 四、非目标（明确不做或未承诺）

- Steam / 安装包 / 上架相关打包
- 把服务端 AI 从代码库物理删除
- 强制默认关闭在线 AI
- 将 `llm` feature 捆绑进默认 LocalPvE
- 跨设备云同步单机进度

---

## 五、相关路径速查

**已实现（读这些即可）**

- `chess-client/src/game/state.rs`
- `chess-client/src/game/ai.rs`
- `chess-client/src/game/mod.rs`
- `chess-client/src/ui/menu.rs`
- `chess-client/src/ui/game_ui.rs`
- `chess-ai/`（引擎本体）

**历史对照（服务端托管 AI，仍可能用于 OnlinePvE）**

- `chess-server/src/server.rs`（AI 房间逻辑）

**可选 LLM（独立）**

- `chess-ai/src/llm/`
- `chess-client` feature `llm`

---

## 六、维护约定

- 改 LocalPvE 行为时优先改 §2 所列文件，并同步本节「已落地」表。
- 勿把本文再改回「待讨论」；未做项写入「非目标」或另开短提案。
