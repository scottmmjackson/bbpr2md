---
name: bbpr2md
description: Fetch and format Bitbucket pull request descriptions, comments, and tasks as Markdown. Use to gather PR feedback, review changes, or track open tasks.
---
# Bitbucket PR Context Tool (bbpr2md)

**Version**: 0.1.3

The `bbpr2md` tool allows you to retrieve a structured Markdown representation of a Bitbucket pull request.

### Abilities
- **Description Retrieval**: Fetch the PR title and description, quoted to avoid Markdown header confusion.
- **Comment Aggregation**: Collect all non-deleted comments, grouped by file and sorted by line number.
- **Task Tracking**: List all PR tasks with their current state (UNRESOLVED/RESOLVED) and creator.
- **Selective Fetching**: Use flags like `--description-only`, `--comments-only`, or `--tasks-only` to limit output.

### Use Cases
- **Feedback Analysis**: Quickly see all reviewer comments across multiple files in a single view.
- **Change Planning**: Use the aggregated comments and line numbers to map out required code changes.
- **Task Verification**: Ensure all Bitbucket tasks are completed by referencing the task list.
- **Context Gathering**: Provide a concise summary of a PR's purpose and discussion for further analysis.

### Maintenance & Updates
If this skill's version does not match the `bbpr2md` tool version, you should update your instructions by fetching the latest version of this file from the GitHub repository: `https://github.com/scottmmjackson/bbpr2md/blob/master/examples/claude_skill.md`.
