# bbpr2md (Bitbucket Pull Request to Markdown)

`bbpr2md` is a Rust CLI tool designed to fetch pull request comments and tasks from Bitbucket Cloud and format them into a clean Markdown document. This is primarily used as high-quality context for AI agents (like Claude or Gemini) to understand and address PR feedback.

## Zero-Config Usage (Git Remote Auto-Detection)

If your current directory is a git repository whose `origin` remote points to Bitbucket, `bbpr2md` can derive everything it needs automatically:

```bash
# On a branch that has an open PR — no flags required
bbpr2md
```

It will:
1. Parse `workspace` and `repo_slug` from `git remote get-url origin`
2. Read the current branch via `git branch --show-current`
3. Query the Bitbucket API for an open PR from that branch

Use `--remote <name>` to specify a different remote than `origin`.

You still need to supply credentials (see [Authentication](#authentication) below).

## Configuration File Flow

For repositories you access frequently, a config file avoids repeating credentials and defaults.

### 1. Initialize Configuration
```bash
bbpr2md --init
```

The command prints the path to the created file (e.g., `~/.config/bbpr2md/default-config.toml`).

### 2. Configure Defaults
```toml
username = "your.email@example.com"
token = "your-bitbucket-personal-access-token"
workspace = "your-workspace"   # optional if using git remote auto-detection
repo_slug = "your-repo"        # optional if using git remote auto-detection
```

### 3. Usage
With a config file, credentials are handled automatically and the PR is still auto-detected:

```bash
bbpr2md            # auto-detect PR from current branch
bbpr2md --pr-id 123  # or specify explicitly
```

## AI Agent Skill Installation

`bbpr2md` can install a native skill into Claude or Gemini CLI so those agents know how to use the tool automatically. The skill content is embedded in the binary, so this works from any install method (Homebrew, direct download, etc.).

### Claude

```bash
# Install into the current project's .claude/skills/
bbpr2md skill claude

# Install globally into ~/.claude/skills/
bbpr2md skill claude --global

# Skip the confirmation prompt
bbpr2md skill claude --global --yes
```

### Gemini CLI

```bash
# Install into the current project's .gemini/skills/
bbpr2md skill gemini

# Install globally into ~/.gemini/skills/
bbpr2md skill gemini --global --yes
```

Once installed, the agent will automatically know how to invoke `bbpr2md` to gather PR context.

## AI Agent Workflow

The primary goal of `bbpr2md` is to simplify the feedback loop between code review and implementation.

### The Agent Cycle
1.  **Gather Context**: An AI agent runs `bbpr2md` (or `bbpr2md --pr-id <ID>`) to retrieve all comments and tasks.
2.  **Analyze**: The agent parses the Markdown to identify requested changes, grouped by file and line.
3.  **Implement**: The agent applies the changes to the codebase.
4.  **Verify**: The agent uses the tasks list to ensure all items are addressed before marking them as resolved.

## Command Line Arguments

`bbpr2md` supports overrides for all configuration values via CLI flags:

-   `--pr-id <ID>`: The pull request ID. If omitted, auto-detected from the current branch (see [Zero-Config Usage](#zero-config-usage-git-remote-auto-detection)).
-   `--workspace <WS>`: Bitbucket workspace ID. If omitted, parsed from the git remote URL.
-   `--repo-slug <REPO>`: Repository slug. If omitted, parsed from the git remote URL.
-   `--remote <NAME>`: Git remote to use for URL parsing (default: `origin`).
-   `--username <USER>`: Bitbucket username/email.
-   `--token <TOKEN>`: Personal Access Token (Bearer Auth).
-   `--app-password <PW>`: App Password (Basic Auth).
-   `--output <FILE>`: Save Markdown to a file instead of printing to stdout.
-   `--debug`: Print debug information (like requested URLs).
-   `--description-only`: Include only the pull request description.
-   `--comments-only`: Include only the comments.
-   `--tasks-only`: Include only the tasks.
-   `--comments-and-tasks`: Include only comments and tasks (exclude description).
-   `--comment <ID_OR_URL>`: Show only the thread containing a specific comment. Accepts a raw comment ID (e.g. `789186489`) or a full Bitbucket comment URL (e.g. `https://bitbucket.org/org/repo/pull-requests/1379#comment-789186489`).
-   `--list-users`: List all unique users who have commented on the pull request, one per line in the form `Display Name (account_id)`.
-   `--author <USER>`: Show only comments authored by the given user. Matches against display name or account ID (case-insensitive). Works alongside `--comments-only`, `--comment`, and other flags.
-   `--hide-resolved`: Exclude comment threads that have been marked as resolved in Bitbucket. Entire threads (root + all replies) are hidden when the root is resolved.

## Environment Variables

You can also use environment variables, which override the config file but are overridden by CLI flags:
-   `BITBUCKET_USERNAME`
-   `BITBUCKET_TOKEN`
-   `BITBUCKET_APP_PASSWORD`

## Examples

The [examples/](./examples) directory contains:
-   `config.toml`: A sample configuration file.
-   `claude_skill.md` / `gemini_skill/SKILL.md`: The skill definitions (use `bbpr2md skill` to install these rather than copying manually).

## Development

Managed via `just`:
-   `just build`: Build the release binary.
-   `just test`: Run the test suite.
-   `just check`: Run linting and formatting checks.
