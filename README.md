# starmap

Generate Awesome Lists from your GitHub Stars, organized by [GitHub Lists](https://docs.github.com/en/get-started/exploring-projects-on-github/saving-repositories-with-stars#organizing-starred-repositories-with-lists).

## When to use starmap

starmap is for people who actively maintain GitHub Lists and want them mirrored as-is in a published Awesome List — no auto-grouping by language or topic, no algorithmic guessing. Your manual curation is the source of truth.

If you don't use GitHub Lists, [maguowei/starred](https://github.com/maguowei/starred) is the mature, widely-used tool in this space and a better fit — it auto-categorizes your stars by language or topic out of the box.

## Focus Lists

Lists whose name starts with `Focus: ` (e.g. `🔥 Focus: In Production`, `🌱 Focus: Watching`) are rendered differently from topic lists:

- They do **not** become Markdown sections
- Each repo that belongs to a Focus List gets the Focus name appended as an inline tag (`` `🔥 In Production` ``)
- A "Focus" legend at the top of the output explains each tag, using each Focus List's GitHub description

Use Focus Lists to express orthogonal axes (e.g. "what I actually use" vs "what I'm watching") without disturbing your topic-based categorization.

## Install

```sh
cargo install --git https://github.com/nozomi-koborinai/starmap
```

## Usage

Show your starred repositories grouped by lists:

```sh
starmap
```

Export to a Markdown file:

```sh
starmap export awesome-list.md
```

Push directly to a GitHub repository:

```sh
starmap push --repo owner/repo-name
```

## GitHub Token

starmap requires a GitHub token. Set it via environment variable or use `gh`:

```sh
# Option 1: environment variable
export GITHUB_TOKEN="ghp_..."

# Option 2: GitHub CLI (token is read automatically)
gh auth login
```

## License

MIT
