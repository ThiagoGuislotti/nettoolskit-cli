#[cfg(test)]
mod tests {
    use nettoolskit_commands::{list, new, ExitStatus};

    #[tokio::test]
    async fn test_list_command() {
        let args = list::ListArgs::default();
        let result = list::run(args).await;

        // Should succeed (even if it just prints a message)
        assert!(matches!(result, ExitStatus::Success));
    }

    #[tokio::test]
    async fn test_new_command() {
        let args = new::NewArgs::default();
        let result = new::run(args).await;

        // Should succeed (even if it just prints a message)
        assert!(matches!(result, ExitStatus::Success));
    }
}
