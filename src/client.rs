use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

/// Represents a Bitbucket user.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct User {
    /// The user's display name.
    pub display_name: String,
    /// The user's unique account ID.
    pub account_id: String,
}

/// Represents the content of a comment or task.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Content {
    /// The raw markdown content.
    pub raw: String,
    /// The rendered HTML content.
    pub html: Option<String>,
}

/// Represents inline location information for a comment.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Inline {
    /// The file path.
    pub path: String,
    /// The start line number.
    pub from: Option<u32>,
    /// The end line number.
    pub to: Option<u32>,
}

/// Represents the state of a task.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    /// The task is open/unresolved.
    Unresolved,
    /// The task has been resolved.
    Resolved,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Unresolved => write!(f, "UNRESOLVED"),
            TaskState::Resolved => write!(f, "RESOLVED"),
        }
    }
}

/// Represents a pull request comment.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Comment {
    /// The comment ID.
    pub id: u64,
    /// The comment content.
    pub content: Content,
    /// The user who created the comment.
    pub user: User,
    /// The creation date-time string.
    pub created_on: String,
    /// The last update date-time string.
    pub updated_on: Option<String>,
    /// Inline location if applicable.
    pub inline: Option<Inline>,
    /// Parent comment link if it's a reply.
    pub parent: Option<CommentParent>,
    /// Whether the comment was deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// Represents a link to a parent comment.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct CommentParent {
    /// The parent comment ID.
    pub id: u64,
}

/// Represents a pull request task.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Task {
    /// The task ID.
    pub id: u64,
    /// The task content.
    pub content: Content,
    /// The current state of the task.
    pub state: TaskState,
    /// The user who created the task.
    pub creator: Option<User>,
    /// The creation date-time string.
    pub created_on: String,
    /// The last update date-time string.
    pub updated_on: Option<String>,
    /// Link to the associated comment if any.
    pub comment: Option<TaskCommentLink>,
}

/// Represents a link to a comment from a task.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TaskCommentLink {
    /// The associated comment ID.
    pub id: u64,
}

#[derive(Debug, Deserialize)]
struct PaginatedResponse<T> {
    pub values: Vec<T>,
    pub next: Option<String>,
}

/// Represents a Bitbucket API error response.
#[derive(Debug, Deserialize)]
pub struct BitbucketError {
    pub error: BitbucketErrorDetail,
}

/// Detailed error information from Bitbucket.
#[derive(Debug, Deserialize)]
pub struct BitbucketErrorDetail {
    pub message: String,
    pub detail: Option<String>,
}

impl std::fmt::Display for BitbucketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error.message)?;
        if let Some(detail) = &self.error.detail {
            write!(f, " ({})", detail)?;
        }
        Ok(())
    }
}

/// Represents a Bitbucket pull request.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    /// The pull request ID.
    pub id: u32,
    /// The pull request title.
    pub title: String,
    /// The pull request description.
    pub description: String,
    /// The user who created the pull request.
    pub author: User,
    /// The creation date-time string.
    pub created_on: String,
    /// The last update date-time string.
    pub updated_on: String,
    /// The current state of the pull request.
    pub state: String,
}

/// Minimal pull request info returned when listing PRs.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PullRequestBrief {
    pub id: u32,
    pub title: String,
}

/// Percent-encodes characters in `s` that are not URL-safe path characters.
/// Leaves `A-Z a-z 0-9 - _ . ~` unencoded.
fn percent_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// A client for interacting with the Bitbucket API.
pub struct BitbucketClient {
    client: Client,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    debug: bool,
}

impl BitbucketClient {
    /// Creates a new Bitbucket client.
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
        debug: bool,
    ) -> Self {
        let client = Client::builder()
            .user_agent("bbpr2md/0.1.0")
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://api.bitbucket.org/2.0".to_string(),
            username,
            password,
            token,
            debug,
        }
    }

    fn auth_request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut b = builder;
        if let Some(token) = &self.token {
            b = b.bearer_auth(token);
        } else if let (Some(u), Some(p)) = (&self.username, &self.password) {
            b = b.basic_auth(u, Some(p));
        }
        b
    }

    /// Fetches pull request details.
    pub async fn get_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u32,
    ) -> Result<PullRequest> {
        let url = format!(
            "{}/repositories/{}/{}/pullrequests/{}",
            self.base_url, workspace, repo_slug, pr_id
        );

        let mut req_builder = self.client.get(&url);
        req_builder = self.auth_request(req_builder);

        if self.debug {
            eprintln!("GET {}", url);
        }

        let resp = req_builder
            .send()
            .await
            .context("Failed to send request for pull request details")?;

        if !resp.status().is_success() {
            let status = resp.status();
            if let Ok(err_resp) = resp.json::<BitbucketError>().await {
                anyhow::bail!(
                    "API request for pull request failed ({}): {}",
                    status,
                    err_resp
                );
            } else {
                anyhow::bail!("API request for pull request failed with status {}", status);
            }
        }

        let pr = resp
            .json::<PullRequest>()
            .await
            .context("Failed to parse pull request response")?;

        Ok(pr)
    }

    /// Fetches all comments for a given pull request, handling pagination.
    pub async fn get_comments(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u32,
    ) -> Result<Vec<Comment>> {
        let mut comments = Vec::new();
        let mut url = format!(
            "{}/repositories/{}/{}/pullrequests/{}/comments",
            self.base_url, workspace, repo_slug, pr_id
        );

        loop {
            let mut req_builder = self.client.get(&url);
            req_builder = self.auth_request(req_builder);

            if self.debug {
                eprintln!("GET {}", url);
            }

            let resp = req_builder
                .send()
                .await
                .context("Failed to send request for comments")?;

            if !resp.status().is_success() {
                let status = resp.status();
                if let Ok(err_resp) = resp.json::<BitbucketError>().await {
                    anyhow::bail!("API request for comments failed ({}): {}", status, err_resp);
                } else {
                    anyhow::bail!("API request for comments failed with status {}", status);
                }
            }

            let resp_json = resp
                .json::<PaginatedResponse<Comment>>()
                .await
                .context("Failed to parse comments response")?;

            comments.extend(resp_json.values);

            if let Some(next_url) = resp_json.next {
                url = next_url;
            } else {
                break;
            }
        }

        Ok(comments)
    }

    /// Finds open pull requests whose source branch matches `branch`.
    pub async fn find_open_prs_for_branch(
        &self,
        workspace: &str,
        repo_slug: &str,
        branch: &str,
    ) -> Result<Vec<PullRequestBrief>> {
        // Percent-encode the q filter value.  Double-quotes and spaces are the
        // only characters the filter expression itself introduces; the branch
        // name is validated by git and cannot contain '"'.
        let q_encoded = format!(
            "source.branch.name%3D%22{}%22%20AND%20state%3D%22OPEN%22",
            percent_encode_path(branch)
        );

        let mut prs = Vec::new();
        let mut page: u32 = 1;

        loop {
            let url = format!(
                "{}/repositories/{}/{}/pullrequests?q={}&pagelen=50&page={}",
                self.base_url, workspace, repo_slug, q_encoded, page
            );

            let req_builder = self.auth_request(self.client.get(&url));

            if self.debug {
                eprintln!("GET {}", url);
            }

            let resp = req_builder
                .send()
                .await
                .context("Failed to send request for PR listing")?;

            if !resp.status().is_success() {
                let status = resp.status();
                if let Ok(err_resp) = resp.json::<BitbucketError>().await {
                    anyhow::bail!("API request for PR listing failed ({}): {}", status, err_resp);
                } else {
                    anyhow::bail!("API request for PR listing failed with status {}", status);
                }
            }

            let resp_json = resp
                .json::<PaginatedResponse<PullRequestBrief>>()
                .await
                .context("Failed to parse PR listing response")?;

            let has_next = resp_json.next.is_some();
            prs.extend(resp_json.values);

            if has_next {
                page += 1;
            } else {
                break;
            }
        }

        Ok(prs)
    }

    /// Fetches all tasks for a given pull request, handling pagination.
    pub async fn get_tasks(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u32,
    ) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let mut url = format!(
            "{}/repositories/{}/{}/pullrequests/{}/tasks",
            self.base_url, workspace, repo_slug, pr_id
        );

        loop {
            let mut req_builder = self.client.get(&url);
            req_builder = self.auth_request(req_builder);

            if self.debug {
                eprintln!("GET {}", url);
            }

            let resp = req_builder
                .send()
                .await
                .context("Failed to send request for tasks")?;

            if !resp.status().is_success() {
                let status = resp.status();
                if let Ok(err_resp) = resp.json::<BitbucketError>().await {
                    anyhow::bail!("API request for tasks failed ({}): {}", status, err_resp);
                } else {
                    anyhow::bail!("API request for tasks failed with status {}", status);
                }
            }

            let resp_json = resp
                .json::<PaginatedResponse<Task>>()
                .await
                .context("Failed to parse tasks response")?;

            tasks.extend(resp_json.values);

            if let Some(next_url) = resp_json.next {
                url = next_url;
            } else {
                break;
            }
        }

        Ok(tasks)
    }
}
