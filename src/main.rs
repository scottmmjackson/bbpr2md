mod client;
mod formatter;

use crate::client::{BitbucketClient, Comment};
use crate::formatter::format_to_markdown;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use directories::UserDirs;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// Configuration for bbpr2md.
#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    /// The default Bitbucket username.
    username: Option<String>,
    /// The default Bitbucket app password.
    app_password: Option<String>, // pragma: allowlist-secret
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
    app_password: Option<String>, // pragma: allowlist-secret

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

    /// Show only the thread containing this comment.
    /// Accepts a comment ID (e.g. 789186489) or a full Bitbucket comment URL
    /// (e.g. https://bitbucket.org/org/repo/pull-requests/1#comment-789186489).
    #[arg(long, conflicts_with_all = ["description_only", "comments_only", "tasks_only", "comments_and_tasks"])]
    comment: Option<String>,
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

/// Parses a comment ID from either a raw integer string or a Bitbucket comment URL.
///
/// Accepts:
/// - A plain integer: `"789186489"`
/// - A Bitbucket URL fragment: `"https://bitbucket.org/org/repo/pull-requests/1379#comment-789186489"`
fn parse_comment_id(s: &str) -> Result<u64> {
    if let Ok(id) = s.trim().parse::<u64>() {
        return Ok(id);
    }
    if let Some(fragment) = s.split('#').nth(1) {
        if let Some(id_str) = fragment.strip_prefix("comment-") {
            if let Ok(id) = id_str.trim().parse::<u64>() {
                return Ok(id);
            }
        }
    }
    anyhow::bail!("Could not parse comment ID from: {}", s)
}

/// Returns all comments belonging to the thread that contains `comment_id`.
///
/// The thread is defined as the root comment (the ancestor with no parent) plus
/// every descendant.  The returned vec is in breadth-first order so the root
/// always comes first.
fn collect_thread(comments: &[Comment], comment_id: u64) -> Result<Vec<Comment>> {
    let id_map: HashMap<u64, &Comment> = comments.iter().map(|c| (c.id, c)).collect();

    if !id_map.contains_key(&comment_id) {
        anyhow::bail!("Comment {} not found in this pull request", comment_id);
    }

    // Walk up the parent chain to locate the thread root.
    let mut root_id = comment_id;
    loop {
        match id_map[&root_id].parent.as_ref() {
            Some(p) => root_id = p.id,
            None => break,
        }
    }

    // Build a children map for efficient descendant traversal.
    let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
    for c in comments {
        if let Some(p) = &c.parent {
            children.entry(p.id).or_default().push(c.id);
        }
    }

    // BFS from root to collect the full thread in order.
    let mut result = Vec::new();
    let mut queue = vec![root_id];
    while !queue.is_empty() {
        let mut next_level = Vec::new();
        for id in queue {
            if let Some(c) = id_map.get(&id) {
                result.push((*c).clone());
            }
            if let Some(kids) = children.get(&id) {
                next_level.extend_from_slice(kids);
            }
        }
        queue = next_level;
    }

    Ok(result)
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
    let password = args // pragma: allowlist-secret
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

    let comment_id = args
        .comment
        .as_deref()
        .map(parse_comment_id)
        .transpose()?;

    let client = BitbucketClient::new(username, password, token, args.debug);

    let (include_description, include_comments, include_tasks) = if comment_id.is_some() {
        (false, true, false)
    } else if args.description_only {
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
        let all_comments = client.get_comments(&workspace, &repo_slug, pr_id).await?;
        if let Some(cid) = comment_id {
            collect_thread(&all_comments, cid)?
        } else {
            all_comments
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Comment, CommentParent, Content, User};

    fn mock_comment(id: u64, parent_id: Option<u64>) -> Comment {
        Comment {
            id,
            content: Content {
                raw: format!("comment {}", id),
                html: None,
            },
            user: User {
                display_name: "Tester".to_string(),
                account_id: "t".to_string(),
            },
            created_on: "2024-01-01T00:00:00Z".to_string(),
            updated_on: None,
            inline: None,
            parent: parent_id.map(|pid| CommentParent { id: pid }),
            deleted: false,
        }
    }

    #[test]
    fn test_parse_comment_id_raw() {
        assert_eq!(parse_comment_id("789186489").unwrap(), 789186489);
    }

    #[test]
    fn test_parse_comment_id_url() {
        let url = "https://bitbucket.org/sotericorp/security-for-everything/pull-requests/1379#comment-789186489";
        assert_eq!(parse_comment_id(url).unwrap(), 789186489);
    }

    #[test]
    fn test_parse_comment_id_invalid() {
        assert!(parse_comment_id("not-an-id").is_err());
    }

    #[test]
    fn test_collect_thread_root() {
        // root(1) -> child(2) -> grandchild(3), plus unrelated(4)
        let comments = vec![
            mock_comment(1, None),
            mock_comment(2, Some(1)),
            mock_comment(3, Some(2)),
            mock_comment(4, None),
        ];
        let thread = collect_thread(&comments, 1).unwrap();
        let ids: Vec<u64> = thread.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_collect_thread_from_reply() {
        // Requesting a reply returns the whole thread from root
        let comments = vec![
            mock_comment(1, None),
            mock_comment(2, Some(1)),
            mock_comment(3, Some(2)),
        ];
        let thread = collect_thread(&comments, 3).unwrap();
        let ids: Vec<u64> = thread.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_collect_thread_not_found() {
        let comments = vec![mock_comment(1, None)];
        assert!(collect_thread(&comments, 999).is_err());
    }
}
