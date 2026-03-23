# wow-windmedia

[![CI](https://github.com/wind-addons/wow-windmedia/actions/workflows/ci.yml/badge.svg)](https://github.com/wind-addons/wow-windmedia/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust 1.94+](https://img.shields.io/badge/rust-1.94.0+-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)

`wow-windmedia` is a Rust library for building and maintaining World of Warcraft SharedMedia addons.

It manages `data.lua`, generates `loader.lua` and `!!!WindMedia.toc`, converts supported media formats into WoW-compatible outputs, and keeps the addon directory in a consistent state through a small stateless API.

## 📦 Installation

```toml
[dependencies]
wow-windmedia = "0.1"
```

**Prerequisites:** Rust 1.94+ (edition 2024).

## 🚀 Quick Start

```rust
use std::path::Path;

use wow_windmedia::{
    ensure_addon_dir, import_media, read_data, ImportOptions, MediaType,
};

fn main() -> Result<(), wow_windmedia::Error> {
    let addon_dir = Path::new("AddOns/!!!WindMedia");
    ensure_addon_dir(addon_dir)?;

    let source = Path::new("assets/my-statusbar.png");
    let result = import_media(
        addon_dir,
        ImportOptions::new(MediaType::Statusbar, "My Statusbar", source),
    )?;

    println!("Imported {} as {}", result.entry.key, result.entry.file);

    let data = read_data(addon_dir)?;
    println!("{} entries registered", data.entries.len());

    Ok(())
}
```

## 🧩 Supported Media Types

| Media type   | Accepted input                                   | Stored output      |
| ------------ | ------------------------------------------------ | ------------------ |
| `statusbar`  | `.tga`, `.png`, `.webp`, `.jpg`, `.jpeg`, `.blp` | `.tga`             |
| `background` | `.tga`, `.png`, `.webp`, `.jpg`, `.jpeg`, `.blp` | `.tga`             |
| `border`     | `.tga`, `.png`, `.webp`, `.jpg`, `.jpeg`, `.blp` | `.tga`             |
| `font`       | `.ttf`, `.otf`                                   | original font file |
| `sound`      | `.ogg`, `.mp3`, `.wav`                           | `.ogg`             |

## 🧭 Design

The crate treats `data.lua` as the single source of truth.

Every write operation follows the same model:

1. Ensure the addon directory and static templates exist
2. Read the current registry state from `data.lua`
3. Apply the requested mutation
4. Write the updated registry back to disk

This keeps the runtime model small, deterministic, and easy to integrate into higher-level tools.

## 🗂️ Addon Layout

```text
!!!WindMedia/
├── !!!WindMedia.toc
├── data.lua
├── loader.lua
├── libraries/
│   ├── LibStub/LibStub.lua
│   ├── CallbackHandler-1.0/CallbackHandler-1.0.lua
│   └── LibSharedMedia-3.0/
│       ├── LibSharedMedia-3.0.lua
│       └── lib.xml
└── media/
    ├── background/
    ├── border/
    ├── font/
    ├── sound/
    └── statusbar/
```

## 📦 Vendor Management

Vendor libraries (LibSharedMedia-3.0, Serpent) are gitignored and regenerated via a Bun script.

```bash
# Requires: bun, svn
bun install
bun run update-vendor
```

This runs `svn export` for libsharedmedia-3.0 and fetches serpent from GitHub. The `vendor/` directory must exist before building (Rust embeds files via `include_str!`).

## 🛠️ Development

### Prerequisites

| Tool      | Purpose                          |
| --------- | -------------------------------- |
| Rust 1.94 | Build and test                   |
| Bun       | Vendor script and JS toolchain   |
| SVN       | Vendor download (libsharedmedia) |

### Setup

```bash
bun install
bun run update-vendor
```

### Checks

```bash
cargo fmt --all --check
cargo clippy -p wow-windmedia --all-targets -- -D warnings
cargo test -p wow-windmedia
cargo doc -p wow-windmedia --no-deps
bun run lint
bun run format:check
stylua --check templates/*.lua
```

### Pre-commit Hooks

```bash
# Requires: cocogitto, prek, stylua
cargo install --locked cocogitto
prek install --hook-type pre-commit --hook-type commit-msg --hook-type pre-push
```

## 📚 Documentation

- Contributor guidance: [`CONTRIBUTING.md`](./CONTRIBUTING.md)
- Publishing workflow: [`PUBLISHING.md`](./PUBLISHING.md)

## 📄 License

[`MIT LICENSE`](./LICENSE).
