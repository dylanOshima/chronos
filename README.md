# Chronos

A CLI for managing cron jobs and one-off scheduled tasks. Understands natural language schedules like "every sunday at 6am" or "tomorrow at 1am" and works as a friendly interface over your system's `crontab` and `at`.

## Install

```bash
cargo install chronos
```

Or build from source:

```bash
git clone <repo-url>
cd chronos
cargo build --release
```

## Usage

### Adding jobs

Chronos auto-detects whether a schedule is recurring or one-off:

```bash
# Recurring jobs (writes to crontab)
chronos add "every sunday at 6am" "backup.sh" --id weekly-backup
chronos add "daily at 8am" "claude -p 'write a daily brief'" --id daily-brief --source claude
chronos add "every weekday at 9:30am" "open https://standup.example.com"
chronos add "every 2 hours" "curl https://healthcheck.example.com/ping"
chronos add "0 8 * * *" "echo hello"  # raw cron expressions work too

# One-off jobs (uses at, or self-destructing cron fallback)
chronos add "tomorrow at 1am" "claude -p 'retry the deploy'" --id retry-deploy
chronos add "sunday 6pm" "remind.sh 'weekly review'"
chronos add "march 31 at noon" "run-migration.sh"
```

Options:
- `--id <name>` -- human-friendly identifier for referencing the job later
- `--desc <text>` -- description stored alongside the job
- `--source <name>` -- who scheduled it (defaults to `$USER`, pass `claude` for AI-scheduled jobs)
- `--force` -- skip duplicate detection warning

### Listing jobs

```bash
chronos list          # table of all jobs (recurring + one-off)
chronos list --json   # machine-readable output
```

```
 # | ID            | Schedule              | Command                    | Source
 1 | daily-brief   | At 08:00 AM           | claude -p 'daily brief'    | claude
 2 | retry-deploy  | Mar 31 at 12:00pm     | claude -p 'retry deploy'   | droshima
 3 | ---           | At 02:00 AM, on Sunday | /usr/local/bin/backup.sh   | unknown
```

Jobs added outside Chronos show up with source `unknown`. Disabled jobs are included in the listing.

### Searching

```bash
chronos search "backup"          # fuzzy search across id, description, command, source
chronos search "claude" --json
```

### Removing jobs

```bash
chronos remove daily-brief   # by id
chronos remove 2             # by row number from list output
```

### Enabling / disabling jobs

Disable comments out the crontab entry. Enable uncomments it. The job stays in your crontab either way.

```bash
chronos disable weekly-backup
chronos enable weekly-backup
```

## How it works

Chronos has no job store. Your system `crontab` and `at` queue are the source of truth -- Chronos reads them on every invocation.

A lightweight metadata file at `~/.config/chronos/meta.toml` (respects `$XDG_CONFIG_HOME`) stores optional id, description, and source for jobs added through Chronos. If this file is missing or stale, everything still works -- you just lose the extra metadata.

When `at` is not available on the system, one-off jobs fall back to a self-destructing cron pair that removes itself after firing.

## Example: Claude scheduling

Have Claude schedule a command to run later:

```bash
chronos add "tomorrow at 6am" "claude -p 'summarize overnight logs'" --id morning-logs --source claude
```

Check what Claude has scheduled:

```bash
chronos search "claude"
```
