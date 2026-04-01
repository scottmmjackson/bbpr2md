use crate::client::{Comment, PullRequest, Task, TaskState};

/// Formats a Bitbucket pull request, comments, and tasks into a Markdown string.
///
/// If `pr` is provided, the pull request title and description are included.
/// Comments are filtered to exclude deleted ones and grouped by file path.
/// Tasks are listed with their current state and creator.
pub fn format_to_markdown(
    pr: Option<&PullRequest>,
    comments: &[Comment],
    tasks: &[Task],
) -> String {
    let mut output = String::new();

    if let Some(pr) = pr {
        output.push_str(&format!("# PR #{}: {}\n\n", pr.id, pr.title));
        output.push_str("## Description\n\n");
        if pr.description.is_empty() {
            output.push_str("*No description provided.*\n");
        } else {
            output.push_str("> ");
            output.push_str(&pr.description.replace('\n', "\n> "));
        }
        output.push_str("\n\n");
    } else {
        output.push_str("# Bitbucket Pull Request Feedback\n\n");
    }

    if !comments.is_empty() {
        output.push_str("## Comments\n\n");

        let mut sorted_comments: Vec<_> = comments.iter().filter(|c| !c.deleted).collect();
        // Sort by path, then by date.
        sorted_comments.sort_by(|a, b| {
            let path_a = a.inline.as_ref().map(|i| i.path.as_str()).unwrap_or("");
            let path_b = b.inline.as_ref().map(|i| i.path.as_str()).unwrap_or("");

            path_a.cmp(path_b).then(a.created_on.cmp(&b.created_on))
        });

        let mut current_path = String::new();

        for comment in sorted_comments {
            let path = comment
                .inline
                .as_ref()
                .map(|i| i.path.clone())
                .unwrap_or_else(|| "General".to_string());

            if path != current_path {
                current_path = path;
                output.push_str(&format!("### File: `{}`\n\n", current_path));
            }

            let user = &comment.user.display_name;
            let date = &comment.created_on[..10];
            let content = &comment.content.raw;
            let line_info = comment
                .inline
                .as_ref()
                .and_then(|i| i.to.map(|to| format!(" (Line {})", to)))
                .unwrap_or_default();

            let header = format!("**{}** ({}){}", user, date, line_info);

            if comment.parent.is_some() {
                output.push_str("> ");
                output.push_str(&header);
                output.push_str("\n> ");
                output.push_str(&content.replace('\n', "\n> "));
            } else {
                output.push_str(&header);
                output.push('\n');
                output.push_str(content);
            }
            output.push_str("\n\n---\n\n");
        }
    }

    if !tasks.is_empty() {
        output.push_str("## Tasks\n\n");
        for task in tasks {
            let state_icon = match task.state {
                TaskState::Unresolved => "[ ]",
                TaskState::Resolved => "[x]",
            };
            let creator_name = task
                .creator
                .as_ref()
                .map(|u| u.display_name.as_str())
                .unwrap_or("Unknown");

            output.push_str(&format!(
                "- {} {} (Creator: {}, State: {})\n",
                state_icon, task.content.raw, creator_name, task.state
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Content, Inline, Task, TaskState, User};

    fn mock_user() -> User {
        User {
            display_name: "User A".to_string(),
            account_id: "a".to_string(),
        }
    }

    #[test]
    fn test_format_to_markdown_empty() {
        let result = format_to_markdown(None, &[], &[]);
        assert!(result.contains("# Bitbucket Pull Request Feedback"));
    }

    #[test]
    fn test_format_to_markdown_with_pr() {
        let pr = PullRequest {
            id: 123,
            title: "My PR".to_string(),
            description: "Some description\nWith multiple lines".to_string(),
            author: mock_user(),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: "2023-10-27T10:00:00Z".to_string(),
            state: "OPEN".to_string(),
        };
        let result = format_to_markdown(Some(&pr), &[], &[]);
        assert!(result.contains("# PR #123: My PR"));
        assert!(result.contains("## Description"));
        assert!(result.contains("> Some description\n> With multiple lines"));
    }

    #[test]
    fn test_format_to_markdown_with_comments() {
        let comments = vec![Comment {
            id: 1,
            content: Content {
                raw: "Comment 1".to_string(),
                html: None,
            },
            user: mock_user(),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            inline: Some(Inline {
                path: "src/main.rs".to_string(),
                from: None,
                to: Some(10),
            }),
            parent: None,
            deleted: false,
        }];
        let result = format_to_markdown(None, &comments, &[]);
        assert!(result.contains("### File: `src/main.rs`"));
        assert!(result.contains("**User A** (2023-10-27) (Line 10)"));
        assert!(result.contains("Comment 1"));
    }

    #[test]
    fn test_format_to_markdown_with_tasks() {
        let tasks = vec![Task {
            id: 1,
            content: Content {
                raw: "Task 1".to_string(),
                html: None,
            },
            state: TaskState::Unresolved,
            creator: Some(mock_user()),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: None,
        }];
        let result = format_to_markdown(None, &[], &tasks);
        assert!(result.contains("## Tasks"));
        assert!(result.contains("- [ ] Task 1 (Creator: User A, State: UNRESOLVED)"));
    }

    #[test]
    fn test_format_to_markdown_with_task_no_creator() {
        let tasks = vec![Task {
            id: 1,
            content: Content {
                raw: "Task 2".to_string(),
                html: None,
            },
            state: TaskState::Unresolved,
            creator: None,
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: None,
        }];
        let result = format_to_markdown(None, &[], &tasks);
        assert!(result.contains("- [ ] Task 2 (Creator: Unknown, State: UNRESOLVED)"));
    }
}
