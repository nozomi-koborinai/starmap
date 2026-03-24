# starmap

Generate Awesome Lists from your GitHub Stars, organized by [GitHub Lists](https://docs.github.com/en/get-started/exploring-projects-on-github/saving-repositories-with-stars#organizing-starred-repositories-with-lists).

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
