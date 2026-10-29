# chinese_chess

学习用 Bevy 中国象棋玩具：可单机 LocalPvE（本地 AI），可选本机联机（默认端口 `9527`）。**不是** Steam / 上架目标；维护口径见群内沙盒约定。

许可：**MIT**（根目录 `LICENSE`）。捆绑字体与第三方摘要见 `THIRD_PARTY.md`。

## 依赖

- Rust（建议 stable）
- 客户端需要本机图形环境（Bevy 0.17）

## 测试

```bash
cargo test -p protocol -p chess-ai -p chess-server --lib
```

只跑各 crate 的 lib 测；`examples/`（如过期 `test_analysis`）不进本闸。走法排序等重性能测试若本地另加，请标 `#[ignore]`，不要放进 CI 必跑。

## 运行

单机客户端（含 LocalPvE）：

```bash
cargo run -p chess-client
```

可选本机服务端（监听 `0.0.0.0:9527`；当前端口写死在代码里，未做环境变量外置）：

```bash
cargo run -p chess-server
```

客户端默认连 `127.0.0.1:9527`（设置里可改地址）。

仅检查客户端能否编译（CI 用）：

```bash
cargo check -p chess-client
```

## Workspace 成员

| 包 | 作用 |
|----|------|
| `protocol` | 共享协议与棋盘 |
| `chess-ai` | AI 搜索 / 评估（可选 `llm` feature） |
| `chess-server` | TCP 对局服 |
| `chess-client` | Bevy 客户端 |

## CI

GitHub Actions：`cargo test --lib`（protocol / chess-ai / chess-server）+ `cargo check -p chess-client`。不编 examples、不打包、不发服、不跑 Steam。
