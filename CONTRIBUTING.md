# Contributing

> Thank you for considering contributing!

Contributions of any size are welcome. Before starting work on a major feature, please open an Issue to discuss it first. This ensures your work aligns with the project's direction and saves everyone time.

## Prerequisites

- Rust stable
- Cargo
- Git

## Building

```bash
cargo build
```

## Running tests

```bash
cargo test
```

## Formatting

```bash
cargo fmt
```

## Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Commit messages

Follow Conventional Commits.

Examples:

- feat(parser): add Pratt parser
- fix(ast): handle empty blocks
- docs: improve README

## Pull Requests

Before opening a PR, make sure:

- [ ] Tests pass
- [ ] Code is formatted
- [ ] Clippy reports no warnings
- [ ] Documentation is updated

## Code style

- Keep functions small.
- Prefer readability over cleverness.
- Avoid unnecessary allocations.
- Document every public API.

## Code of Conduct
> By participating in this project, you agree to abide by our Code of Conduct.