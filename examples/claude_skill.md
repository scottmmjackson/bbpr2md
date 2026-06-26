---
name: bbpr2md
description: Fetch and format Bitbucket pull request descriptions, comments, and tasks as Markdown. Use to gather PR feedback, review changes, or track open tasks.
---
# Bitbucket PR Context Tool (bbpr2md)

**Version**: 0.1.10

The `bbpr2md` tool allows you to retrieve a structured Markdown representation of a Bitbucket pull request.

### Abilities
- **Zero-Config PR Detection**: When run inside a git repository whose `origin` remote points to Bitbucket, `bbpr2md` automatically derives the workspace, repo slug, and PR ID — no flags needed on a branch with an open PR.
- **Description Retrieval**: Fetch the PR title and description, quoted to avoid Markdown header confusion.
- **Comment Aggregation**: Collect all non-deleted comments, grouped by file and sorted by line number.
- **Task Tracking**: List all PR tasks with their current state (UNRESOLVED/RESOLVED) and creator.
- **Selective Fetching**: Use flags like `--description-only`, `--comments-only`, or `--tasks-only` to limit output.
- **Single Thread**: Use `--comment <ID_OR_URL>` to fetch only the thread containing a specific comment.
- **List Commenters**: Use `--list-users` to get a list of all unique users who commented on the PR.
- **Author Filter**: Use `--author <USER>` to show only comments from a specific reviewer (matches display name or account ID).
- **Hide Resolved**: Use `--hide-resolved` to suppress comment threads that have been resolved in Bitbucket, reducing noise.
- **Inline Tasks**: Comment-linked tasks are rendered directly beneath their parent comment; only PR-level tasks appear in the global Tasks section.

### Usage

Minimal invocation (inside a git repo on a branch with an open PR):
```
bbpr2md
```

With an explicit PR ID:
```
bbpr2md --pr-id 123
```

With a non-default remote:
```
bbpr2md --remote upstream
```

### Key Flags
- `--pr-id <ID>` — PR ID (auto-detected from current branch if omitted)
- `--workspace <WS>` — Bitbucket workspace (auto-detected from git remote if omitted)
- `--repo-slug <REPO>` — Repository slug (auto-detected from git remote if omitted)
- `--remote <NAME>` — Git remote to use for auto-detection (default: `origin`)
- `--description-only` / `--comments-only` / `--tasks-only` / `--comments-and-tasks`
- `--comment <ID_OR_URL>` — Fetch only the thread containing a specific comment
- `--list-users` — List all unique commenters (output: `Display Name (account_id)`, one per line)
- `--author <USER>` — Filter output to comments from this user (display name or account ID, case-insensitive)
- `--hide-resolved` — Suppress resolved comment threads (entire thread hidden when root is resolved)

### Use Cases
- **Feedback Analysis**: Quickly see all reviewer comments across multiple files in a single view.
- **Change Planning**: Use the aggregated comments and line numbers to map out required code changes.
- **Task Verification**: Ensure all Bitbucket tasks are completed by referencing the task list.
- **Context Gathering**: Provide a concise summary of a PR's purpose and discussion for further analysis.
- **Reviewer Discovery**: Run `--list-users` to find who has commented, then use their name with `--author` to focus on one reviewer's feedback at a time.

### Maintenance & Updates
If this skill's version does not match the `bbpr2md` tool version, you should update your instructions by fetching the latest version of this file from the GitHub repository: `https://github.com/scottmmjackson/bbpr2md/blob/master/examples/claude_skill.md`.
