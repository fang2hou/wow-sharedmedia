# Publishing Guide

This document describes the release workflow for `wow-windmedia`.

The goal is a release process that feels consistent with mature Rust libraries: explicit versioning, repeatable validation, and safe crates.io publication.

## 1. Repository Layout

This guide assumes `wow-windmedia` is the repository root and that `Cargo.toml`, `src/`, `.github/`, `templates/`, and the repository policy files live alongside each other.

## 2. Pre-Release Checklist

Before cutting a release, make sure all of the following are true:

- `Cargo.toml` version is correct
- `README.md` reflects the current public API
- CI is green on the target branch
- `cargo publish --dry-run` succeeds locally

## 3. Local Validation

Run the same checks maintainers expect in CI:

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy -p wow-windmedia --all-targets -- -D warnings
cargo test -p wow-windmedia --lib
RUSTDOCFLAGS="-D warnings" cargo doc -p wow-windmedia --no-deps
cargo publish --dry-run
```

For commit-policy validation from the repository root:

```bash
prek run --all-files --stage pre-commit
printf 'feat: verify commit policy\n' > .git/COMMIT_EDITMSG
prek run conventional-commit-msg --stage commit-msg --files .git/COMMIT_EDITMSG --commit-msg-filename .git/COMMIT_EDITMSG
```

## 4. Versioning

Until `1.0.0`, treat releases conservatively:

- use patch releases for fixes and small polish
- use minor releases for meaningful API changes
- avoid unnecessary public API churn between releases

## 5. Release Flow

Recommended release order:

1. Update version in `Cargo.toml`
2. Commit the release changes
3. Create and push a version tag such as `v0.1.0`
4. Run the GitHub `Release` workflow or publish locally with `cargo publish`
5. Verify crates.io page renders correctly
6. Verify docs.rs builds successfully
7. Create a short GitHub release summary for the published version

## 6. GitHub Actions

The repository includes two workflows:

- `CI` — formatting, linting, cross-platform library tests, docs, MSRV, and package dry run
- `Release` — manually triggered publication workflow guarded by a crates.io token

The release workflow expects:

- a configured repository environment named `release`
- a repository secret named `CARGO_REGISTRY_TOKEN`

## 7. crates.io Publication

Local publication:

```bash
cargo login
cargo publish -p wow-windmedia
```

If publishing through GitHub Actions, use the manual `Release` workflow after verifying the requested version matches `Cargo.toml`.

## 8. Post-Release Verification

After a release:

- confirm the crate appears on crates.io
- confirm docs.rs built the crate successfully
- update README badges if crates.io/docs.rs badges are introduced later
- create a short GitHub release summary for the published version

## 9. Practical Maintainer Advice

- Keep release commits small and boring
- Avoid mixing refactors with release prep
- Prefer manual release approval over fully automatic publish-on-tag for early releases
- Treat the public API as expensive to change, even before `1.0.0`
