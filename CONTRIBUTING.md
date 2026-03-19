# Contributing to wow-windmedia

Thanks for your interest in improving `wow-windmedia`.

This project aims to be a polished, predictable Rust library for managing World of Warcraft SharedMedia assets. Contributions are welcome, but changes should preserve the crate's small public surface, stateless design, and release quality.

## Before You Start

- Read `README.md` for project goals and supported scope
- Read `PUBLISHING.md` if your change affects release behavior

## Development Principles

Please keep changes aligned with these principles:

- **Stateless by design** — no hidden runtime state or background synchronization
- **`data.lua` is the source of truth** — avoid introducing parallel metadata stores
- **WoW-compatible outputs** — generated assets should remain practical for real addon usage
- **Small, stable API surface** — avoid exposing internal helpers without a strong reason
- **Clear failure modes** — prefer explicit errors over silent fallback behavior

## Setup

Use the pinned toolchain:

```bash
rustup show
cargo check
```

Recommended validation commands:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo doc --no-deps
```

If you edit files under `templates/`, it is also helpful to run:

```bash
stylua templates/*.lua
```

## Commit Convention and Hooks

The repository uses **Conventional Commits**.

Examples:

- `feat: add BLP smoke tests`
- `fix: sync generated file version metadata`
- `docs: refine publishing guide`
- `test: add Lua 5.1 loader runtime coverage`

Recommended local setup from the repository root:

```bash
cargo install --locked cocogitto
winget install --id j178.Prek --exact
prek install --hook-type pre-commit --hook-type commit-msg --hook-type pre-push
```

This crate keeps its hook and commit configuration in:

- `prek.toml`
- `cog.toml`

## Pull Request Expectations

Please keep pull requests focused and reviewable.

Good pull requests usually:

- explain the problem being solved
- describe the chosen approach and tradeoffs
- include tests for behavior changes
- update docs when the public API or workflows change
- avoid unrelated cleanup in the same patch

## Commit and Change Quality

Before opening a PR, make sure:

- the crate builds cleanly
- tests pass locally
- public-facing changes are documented
- new files and docs use professional English

## API Changes

For changes that affect the public API, please be extra conservative.

In particular:

- avoid adding public modules or functions unless necessary
- avoid locking in awkward APIs that will be expensive to support after `0.1.0`
- prefer additive changes over breaking changes where practical

## Documentation Changes

Docs should be concise, professional, and easy to scan.

- README tone should stay polished and release-oriented
- Emoji are welcome, but should be used sparingly and deliberately
- Usage examples should be realistic and minimal

## Reporting Issues

If you are reporting a bug, please include:

- the crate version
- your Rust version
- your operating system
- the asset type and input format involved
- a minimal reproduction, if possible

## Conduct

By participating in this project, you agree to follow the expectations in `CODE_OF_CONDUCT.md`.
