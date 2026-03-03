use crate::client::{Comment, Task, TaskState};

/// Formats a list of Bitbucket comments and tasks into a Markdown string.
pub fn format_to_markdown(comments: &[Comment], tasks: &[Task]) -> String {
    let mut output = String::new();
    output.push_str("# Bitbucket Pull Request Feedback\n\n");

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
            output.push_str(&format!(
                "- {} {} (Creator: {}, State: {})\n",
                state_icon, task.content.raw, task.creator.display_name, task.state
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Content, Inline, Task, TaskState, User};

    #[test]
    fn test_format_to_markdown_empty() {
        let result = format_to_markdown(&[], &[]);
        assert!(result.contains("# Bitbucket Pull Request Feedback"));
    }

    #[test]
    fn test_format_to_markdown_with_comments() {
        let comments = vec![Comment {
            id: 1,
            content: Content {
                raw: "Comment 1".to_string(),
                html: None,
            },
            user: User {
                display_name: "User A".to_string(),
                account_id: "a".to_string(),
            },
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
        let result = format_to_markdown(&comments, &[]);
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
            creator: User {
                display_name: "User B".to_string(),
                account_id: "b".to_string(),
            },
            created_on: "2023-10-27T10:00:00Z".to_string(),
            updated_on: None,
            comment: None,
        }];
        let result = format_to_markdown(&[], &tasks);
        assert!(result.contains("## Tasks"));
        assert!(result.contains("- [ ] Task 1 (Creator: User B, State: UNRESOLVED)"));
    }
}
