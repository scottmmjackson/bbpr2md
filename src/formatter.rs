use crate::client::{Comment, PullRequest, Task, TaskState};
use std::collections::HashMap;

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

    // Split tasks into those linked to a specific comment and global (PR-level) tasks.
    let mut comment_tasks: HashMap<u64, Vec<&Task>> = HashMap::new();
    let mut global_tasks: Vec<&Task> = Vec::new();
    for task in tasks {
        match &task.comment {
            Some(link) => comment_tasks.entry(link.id).or_default().push(task),
            None => global_tasks.push(task),
        }
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

            // Render tasks linked to this comment inline.
            if let Some(linked_tasks) = comment_tasks.get(&comment.id) {
                output.push('\n');
                for task in linked_tasks {
                    let state_icon = match task.state {
                        TaskState::Unresolved => "[ ]",
                        TaskState::Resolved => "[x]",
                    };
                    let creator = task
                        .creator
                        .as_ref()
                        .map(|u| u.display_name.as_str())
                        .unwrap_or("Unknown");
                    output.push_str(&format!(
                        "\n- {} {} *(Creator: {})*",
                        state_icon, task.content.raw, creator
                    ));
                }
            }

            output.push_str("\n\n---\n\n");
        }
    }

    if !global_tasks.is_empty() {
        let inline_count: usize = comment_tasks.values().map(|v| v.len()).sum();
        let heading = if inline_count > 0 {
            format!(
                "## Tasks ({} other inline task{} shown above)\n\n",
                inline_count,
                if inline_count == 1 { "" } else { "s" }
            )
        } else {
            "## Tasks\n\n".to_string()
        };
        output.push_str(&heading);
        for task in global_tasks {
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

    fn mock_comment(id: u64) -> Comment {
        Comment {
            id,
            content: Content {
                raw: format!("Comment {}", id),
                html: None,
            },
            user: mock_user(),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            inline: None,
            parent: None,
            deleted: false,
            resolution: None,
        }
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
            resolution: None,
        }];
        let result = format_to_markdown(None, &comments, &[]);
        assert!(result.contains("### File: `src/main.rs`"));
        assert!(result.contains("**User A** (2023-10-27) (Line 10)"));
        assert!(result.contains("Comment 1"));
    }

    #[test]
    fn test_comment_linked_task_renders_inline() {
        use crate::client::TaskCommentLink;
        let comments = vec![mock_comment(1)];
        let tasks = vec![Task {
            id: 10,
            content: Content {
                raw: "Fix this".to_string(),
                html: None,
            },
            state: TaskState::Unresolved,
            creator: Some(mock_user()),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: Some(TaskCommentLink { id: 1 }),
        }];
        let result = format_to_markdown(None, &comments, &tasks);
        // Inline task should appear, global Tasks section should not
        assert!(result.contains("- [ ] Fix this *(Creator: User A)*"));
        assert!(!result.contains("## Tasks"));
    }

    #[test]
    fn test_global_task_renders_at_bottom() {
        let comments = vec![mock_comment(1)];
        let tasks = vec![Task {
            id: 10,
            content: Content {
                raw: "Global task".to_string(),
                html: None,
            },
            state: TaskState::Resolved,
            creator: Some(mock_user()),
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: None,
        }];
        let result = format_to_markdown(None, &comments, &tasks);
        assert!(result.contains("## Tasks\n"));
        assert!(result.contains("[x] Global task (Creator: User A, State: RESOLVED)"));
        // Global task must not appear inside the comments section
        let tasks_pos = result.find("## Tasks").unwrap();
        let comment_pos = result.find("Comment 1").unwrap();
        assert!(tasks_pos > comment_pos);
    }

    #[test]
    fn test_tasks_heading_counts_inline_tasks() {
        use crate::client::TaskCommentLink;
        let comments = vec![mock_comment(1)];
        let tasks = vec![
            Task {
                id: 10,
                content: Content { raw: "Inline task".to_string(), html: None },
                state: TaskState::Unresolved,
                creator: Some(mock_user()),
                created_on: "2023-10-27T10:00:00Z".to_string(),
                updated_on: None,
                comment: Some(TaskCommentLink { id: 1 }),
            },
            Task {
                id: 11,
                content: Content { raw: "Global task".to_string(), html: None },
                state: TaskState::Unresolved,
                creator: Some(mock_user()),
                created_on: "2023-10-27T10:00:00Z".to_string(),
                updated_on: None,
                comment: None,
            },
        ];
        let result = format_to_markdown(None, &comments, &tasks);
        assert!(result.contains("## Tasks (1 other inline task shown above)"));
    }

    #[test]
    fn test_tasks_heading_plural_inline_tasks() {
        use crate::client::TaskCommentLink;
        let comments = vec![mock_comment(1)];
        let tasks = vec![
            Task {
                id: 10,
                content: Content { raw: "Inline 1".to_string(), html: None },
                state: TaskState::Unresolved,
                creator: None,
                created_on: "2023-10-27T10:00:00Z".to_string(),
                updated_on: None,
                comment: Some(TaskCommentLink { id: 1 }),
            },
            Task {
                id: 11,
                content: Content { raw: "Inline 2".to_string(), html: None },
                state: TaskState::Unresolved,
                creator: None,
                created_on: "2023-10-27T10:00:00Z".to_string(),
                updated_on: None,
                comment: Some(TaskCommentLink { id: 1 }),
            },
            Task {
                id: 12,
                content: Content { raw: "Global".to_string(), html: None },
                state: TaskState::Unresolved,
                creator: None,
                created_on: "2023-10-27T10:00:00Z".to_string(),
                updated_on: None,
                comment: None,
            },
        ];
        let result = format_to_markdown(None, &comments, &tasks);
        assert!(result.contains("## Tasks (2 other inline tasks shown above)"));
    }

    #[test]
    fn test_tasks_heading_no_inline_annotation_when_none() {
        let tasks = vec![Task {
            id: 10,
            content: Content { raw: "Global only".to_string(), html: None },
            state: TaskState::Unresolved,
            creator: None,
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: None,
        }];
        let result = format_to_markdown(None, &[], &tasks);
        assert!(result.contains("## Tasks\n"));
        assert!(!result.contains("inline task"));
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
