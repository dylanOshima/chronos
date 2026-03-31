# Chronos — Design Spec

A Rust CLI for managing cron jobs and one-off scheduled tasks. Provides a human-friendly interface over system `crontab` and `at`, with natural language schedule parsing. Designed for both human users and programmatic use by Claude.

## Architecture

### No Store — System as Source of Truth

Chronos has no job store. The system `crontab` and `at` queues are the source of truth. Every command reads directly from the system:

- **Recurring jobs:** Parsed from `crontab -l` output.
- **One-off jobs:** Parsed from `atq` + `at -c <job_id>` output.
- **Adding a recurring job:** Read current crontab, append entry, write back via `crontab`.
- **Adding a one-off:** Pipe to `at`. If `at` is unavailable, fall back to self-destructing cron pair.
- **Removing:** Filter the entry out of crontab, or `atrm` for one-offs.

### Metadata Sidecar

A lightweight TOML file at `~/.config/chronos/meta.toml` (respecting `$XDG_CONFIG_HOME`) stores optional metadata for jobs added through Chronos. This is supplementary — if the file is missing or stale, everything still works; you just lose descriptions and source info.

```toml
[cron."0 8 * * * claude -p 'Write a daily brief'"]
id = "daily-brief"
description = "Morning brief from Claude"
source = "claude"

[at.42]
id = "rate-limit-retry"
description = "Retry deploy after rate limit resets"
source = "droshima"
```

- Cron entries keyed by the full crontab line (schedule + command).
- At entries keyed by `at` job number.
- Stale entries pruned silently on each `list` invocation.
- Only jobs added through Chronos get sidecar entries. Pre-existing entries show as `unknown` source.

## CLI Interface

### Adding Jobs

```
# Recurring — detected from "every", "daily", "weekday", or cron expression
chronos add "every sunday at 6am" "command"
chronos add "daily at midnight" "command"
chronos add "0 8 * * *" "command"

# One-off — detected from specific date/time
chronos add "tomorrow at 1am" "command"
chronos add "sunday 6pm" "command"
chronos add "march 31 at noon" "command"
```

Optional flags:
- `--id <name>` — human-friendly identifier for referencing the job later.
- `--desc <description>` — description stored in sidecar.
- `--source <name>` — who scheduled it (defaults to `$USER`).

### Schedule Parsing

The parser auto-detects recurring vs one-off:

- Contains "every", "daily", "weekday", or is a valid cron expression → **recurring** → `crontab`.
- Resolves to a specific point in time → **one-off** → `at` (or self-destructing cron fallback).

If ambiguous, error with a suggestion: `"Could not parse schedule. Did you mean 'every sunday at 6pm' (recurring) or 'this sunday at 6pm' (once)?"`

Implementation uses Rust crates for natural date/time parsing (e.g. `chrono-english` or `two-timer`), with a thin layer to handle "every X" patterns and convert to cron expressions.

### Listing Jobs

```
chronos list          # All jobs, both recurring and one-off
chronos list --json   # Machine-readable output
```

Table output (human-readable schedules, no cron expressions):

```
#   ID            Schedule              Command                    Source
1   daily-brief   Every day at 8:00am   claude -p 'daily brief'    claude
2   retry-deploy  Mar 31 at 12:00pm     claude -p 'retry deploy'   droshima
3   —             Every Sunday at 2am   /usr/local/bin/backup.sh   unknown
```

### Other Commands

```
chronos remove <identifier>          # By sidecar id or row number from list output
chronos enable <identifier>          # Uncomment crontab line
chronos disable <identifier>         # Comment out crontab line (# prefix)
chronos search <query>               # Fuzzy search across id, description, command
chronos search <query> --json
```

## One-Off Fallback: Self-Destructing Cron Pair

When `at` is not available, one-off jobs are implemented as a self-destructing cron pair:

1. A cron entry for the target time runs the command via an internal wrapper: `chronos _run-once <id> -- <original-command>`
2. A cleanup cron entry fires one minute later: `chronos remove <id> && chronos remove <cleanup-id>`

The `_run-once` internal subcommand:
1. Executes the original command.
2. Removes both the job and its cleanup entry from crontab.
3. Cleans up sidecar metadata.

The cleanup cron is a fallback — the primary job self-removes on first successful run. This guards against the edge case where the machine was off when the cleanup job was supposed to fire.

## Error Handling

- **`at` not available:** Detected at runtime. Falls back to self-destructing cron pair. No degradation in user experience.
- **Ambiguous schedule:** Error with suggested alternatives (see Schedule Parsing above).
- **Stale sidecar entries:** Pruned silently on each `list` invocation.
- **Duplicate detection:** Warn if adding a job with identical schedule + command. Allow with `--force`.
- **Permission issues:** Surface the underlying system error clearly.

## Output Formats

- **Default:** Human-friendly table with columns: ID, Schedule (human-readable), Command, Source.
- **`--json`:** Machine-readable JSON array, available on `list` and `search`.

## Testing

- **Unit tests:** Schedule parser (cron expressions, natural language → cron, natural language → datetime). Crontab parsing (read/write, preserving non-Chronos entries). Sidecar read/write/prune.
- **Integration tests:** Use mock crontab file — never touch real system crontab in CI. Self-destructing cron pair lifecycle.

## Distribution

- Publish to crates.io.
- Pre-built binaries via GitHub releases (Linux + macOS, x86_64 + aarch64).
- Homebrew formula for macOS.
