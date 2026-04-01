mod client;
mod formatter;

use crate::client::BitbucketClient;
use crate::formatter::format_to_markdown;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use directories::UserDirs;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// Configuration for bbpr2md.
#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    /// The default Bitbucket username.
    username: Option<String>,
    /// The default Bitbucket app password.
    app_password: Option<String>,
    /// The default Bitbucket Bearer token.
    token: Option<String>,
    /// The default Bitbucket workspace ID.
    workspace: Option<String>,
    /// The default repository slug.
    repo_slug: Option<String>,
}

/// A CLI tool to fetch Bitbucket PR comments and tasks and output them as Markdown.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<SubCommand>,

    /// The Bitbucket workspace ID.
    #[arg(short, long)]
    workspace: Option<String>,

    /// The repository slug.
    #[arg(short, long)]
    repo_slug: Option<String>,

    /// The pull request ID.
    #[arg(short, long)]
    pr_id: Option<u32>,

    /// Optional path to save the Markdown output. If not provided, prints to stdout.
    #[arg(short, long)]
    output: Option<String>,

    /// Initialize the configuration file with default values if it doesn't exist.
    #[arg(long)]
    init: bool,

    /// Bitbucket username.
    #[arg(long)]
    username: Option<String>,

    /// Bitbucket app password.
    #[arg(long)]
    app_password: Option<String>,

    /// Bitbucket Personal Access Token (Bearer token).
    #[arg(long)]
    token: Option<String>,

    /// Print debug information (e.g., request headers).
    #[arg(long)]
    debug: bool,

    /// Include only the pull request description in the output.
    #[arg(long, conflicts_with_all = ["comments_only", "tasks_only", "comments_and_tasks"])]
    description_only: bool,

    /// Include only the pull request comments in the output.
    #[arg(long, conflicts_with_all = ["description_only", "tasks_only", "comments_and_tasks"])]
    comments_only: bool,

    /// Include only the pull request tasks in the output.
    #[arg(long, conflicts_with_all = ["description_only", "comments_only", "comments_and_tasks"])]
    tasks_only: bool,

    /// Include only comments and tasks (exclude description).
    #[arg(long, conflicts_with_all = ["description_only", "comments_only", "tasks_only"])]
    comments_and_tasks: bool,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Manage AI agent skills.
    Skill {
        #[command(subcommand)]
        agent: SkillSubcommand,
    },
}

#[derive(Subcommand, Debug)]
enum SkillSubcommand {
    /// Install the Claude skill.
    Claude {
        /// Install globally in the home directory.
        #[arg(short, long)]
        global: bool,

        /// Skip confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
    /// Install the Gemini skill.
    Gemini {
        /// Install globally in the home directory.
        #[arg(short, long)]
        global: bool,

        /// Skip confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present.
    dotenv().ok();

    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            SubCommand::Skill { agent } => match agent {
                SkillSubcommand::Claude { global, yes } => {
                    install_skill("Claude", global, yes)?;
                }
                SkillSubcommand::Gemini { global, yes } => {
                    install_skill("Gemini", global, yes)?;
                }
            },
        }
        return Ok(());
    }

    if args.init {
        let path = confy::get_configuration_file_path("bbpr2md", None)
            .context("Failed to get configuration file path")?;
        if !path.exists() {
            confy::store("bbpr2md", None, Config::default())
                .context("Failed to initialize configuration")?;
            eprintln!("Initialized default configuration at: {}", path.display());
        } else {
            eprintln!("Configuration already exists at: {}", path.display());
        }
        return Ok(());
    }

    // Load configuration from the standard location.
    let cfg: Config = confy::load("bbpr2md", None).context("Failed to load configuration")?;

    // Resolve workspace.
    let workspace = args
        .workspace
        .or(cfg.workspace)
        .context("Workspace must be provided via CLI, config file, or environment")?;

    // Resolve repo slug.
    let repo_slug = args
        .repo_slug
        .or(cfg.repo_slug)
        .context("Repository slug must be provided via CLI, config file, or environment")?;

    // Resolve username (CLI > Env > Config).
    let username = args
        .username
        .clone()
        .or_else(|| env::var("BITBUCKET_USERNAME").ok())
        .or_else(|| cfg.username.clone());

    // Resolve password (CLI > Env > Config).
    let password = args
        .app_password
        .clone()
        .or_else(|| env::var("BITBUCKET_APP_PASSWORD").ok())
        .or_else(|| cfg.app_password.clone());

    // Resolve token (CLI > Env > Config).
    let token = args
        .token
        .clone()
        .or_else(|| env::var("BITBUCKET_TOKEN").ok())
        .or_else(|| cfg.token.clone());

    if token.is_none() && (username.is_none() || password.is_none()) {
        anyhow::bail!("Bitbucket credentials must be set (either BITBUCKET_TOKEN/config.token or BITBUCKET_USERNAME+BITBUCKET_APP_PASSWORD/config.username+config.app_password)");
    }

    let pr_id = args.pr_id.context("Pull request ID must be provided")?;

    let client = BitbucketClient::new(username, password, token, args.debug);

    let (include_description, include_comments, include_tasks) = if args.description_only {
        (true, false, false)
    } else if args.comments_only {
        (false, true, false)
    } else if args.tasks_only {
        (false, false, true)
    } else if args.comments_and_tasks {
        (false, true, true)
    } else {
        (true, true, true)
    };

    let pr = if include_description {
        eprintln!("Fetching pull request details for #{}...", pr_id);
        Some(
            client
                .get_pull_request(&workspace, &repo_slug, pr_id)
                .await?,
        )
    } else {
        None
    };

    let comments = if include_comments {
        eprintln!(
            "Fetching comments for PR #{} from {}/{}...",
            pr_id, workspace, repo_slug
        );
        client.get_comments(&workspace, &repo_slug, pr_id).await?
    } else {
        Vec::new()
    };

    let tasks = if include_tasks {
        eprintln!("Fetching tasks for PR #{}...", pr_id);
        client.get_tasks(&workspace, &repo_slug, pr_id).await?
    } else {
        Vec::new()
    };

    let markdown = format_to_markdown(pr.as_ref(), &comments, &tasks);

    if let Some(output_path) = args.output {
        fs::write(&output_path, markdown)
            .context(format!("Failed to write to file: {}", output_path))?;
        eprintln!("Markdown output saved to: {}", output_path);
    } else {
        println!("{}", markdown);
    }

    Ok(())
}

fn install_skill(agent: &str, global: bool, yes: bool) -> Result<()> {
    let (source_file, skill_name, sub_dir) = match agent {
        "Claude" => ("examples/claude_skill.md", "bbpr2md", ".claude/skills"),
        "Gemini" => (
            "examples/gemini_skill/SKILL.md",
            "bbpr2md-bitbucket-pull-request-describer",
            ".gemini/skills",
        ),
        _ => anyhow::bail!("Unsupported agent: {}", agent),
    };

    let base_dir = if global {
        UserDirs::new()
            .context("Failed to get user home directory")?
            .home_dir()
            .to_path_buf()
    } else {
        env::current_dir().context("Failed to get current directory")?
    };

    let target_dir = base_dir.join(sub_dir).join(skill_name);
    let target_file = target_dir.join("SKILL.md");

    if !yes {
        let prompt = format!("Install {} skill to {}?", agent, target_file.display());
        if !Confirm::new().with_prompt(prompt).interact()? {
            eprintln!("Installation cancelled.");
            return Ok(());
        }
    }

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).context(format!(
            "Failed to create directory: {}",
            target_dir.display()
        ))?;
    }

    let source_path = Path::new(source_file);
    if !source_path.exists() {
        // If we are running from a binary, we might not have the source file locally.
        // For now, assume it exists since it's a local development tool.
        anyhow::bail!("Source skill file not found at: {}", source_file);
    }

    fs::copy(source_path, &target_file).context(format!(
        "Failed to copy {} to {}",
        source_file,
        target_file.display()
    ))?;

    eprintln!(
        "Successfully installed {} skill to {}",
        agent,
        target_file.display()
    );
    Ok(())
}
