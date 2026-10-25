// SPDX-License-Identifier: AGPL-3.0-or-later
//! The app-server messages Emmet sends, as types, and the fields it reads, as paths: both are checked against the
//! committed schema for each supported Codex (`schema/codex/<version>/`, `codex app-server generate-json-schema`), so
//! a Codex update that changes the protocol fails a test (`tests/it/tests/agent_codex.rs`) instead of a chat.

use serde::Serialize;
use serde_json::Value;

/// The Codex versions whose schema is committed, newest first.
pub const SCHEMAS: &[&str] = &["0.160.1"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub name: String,
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadStartParams {
    pub cwd: String,
    pub developer_instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadResumeParams {
    pub thread_id: String,
    pub cwd: String,
    pub developer_instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UserInput {
    Text { text: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartParams {
    pub thread_id: String,
    pub input: Vec<UserInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnSteerParams {
    pub thread_id: String,
    pub expected_turn_id: String,
    pub input: Vec<UserInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnInterruptParams {
    pub thread_id: String,
    pub turn_id: String,
}

/// The answer to a command or file-change approval request.
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalResponse {
    /// "accept", "acceptForSession", or "decline".
    pub decision: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElicitationResponse {
    /// "accept" or "decline".
    pub action: &'static str,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionsResponse {
    /// What's granted: the request's own permissions to allow, none to deny.
    pub permissions: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<&'static str>,
}

/// Every message Emmet writes, by schema file (relative to the version folder), as an example value. The test
/// validates each against its schema.
pub fn sent_examples() -> Vec<(&'static str, Value)> {
    fn to(v: &impl Serialize) -> Value {
        serde_json::to_value(v).expect("serializes")
    }
    vec![
        (
            "v1/InitializeParams.json",
            to(&InitializeParams {
                client_info: ClientInfo {
                    name: "emmet".into(),
                    title: "Emmet".into(),
                    version: "0.1.0".into(),
                },
            }),
        ),
        (
            "v2/ThreadStartParams.json",
            to(&ThreadStartParams {
                cwd: "/p".into(),
                developer_instructions: "x".into(),
                model: Some("m".into()),
            }),
        ),
        (
            "v2/ThreadResumeParams.json",
            to(&ThreadResumeParams {
                thread_id: "t".into(),
                cwd: "/p".into(),
                developer_instructions: "x".into(),
                model: None,
            }),
        ),
        (
            "v2/TurnStartParams.json",
            to(&TurnStartParams {
                thread_id: "t".into(),
                input: vec![UserInput::Text { text: "hi".into() }],
            }),
        ),
        (
            "v2/TurnSteerParams.json",
            to(&TurnSteerParams {
                thread_id: "t".into(),
                expected_turn_id: "u".into(),
                input: vec![UserInput::Text { text: "hi".into() }],
            }),
        ),
        (
            "v2/TurnInterruptParams.json",
            to(&TurnInterruptParams {
                thread_id: "t".into(),
                turn_id: "u".into(),
            }),
        ),
        (
            "CommandExecutionRequestApprovalResponse.json",
            to(&ApprovalResponse { decision: "accept" }),
        ),
        (
            "CommandExecutionRequestApprovalResponse.json",
            to(&ApprovalResponse {
                decision: "acceptForSession",
            }),
        ),
        (
            "CommandExecutionRequestApprovalResponse.json",
            to(&ApprovalResponse { decision: "decline" }),
        ),
        (
            "FileChangeRequestApprovalResponse.json",
            to(&ApprovalResponse {
                decision: "acceptForSession",
            }),
        ),
        (
            "FileChangeRequestApprovalResponse.json",
            to(&ApprovalResponse { decision: "decline" }),
        ),
        (
            "McpServerElicitationRequestResponse.json",
            to(&ElicitationResponse {
                action: "accept",
                content: serde_json::json!({}),
            }),
        ),
        (
            "McpServerElicitationRequestResponse.json",
            to(&ElicitationResponse {
                action: "decline",
                content: Value::Null,
            }),
        ),
        (
            "PermissionsRequestApprovalResponse.json",
            to(&PermissionsResponse {
                permissions: serde_json::json!({ "network": { "enabled": true } }),
                scope: Some("session"),
            }),
        ),
        (
            "PermissionsRequestApprovalResponse.json",
            to(&PermissionsResponse {
                permissions: serde_json::json!({}),
                scope: None,
            }),
        ),
    ]
}

/// Every field Emmet reads, as (schema file, path). A path step is a property name; `[type]` picks the variant of a
/// tagged union whose `type` is that value. The test resolves each through the schema's `$ref`s and unions.
pub const READS: &[(&str, &str)] = &[
    ("v1/InitializeResponse.json", "codexHome"),
    ("v2/GetAccountResponse.json", "requiresOpenaiAuth"),
    ("v2/ThreadStartResponse.json", "thread.id"),
    ("v2/ThreadStartResponse.json", "model"),
    ("v2/ThreadResumeResponse.json", "thread.id"),
    ("v2/TurnStartResponse.json", "turn.id"),
    ("v2/TurnStartedNotification.json", "turn.id"),
    ("v2/TurnCompletedNotification.json", "turn.status"),
    ("v2/TurnCompletedNotification.json", "turn.error"),
    ("v2/AgentMessageDeltaNotification.json", "itemId"),
    ("v2/AgentMessageDeltaNotification.json", "delta"),
    ("v2/ReasoningSummaryTextDeltaNotification.json", "itemId"),
    ("v2/ReasoningSummaryTextDeltaNotification.json", "delta"),
    ("v2/ItemStartedNotification.json", "item.[mcpToolCall].server"),
    ("v2/ItemStartedNotification.json", "item.[mcpToolCall].tool"),
    ("v2/ItemStartedNotification.json", "item.[mcpToolCall].arguments"),
    ("v2/ItemStartedNotification.json", "item.[commandExecution].command"),
    ("v2/ItemStartedNotification.json", "item.[fileChange].changes"),
    ("v2/ItemCompletedNotification.json", "item.[mcpToolCall].status"),
    ("v2/ItemCompletedNotification.json", "item.[mcpToolCall].result"),
    ("v2/ItemCompletedNotification.json", "item.[mcpToolCall].error"),
    ("v2/ItemCompletedNotification.json", "item.[commandExecution].exitCode"),
    (
        "v2/ItemCompletedNotification.json",
        "item.[commandExecution].aggregatedOutput",
    ),
    ("v2/ItemCompletedNotification.json", "item.[commandExecution].status"),
    ("v2/ItemCompletedNotification.json", "item.[fileChange].status"),
    ("v2/ItemCompletedNotification.json", "item.[agentMessage].text"),
    (
        "v2/ThreadTokenUsageUpdatedNotification.json",
        "tokenUsage.last.inputTokens",
    ),
    (
        "v2/ThreadTokenUsageUpdatedNotification.json",
        "tokenUsage.last.outputTokens",
    ),
    ("v2/ErrorNotification.json", "error.message"),
    ("v2/ErrorNotification.json", "error.codexErrorInfo"),
    ("v2/ErrorNotification.json", "willRetry"),
    ("CommandExecutionRequestApprovalParams.json", "command"),
    ("CommandExecutionRequestApprovalParams.json", "reason"),
    ("FileChangeRequestApprovalParams.json", "itemId"),
    ("FileChangeRequestApprovalParams.json", "reason"),
    ("PermissionsRequestApprovalParams.json", "permissions"),
    ("PermissionsRequestApprovalParams.json", "reason"),
    ("McpServerElicitationRequestParams.json", "serverName"),
];

/// The turn statuses the mapping knows, which the test checks are the schema's.
pub const TURN_STATUSES: &[&str] = &["completed", "interrupted", "failed", "inProgress"];

/// The server requests Emmet answers, which the test checks are the schema's `ServerRequest` methods.
pub const SERVER_REQUESTS: &[&str] = &[
    "item/commandExecution/requestApproval",
    "item/fileChange/requestApproval",
    "item/permissions/requestApproval",
    "mcpServer/elicitation/request",
];

/// The client requests and notification Emmet sends, which the test checks are the schema's.
pub const CLIENT_METHODS: &[&str] = &[
    "initialize",
    "account/read",
    "thread/start",
    "thread/resume",
    "turn/start",
    "turn/steer",
    "turn/interrupt",
];
