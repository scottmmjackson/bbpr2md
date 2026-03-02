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

    eprintln!(
        "Fetching comments for PR #{} from {}/{}...",
        pr_id, workspace, repo_slug
    );
    let comments = client.get_comments(&workspace, &repo_slug, pr_id).await?;

    eprintln!("Fetching tasks for PR #{}...", pr_id);
    let tasks = client.get_tasks(&workspace, &repo_slug, pr_id).await?;

    let markdown = format_to_markdown(&comments, &tasks);

    if let Some(output_path) = args.output {
        fs::write(&output_path, markdown)
            .context(format!("Failed to write to file: {}", output_path))?;
        eprintln!("Markdown output saved to: {}", output_path);
    } else {
        println!("{}", markdown);
    }

    Ok(())
}
