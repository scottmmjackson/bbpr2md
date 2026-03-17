mod client;
mod formatter;

use crate::client::BitbucketClient;
use crate::formatter::format_to_markdown;
use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present.
    dotenv().ok();

    let args = Args::parse();

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
        Some(client.get_pull_request(&workspace, &repo_slug, pr_id).await?)
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
