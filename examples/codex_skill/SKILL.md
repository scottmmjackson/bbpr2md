---
name: bbpr2md
description: Fetch and format Bitbucket pull request descriptions, comments, and tasks as Markdown. Use to gather PR feedback, review changes, or track open tasks.
---
# Bitbucket PR Context Tool (bbpr2md)

**Version**: 0.1.13

Use `bbpr2md` to retrieve a structured Markdown representation of a Bitbucket pull request.

## Capabilities

- **Zero-config PR detection**: Inside a git repository whose `origin` remote points to Bitbucket, `bbpr2md` derives the workspace, repository slug, and PR ID automatically.
- **Description retrieval**: Fetch the PR title and description, quoted to avoid Markdown header confusion.
- **Comment aggregation**: Collect all non-deleted comments, grouped by file and sorted by line number.
- **Task tracking**: List all PR tasks with their current state (`UNRESOLVED` or `RESOLVED`) and creator.
- **Selective fetching**: Limit output to the description, comments, tasks, comments and tasks, or one comment thread.
- **Reviewer filtering**: List commenters, select one author, or hide resolved threads.

## Usage

Run the tool without arguments inside a repository on a branch with an open PR:

```bash
bbpr2md
```

Specify a PR ID or non-default remote when auto-detection is not appropriate:

```bash
bbpr2md --pr-id 123
bbpr2md --remote upstream
```

## Key flags

- `--pr-id <ID>`: PR ID; auto-detected from the current branch when omitted.
- `--workspace <WS>`: Bitbucket workspace; auto-detected from the git remote when omitted.
- `--repo-slug <REPO>`: Repository slug; auto-detected from the git remote when omitted.
- `--remote <NAME>`: Git remote used for auto-detection; defaults to `origin`.
- `--description-only`, `--comments-only`, `--tasks-only`, `--comments-and-tasks`: Select report sections.
- `--comment <ID_OR_URL>`: Fetch only the thread containing a specific comment.
- `--list-users`: List unique commenters as `Display Name (account_id)`.
- `--author <USER>`: Filter comments by display name or account ID, case-insensitively.
- `--hide-resolved`: Suppress resolved comment threads.

## Maintenance

If this skill's version does not match the `bbpr2md` tool version, update the skill from `https://github.com/scottmmjackson/bbpr2md/blob/master/examples/codex_skill/SKILL.md`.
