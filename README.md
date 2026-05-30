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

## Custom Category Order

Create a `starmap.toml` at your repo root to control output order:

```toml
order = [
  "🌱 Focus: Learning",
  "🤖 AI Frameworks",
  "🦾 Agent Tools",
  # ...
  "🎉 Other",
]

[llms_full]
max_readme_size_kb = 10  # cap per repo (default: 10)
```

Lists not listed in `order` are appended at the end.

## llms.txt and llms-full.md

Generate AI-agent-friendly companion files:

```sh
starmap export-llms-txt llms.txt        # llmstxt.org-compliant index
starmap export-llms-full llms-full.md   # full README archive (truncated per repo)
```

`llms-full.md` fetches each repo's README via REST; expect ~2 minutes for several hundred stars.

## GitHub Token

starmap requires a GitHub token. Set it via environment variable or use `gh`:

```sh
# Option 1: environment variable
export GITHUB_TOKEN="ghp_..."

# Option 2: GitHub CLI (token is read automatically)
gh auth login
```

## Sync via GitHub Actions

To keep your published Awesome List in sync automatically, add this workflow to your **target repo** (e.g. `your-name/awesome-stars`) — not to the starmap repo:

```yaml
name: Update Awesome List
on:
  schedule:
    - cron: '0 0 * * *'  # daily at 00:00 UTC
  workflow_dispatch:

permissions:
  contents: write

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install --git https://github.com/nozomi-koborinai/starmap
      - run: starmap export README.md
        env:
          GITHUB_TOKEN: ${{ secrets.STARMAP_PAT }}
      - run: starmap export-llms-txt llms.txt
        env:
          GITHUB_TOKEN: ${{ secrets.STARMAP_PAT }}
      - run: starmap export-llms-full llms-full.md
        env:
          GITHUB_TOKEN: ${{ secrets.STARMAP_PAT }}
      - name: Commit if changed
        run: |
          if [[ -n "$(git status --porcelain)" ]]; then
            git config user.name "github-actions[bot]"
            git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
            git add README.md llms.txt llms-full.md
            git commit -m "chore: update Awesome List"
            git push
          fi
```

Add a classic Personal Access Token (with read access to your starred lists) as the `STARMAP_PAT` repo secret. The default `GITHUB_TOKEN` runs as `github-actions[bot]` and cannot see your stars.

## Example

See [koborin-ai/stars](https://github.com/koborin-ai/stars) for a live, auto-synced setup — `README.md`, `llms.txt`, and `llms-full.md` are all regenerated daily by starmap.

## License

MIT
