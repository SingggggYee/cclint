# cclint

> ccwhy tells you where tokens went. cclint tells you what to fix in your config.

Lint your Claude Code setup. Finds token waste in your CLAUDE.md, hooks, skills, and commands. Gives you a health score and actionable fixes.

## Example Output

```
  cclint — Claude Code Config Linter
  Find what's wasting tokens in your config.

  Config Health Score: 90/100

  [WARN] ~/.claude/skills/ 32 skills installed. Each skill's description is loaded into context.
         fix: Remove skills you don't use. Each unused skill wastes ~50-200 tokens per session.
         impact: ~1600-6400 tokens from skill descriptions alone
  [WARN] ~/.claude/commands/ 42 commands configured. Commands are listed in context when relevant.
         fix: Remove commands you don't use
  [INFO] ~/.claude/CLAUDE.md CLAUDE.md size OK (~416 tokens, 45 lines)
  [INFO] settings.json skipDangerousModePermissionPrompt is enabled
         fix: This skips safety confirmations. Make sure you understand the risks.
```

## Install

```bash
cargo install cclint
```

Or build from source:

```bash
git clone https://github.com/SingggggYee/cclint
cd cclint
cargo build --release
./target/release/cclint
```

## What It Checks

### CLAUDE.md
- File size and estimated token cost per API call
- Redundant verbosity rules (already Claude's default behavior)
- Contradictory rules
- Duplicate section headers
- Vague rules that waste tokens ("write clean code", "follow best practices")
- Very long lines (likely copy-pasted content)
- Missing structure (no headers)

### settings.json
- Total hook count and latency impact
- PreToolUse hooks without matcher (runs on every tool call)
- Dangerous permission settings

### Skills
- Total skill count and token impact from descriptions
- Skill directories without SKILL.md
- Follows symlinks (handles linked skill installations)

### Commands
- Total command count
- Oversized command files

## Usage

```bash
# Lint current directory + global config
cclint

# Lint specific project
cclint /path/to/project

# Skip global config
cclint --global false

# JSON output
cclint --json
```

## Health Score

100 = no issues. Each error costs 15 points, each warning costs 5 points.

| Score | Meaning |
|-------|---------|
| 80-100 | Good. Minor optimizations possible. |
| 50-79 | Needs attention. Likely wasting tokens. |
| 0-49 | Fix immediately. Significant token waste. |

## Works with ccwhy

Use together for full usage optimization:
- [ccwhy](https://github.com/SingggggYee/ccwhy) — tells you where tokens went (past usage)
- **cclint** — tells you what to fix in your config (prevent future waste)

## License

MIT
