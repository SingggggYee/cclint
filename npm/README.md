# cclint

Token-cost linter for Claude Code configs. Audits your `CLAUDE.md`, `settings.json`, skills, and commands, finds token waste, and suggests concrete fixes with a 0-100 health score.

```bash
npm install -g cclint
cclint
```

This package downloads the prebuilt Rust binary for your platform (macOS or Linux, arm64/x64) from [GitHub releases](https://github.com/SingggggYee/cclint/releases). No Rust toolchain needed.

## What it checks

- `CLAUDE.md` rules that burn tokens on every session
- Unused or overlapping skills and commands
- settings.json misconfigurations
- Overall config health score (80-100 is good)

## Other install options

```bash
cargo install cclint                 # via crates.io
brew install SingggggYee/tap/cclint  # via Homebrew
```

Works with [ccwhy](https://github.com/SingggggYee/ccwhy): ccwhy tells you where tokens went, cclint prevents the waste up front.

Full docs: [github.com/SingggggYee/cclint](https://github.com/SingggggYee/cclint)

MIT
