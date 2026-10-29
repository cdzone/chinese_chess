# 第三方组件与许可说明

本仓库自身代码以根目录 [`LICENSE`](LICENSE)（MIT）授权，与 `Cargo.toml` 中 `workspace.package.license = "MIT"` 一致。

本文件列出**随本仓库分发或运行时捆绑**的、不属于本项目原创代码的重要第三方材料。学习沙盒阶段先覆盖发行包必带项；完整 Rust crate 许可树可用 `cargo license` / `cargo deny` 另做审计（未纳入本文件正文）。

## 1. 捆绑字体：Source Han Sans SC（思源黑体 简体）

| 项 | 说明 |
| --- | --- |
| 文件 | `chess-client/assets/fonts/SourceHanSansSC-Regular.otf`、`SourceHanSansSC-Bold.otf` |
| 来源 | [adobe-fonts/source-han-sans](https://github.com/adobe-fonts/source-han-sans) |
| 许可 | **SIL Open Font License 1.1**（OFL-1.1） |
| 版权 | Copyright 2014–2025 Adobe；保留字号 **Source**（Reserved Font Name） |
| 全文 | 见同目录 [`chess-client/assets/fonts/OFL.txt`](chess-client/assets/fonts/OFL.txt)（与上游 `LICENSE.txt` 一致） |

要点（非替代全文）：

- 字体可与软件一并捆绑、嵌入、再分发；**不得单独出售字体本身**。
- 衍生字体不得使用保留字号「Source」等，且须继续以 OFL 发布。
- 用该字体排版产生的**文档/画面本身**不必再以 OFL 授权。

发行或打包客户端时，请保留上述 otf 与 `OFL.txt`，不要只拷贝字体文件而丢掉许可文本。

## 2. 游戏引擎与 Rust 依赖（摘要）

客户端依赖 **Bevy 0.17**（及传递依赖如 `wgpu`、`winit` 等）。Bevy 工程本身为 **MIT OR Apache-2.0** 双许可（见 [bevyengine/bevy](https://github.com/bevyengine/bevy)）。本仓库 `Cargo.lock` 中其余 crate 多为 MIT / Apache-2.0 / BSD / Zlib 等宽松许可；**当前学习维护不在此逐条粘贴数百条 crate 许可全文**。

若日后做正式发行或上架审计，建议在 CI 增加 `cargo deny`（或等价）并生成完整第三方清单；发现 GPL/AGPL 等强互惠许可再单独处理。

## 3. 可选功能：LLM（`chess-client` feature `llm`）

启用 `--features llm` 时，客户端可调用本机 **Ollama** 等外部服务做复盘。Ollama 与其模型权重**不随本仓库分发**；使用者须自行遵守所选模型与运行时的许可，本仓不代为捆绑。

## 4. 未纳入本清单的内容

- 用户自备的棋谱存档、截图等运行时数据。
- 仅开发时拉取、未打进客户端资源包的工具链本身。
