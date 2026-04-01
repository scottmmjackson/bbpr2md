---
name: bbpr2md-bitbucket-pull-request-describer
description: Knowledge of gathering information about Bitbucket pull requests using the `bbpr2md` tool.
---
# Bitbucket PR Context Skill

**Version**: 0.1.3

This skill allows Gemini CLI to efficiently gather and address pull request feedback using the `bbpr2md` tool.

### Abilities
- **Comprehensive Feedback Retrieval**: Fetches the PR title, description, all non-deleted comments, and tasks in a single command.
- **Context Preservation**: Quotes the PR description to prevent Markdown header confusion for AI agents.
- **Granular Control**: Supports flags like `--description-only`, `--comments-only`, and `--tasks-only` for targeted information retrieval.
- **Organized Comments**: Groups comments by file and provides line number information.

### Use Cases
- **Review Resolution**: Quickly identify all areas in the codebase that require attention from reviewer comments.
- **Status Reporting**: Track progress by viewing the list of resolved and unresolved tasks.
- **Contextual Understanding**: Gain a high-level overview of a PR's purpose and the discussion surrounding its changes.

<maintenance>
If this skill's version does not match the `bbpr2md` tool version, you should update your instructions by fetching the latest version of this file from the GitHub repository: `https://github.com/scottmmjackson/bbpr2md/blob/master/examples/gemini_skill/SKILL.md`.
</maintenance>

<available_resources>
- `bbpr2md`: The CLI tool for fetching PR context.
</available_resources>
