# Third-party notices

This repository’s own code is licensed under the MIT License (see `LICENSE`).
This file lists **bundled third-party materials** that are not covered solely
by that MIT grant, plus a short note on Rust crate dependencies.

## Bundled assets

### Source Han Sans SC (思源黑体 简体)

| Item | Detail |
|------|--------|
| Files | `chess-client/assets/fonts/SourceHanSansSC-Regular.otf`, `chess-client/assets/fonts/SourceHanSansSC-Bold.otf` |
| Upstream | [adobe-fonts/source-han-sans](https://github.com/adobe-fonts/source-han-sans) (Simplified Chinese OTF) |
| License | SIL Open Font License 1.1 |
| Full text | [`chess-client/assets/fonts/OFL.txt`](chess-client/assets/fonts/OFL.txt) |
| Copyright (upstream) | Copyright 2014–2025 Adobe (http://www.adobe.com/), with Reserved Font Name `Source`. |

Summary of OFL obligations relevant to redistributing this game (not legal advice):

- You may embed and redistribute these fonts **with** the software.
- You must keep the copyright notice and OFL text available to recipients (this file + `chess-client/assets/fonts/OFL.txt` satisfy that for source trees; release packages should ship the same).
- Do **not** sell the font files by themselves.
- Do **not** rename a modified font using the reserved name `Source` without Adobe’s written permission.

Verified against Adobe’s upstream `LICENSE.txt` on the `release` branch
(https://raw.githubusercontent.com/adobe-fonts/source-han-sans/release/LICENSE.txt),
2026-10-29 (local sha256 of `OFL.txt` matches that file).

## Engine and crate dependencies

| Component | License (declared) | Notes |
|-----------|--------------------|-------|
| [Bevy](https://bevy.org/) `0.17` (via Cargo) | Dual **MIT OR Apache-2.0** | Engine and most of its crates; some nested crates may carry additional BSD-like notices—see each crate README / `cargo license` output when preparing a binary release. |
| Other crates in `Cargo.lock` | Various permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib, …) | Workspace `Cargo.toml` sets this project’s code to MIT. A full machine-readable inventory for a given lockfile can be regenerated with `cargo install cargo-license && cargo license` (or equivalent). |

This learning project does **not** currently vendor a complete copy of every dependency’s license text. Before any commercial redistribution (e.g. Steam), regenerate and archive a lockfile-tied notice list; also re-check for copyleft (GPL/AGPL/LGPL) crates if Steamworks SDK is linked (Steamworks documentation warns against combining copyleft with the SDK).

## What is not listed here

- Original game code, docs, and non-font assets authored in this repository → `LICENSE` (MIT).
- Optional LLM features (`chess-ai` `llm` feature / local Ollama, etc.) may pull extra crates when enabled; re-run a license inventory if that feature is shipped.
