//! ChatOps public API integration tests.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use nettoolskit_core::IngressTransport;
use nettoolskit_orchestrator::{
    build_chatops_runtime, execute_chatops_envelope, parse_chatops_intent,
    ChatOpsAuthorizationPolicy, ChatOpsCommandEnvelope, ChatOpsIntent, ChatOpsLocalAuditStore,
    ChatOpsPlatform, ChatOpsRuntimeConfig, RecordingChatOpsNotifier,
};
use serial_test::serial;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

struct EnvVarGuard {
    saved: Vec<(String, Option<String>)>,
}

impl EnvVarGuard {
    fn set(vars: &[(&str, Option<&str>)]) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for (key, value) in vars {
            saved.push(((*key).to_string(), std::env::var(key).ok()));
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
        Self { saved }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..) {
            match value {
                Some(value) => std::env::set_var(&key, value),
                None => std::env::remove_var(&key),
            }
        }
    }
}

#[derive(Clone, Default)]
struct TelegramMockState {
    updates_calls: Arc<AtomicUsize>,
    send_message_calls: Arc<AtomicUsize>,
}

impl TelegramMockState {
    fn updates_calls(&self) -> usize {
        self.updates_calls.load(Ordering::SeqCst)
    }

    fn send_message_calls(&self) -> usize {
        self.send_message_calls.load(Ordering::SeqCst)
    }
}

async fn spawn_telegram_mock_server() -> (
    String,
    TelegramMockState,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock server should bind");
    let address = listener
        .local_addr()
        .expect("mock server should expose local address");
    let state = TelegramMockState::default();
    let state_for_task = state.clone();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((mut stream, _)) = accepted else {
                        break;
                    };

                    let mut buffer = [0_u8; 8 * 1024];
                    let bytes_read = match stream.read(&mut buffer).await {
                        Ok(bytes) => bytes,
                        Err(_) => continue,
                    };
                    if bytes_read == 0 {
                        continue;
                    }

                    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let request_line = request.lines().next().unwrap_or_default();
                    let mut parts = request_line.split_whitespace();
                    let method = parts.next().unwrap_or_default();
                    let path = parts.next().unwrap_or_default();

                    let (status_line, response_body) = match (method, path) {
                        ("GET", path) if path.starts_with("/bottest-token/getUpdates") => {
                            let call_index =
                                state_for_task.updates_calls.fetch_add(1, Ordering::SeqCst);
                            let body = if call_index == 0 {
                                r#"{"ok":true,"result":[{"update_id":101,"message":{"date":1737200000,"text":"list","chat":{"id":555},"from":{"id":777}}}]}"#
                                    .to_string()
                            } else {
                                r#"{"ok":true,"result":[]}"#.to_string()
                            };
                            ("200 OK", body)
                        }
                        ("POST", path) if path.starts_with("/bottest-token/sendMessage") => {
                            state_for_task
                                .send_message_calls
                                .fetch_add(1, Ordering::SeqCst);
                            ("200 OK", r#"{"ok":true,"result":{"message_id":1}}"#.to_string())
                        }
                        _ => (
                            "404 Not Found",
                            r#"{"ok":false,"description":"not found"}"#.to_string(),
                        ),
                    };

                    let response = format!(
                        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
                        response_body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            }
        }
    });

    (format!("http://{address}"), state, shutdown_tx, server_task)
}

#[test]
fn parse_chatops_intent_supports_prefixed_task_command() {
    let parsed = parse_chatops_intent("/ntk task submit ai-plan harden pipeline");
    assert_eq!(
        parsed,
        Ok(ChatOpsIntent::TaskSubmit {
            intent: "ai-plan".to_string(),
            payload: "harden pipeline".to_string()
        })
    );
}

#[tokio::test]
async fn execute_chatops_envelope_persists_local_audit_entries() {
    let dir = tempfile::tempdir().expect("temp dir");
    let audit_store = ChatOpsLocalAuditStore::from_path(dir.path().join("chatops-audit.jsonl"));
    let policy = ChatOpsAuthorizationPolicy::new(
        vec!["trusted-user".to_string()],
        vec!["trusted-channel".to_string()],
    );
    let notifier = RecordingChatOpsNotifier::new();
    let envelope = ChatOpsCommandEnvelope::new(
        ChatOpsPlatform::Telegram,
        "trusted-channel",
        "trusted-user",
        "list",
        1_737_200_000_000,
    );

    let status = execute_chatops_envelope(&envelope, &policy, &notifier, Some(&audit_store))
        .await
        .expect("authorized chatops command should execute");
    assert_eq!(status.to_string(), "success");

    let entries = audit_store
        .load_latest(16)
        .expect("audit entries should be readable");
    assert!(
        entries.len() >= 3,
        "expected received + executed + notified entries"
    );
    assert!(entries.iter().any(|entry| entry.note.contains("envelope")));
    assert!(entries
        .iter()
        .any(|entry| entry.internal_command.as_deref() == Some("/task list")));
    assert!(entries.iter().all(|entry| entry
        .request_id
        .as_deref()
        .is_some_and(|value| !value.is_empty())));
    assert!(entries
        .iter()
        .all(|entry| entry.transport == Some(IngressTransport::TelegramPolling)));
}

#[tokio::test]
async fn execute_chatops_envelope_rejects_disallowed_submit_scope() {
    let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
        vec!["trusted-user".to_string()],
        vec!["trusted-channel".to_string()],
        vec!["submit:ai-plan".to_string()],
    );
    let notifier = RecordingChatOpsNotifier::new();
    let envelope = ChatOpsCommandEnvelope::new(
        ChatOpsPlatform::Discord,
        "trusted-channel",
        "trusted-user",
        "submit ai-explain why deployment failed",
        1_737_200_000_111,
    );

    let result = execute_chatops_envelope(&envelope, &policy, &notifier, None).await;
    assert!(
        result.is_err(),
        "submit scope outside allowlist must be rejected"
    );
    let notifications = notifier.snapshot();
    assert_eq!(notifications.len(), 1);
}

#[tokio::test]
async fn chatops_vps_smoke_profile_executes_poll_and_notification_flow() {
    let (telegram_api_base, mock_state, shutdown_tx, server_task) =
        spawn_telegram_mock_server().await;
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let audit_path = temp_dir.path().join("chatops-vps-smoke-audit.jsonl");

    let config = ChatOpsRuntimeConfig {
        enabled: true,
        poll_interval: Duration::from_millis(500),
        max_batch_size: 4,
        allowed_user_ids: vec!["777".to_string()],
        allowed_channel_ids: vec!["555".to_string()],
        allowed_command_scopes: vec!["list".to_string()],
        telegram_bot_token: Some("test-token".to_string()),
        telegram_api_base,
        audit_path: Some(audit_path.clone()),
        ..ChatOpsRuntimeConfig::default()
    };

    let runtime = build_chatops_runtime(config)
        .expect("runtime build should succeed")
        .expect("enabled runtime should be present");
    let summary = runtime.tick().await;
    let _ = shutdown_tx.send(());
    let _ = server_task.await;

    assert_eq!(summary.envelopes_received, 1);
    assert_eq!(summary.executed_success, 1);
    assert_eq!(summary.executed_failed, 0);
    assert_eq!(summary.notification_errors, 0);
    assert_eq!(mock_state.updates_calls(), 1);
    assert_eq!(mock_state.send_message_calls(), 1);

    let audit_store = ChatOpsLocalAuditStore::from_path(audit_path);
    let entries = audit_store
        .load_latest(32)
        .expect("smoke audit entries should be readable");
    assert!(
        entries.len() >= 3,
        "expected envelope + execution + notification"
    );
    assert!(entries
        .iter()
        .any(|entry| entry.internal_command.as_deref() == Some("/task list")));
}

#[tokio::test]
#[serial]
async fn execute_chatops_envelope_submit_emits_control_plane_metadata_in_audit() {
    let _env_guard = EnvVarGuard::set(&[
        ("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan")),
        ("NTK_TOOL_SCOPE_INTENT_AI_PLAN_TOOLS", Some("ai.plan")),
    ]);
    let dir = tempfile::tempdir().expect("temp dir");
    let audit_store =
        ChatOpsLocalAuditStore::from_path(dir.path().join("chatops-submit-audit.jsonl"));
    let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
        vec!["trusted-user".to_string()],
        vec!["trusted-channel".to_string()],
        vec!["submit:ai-plan".to_string()],
    );
    let notifier = RecordingChatOpsNotifier::new();
    let envelope = ChatOpsCommandEnvelope::new(
        ChatOpsPlatform::Discord,
        "trusted-channel",
        "trusted-user",
        "submit ai-plan harden service control plane",
        1_737_200_000_222,
    )
    .with_transport(IngressTransport::DiscordInteractions)
    .with_request_id("discord-req-9000")
    .with_correlation_id("corr-discord-9000");

    let status = execute_chatops_envelope(&envelope, &policy, &notifier, Some(&audit_store))
        .await
        .expect("authorized chatops submit should execute");
    assert_eq!(status.to_string(), "success");

    let entries = audit_store
        .load_latest(16)
        .expect("audit entries should be readable");
    let executed = entries
        .iter()
        .find(|entry| {
            entry.internal_command.as_deref()
                == Some("/task submit ai-plan harden service control plane")
        })
        .expect("executed audit entry should exist");
    assert_eq!(executed.request_id.as_deref(), Some("discord-req-9000"));
    assert_eq!(
        executed.correlation_id.as_deref(),
        Some("corr-discord-9000")
    );
    assert_eq!(
        executed.operator_id.as_deref(),
        Some("discord:trusted-user")
    );
    assert_eq!(
        executed.session_id.as_deref(),
        Some("chatops-discord-trusted-user-trusted-channel")
    );
    assert_eq!(
        executed.transport,
        Some(IngressTransport::DiscordInteractions)
    );
    assert!(executed.task_id.as_deref().is_some());
}
