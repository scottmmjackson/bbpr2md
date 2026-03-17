## Bitbucket Pull Request 2 Markdown (bbpr2md)

A Rust CLI tool that fetches comments and tasks from a Bitbucket pull request and outputs them in Markdown.

### Requirements

- **Authentication**: Supports Bitbucket Cloud API via username and app password.
- **Configuration**:
    - Credentials can be provided via environment variables (`BITBUCKET_USERNAME`, `BITBUCKET_APP_PASSWORD`).
    - Default values for `workspace`, `repo_slug`, `username`, and `app_password` can be loaded from a YAML configuration file.
    - The configuration file must be stored in a standard, cross-platform location (e.g., using the `confy` crate).
    - CLI arguments should override configuration file values.
- **Output**: Generates a structured Markdown report of all non-deleted comments (grouped by file) and tasks.

### Implementation Lessons & Maintenance Notes

- **Authentication Nuances**:
    - **Basic Auth**: Requires a Bitbucket-specific **App Password** (created in Bitbucket Personal Settings). Atlassian API Tokens (prefixed with `ATATT`) are *not* supported by the Bitbucket Cloud API.
    - **Bearer Auth**: Supported for Bitbucket **Personal Access Tokens (PATs)**.
- **API Data Model**:
    - **Task States**: The Bitbucket API uses `UNRESOLVED` as the variant for open tasks (contrary to some documentation suggesting `OPEN`). The `TaskState` enum handles this.
    - **Comment Threads**: Comments are flattened in the API response. Threading is reconstructed by checking the `parent` field. In Markdown, replies are indented using blockquotes (`> `) to maintain visual hierarchy.
    - **Deleted Comments**: The formatter filters out comments where `deleted: true` to keep the context clean for AI agents.
- **PR Descriptions & Section Filtering**:
    - **Description Inclusion**: The PR title and description are included by default to provide top-level context.
    - **Markdown Quoting**: The PR description (which may contain its own Markdown headers) is quoted using blockquotes (`> `). This prevents AI agents from confusing description subheadings with the main report's structure.
    - **Granular Control**: CLI flags (`--description-only`, `--comments-only`, `--tasks-only`, `--comments-and-tasks`) allow agents to fetch only the specific context they need, reducing token usage.
- **Configuration Management**:
    - Uses the `confy` crate. The application name is `bbpr2md`.
    - Default config path can be found via `bbpr2md --init`.
- **Build System**:
    - Replicates a robust cross-platform system using `just`.
    - Supports macOS (Intel/M1), Linux (amd64/arm64/termux), and Windows.
    - Automation is handled via `.github/workflows/release.yml`.

### Coding Standards

- Include docstrings on all functions, traits, and structs.
- Always test new code for correctness and robustness.
- Always update the README.md to reflect the latest changes.
- Always version skill files in `examples/` in lockstep with the tool's version in `Cargo.toml`.
- Ensure skill files contain instructions on how to update themselves from the GitHub repository (`https://github.com/scottmmjackson/bbpr2md`) if they are out of date.
- Update AGENTS.md with any newly-adopted coding standards, requirements, implementation lessons, and maintenance notes.
- Update the example skills in `examples` to reflect the latest changes.
