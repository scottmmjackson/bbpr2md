# Bitbucket PR Context Skill

**Version**: 0.1.3

This skill allows Gemini CLI to efficiently gather and address pull request feedback using the `bbpr2md` tool.

<instructions>
1. **Objective**: When a user asks you to "address PR feedback" or "look at PR #123", use `bbpr2md` to get the context.
2. **Context Gathering**: Run `cargo run -- --pr-id <ID>` (or use the installed binary) to get the Markdown feedback.
    - Use `--description-only` if you only need the PR context.
    - Use `--tasks-only` if you only need the current task list.
    - Use `--comments-only` if you only need to see the discussion.
3. **Surgical Edits**: Focus on the specific files and line numbers identified in the comments.
4. **Verification**: After making edits, cross-reference the `Tasks` list from the `bbpr2md` output.
5. **Status Reporting**: Inform the user which comments you've addressed and if there are any pending tasks you couldn't resolve automatically.
</instructions>

<maintenance>
If this skill's version does not match the `bbpr2md` tool version, you should update your instructions by fetching the latest version of this file from the GitHub repository: `https://github.com/scottmmjackson/bbpr2md/blob/master/examples/gemini_skill/SKILL.md`.
</maintenance>

<available_resources>
- `bbpr2md`: The CLI tool for fetching PR context.
</available_resources>
