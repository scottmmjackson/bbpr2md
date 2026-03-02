# bbpr2md (Bitbucket Pull Request to Markdown)

`bbpr2md` is a Rust CLI tool designed to fetch pull request comments and tasks from Bitbucket Cloud and format them into a clean Markdown document. This is primarily used as high-quality context for AI agents (like Claude or Gemini) to understand and address PR feedback.

## Preferred Use Case: Configuration Flow

The recommended way to use `bbpr2md` is through a persistent configuration file. This avoids passing common arguments (like workspace and repo) every time.

### 1. Initialize Configuration
Run the following command to create a default configuration file in your system's standard location:

```bash
bbpr2md --init
```

The command will print the path to the created file (e.g., `~/Library/Application Support/rs.bbpr2md/default-config.toml`).

### 2. Configure Defaults
Edit the created file to set your default values:

```toml
username = "your.email@example.com"
token = "your-bitbucket-personal-access-token"
workspace = "your-workspace"
repo_slug = "your-repo"
```

### 3. Usage
Once configured, you only need to provide the PR ID:

```bash
bbpr2md --pr-id 123
```

## AI Agent Workflow

The primary goal of `bbpr2md` is to simplify the feedback loop between code review and implementation.

### The Agent Cycle
1.  **Gather Context**: An AI agent runs `bbpr2md --pr-id <ID>` to retrieve all comments and tasks.
2.  **Analyze**: The agent parses the Markdown to identify requested changes, grouped by file and line.
3.  **Implement**: The agent applies the changes to the codebase.
4.  **Verify**: The agent uses the tasks list to ensure all items are addressed before marking them as resolved.

## Command Line Arguments

`bbpr2md` supports overrides for all configuration values via CLI flags:

-   `--pr-id <ID>`: (Required) The ID of the pull request.
-   `--workspace <WS>`: Bitbucket workspace ID.
-   `--repo-slug <REPO>`: Repository slug.
-   `--username <USER>`: Bitbucket username/email.
-   `--token <TOKEN>`: Personal Access Token (Bearer Auth).
-   `--app-password <PW>`: App Password (Basic Auth).
-   `--output <FILE>`: Save Markdown to a file instead of printing to stdout.
-   `--debug`: Print debug information (like requested URLs).

## Environment Variables

You can also use environment variables, which override the config file but are overridden by CLI flags:
-   `BITBUCKET_USERNAME`
-   `BITBUCKET_TOKEN`
-   `BITBUCKET_APP_PASSWORD`

## Examples

Check the [examples/](./examples) directory for:
-   `config.toml`: A sample configuration file.
-   `claude_skill.md`: A system prompt for Claude to use this tool effectively.
-   `gemini_skill/SKILL.md`: A native Gemini CLI skill definition.

## Development

Managed via `just`:
-   `just build`: Build the release binary.
-   `just test`: Run the test suite.
-   `just check`: Run linting and formatting checks.
