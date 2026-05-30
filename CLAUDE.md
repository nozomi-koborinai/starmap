# CLAUDE.md

## About

starmap is a Rust CLI tool that generates Awesome Lists from GitHub Stars, organized by GitHub Star Lists. Unlike language/topic-based auto-categorization, starmap respects the user's manual curation via GitHub Lists.

## Structure

| Directory | Purpose |
|-----------|---------|
| `src/main.rs` | CLI entry point (clap derive) |
| `src/github/client.rs` | GitHub GraphQL API client with cursor-based pagination |
| `src/github/types.rs` | Domain types + raw GraphQL response types + conversion |
| `src/generator/markdown.rs` | Awesome List Markdown generation |
| `src/commands/show.rs` | `starmap` — stdout output |
| `src/commands/export.rs` | `starmap export <path>` — file output |
| `src/commands/push.rs` | `starmap push --repo <owner/name>` — push to GitHub repo |

## Commands

```sh
cargo run                              # show (stdout)
cargo run -- export awesome-list.md    # export to file
cargo run -- push --repo owner/name    # push to GitHub repo
cargo run -- export-llms-txt llms.txt       # generate llmstxt.org index
cargo run -- export-llms-full llms-full.md  # generate README archive (slow: fetches per repo)
```

## Development

```sh
cargo fmt -- --check    # format check
cargo clippy -- -D warnings  # lint (must pass with zero warnings)
cargo test              # run tests
cargo build --release   # release build
```

All four checks must pass locally before pushing.

## Conventions

- Comments and documentation in English
- Error handling with `anyhow`
- GitHub API: GraphQL only (REST API does not support Star Lists)
- Authentication: `GITHUB_TOKEN` env var, falls back to `gh auth token`
