// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one result shape every CLI command and MCP tool returns (plan section 09), and the error codes.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The error codes in plan section 09.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ProjectNotFound,
    ProjectUntrusted,
    ModelExecFailed,
    ModelTimeout,
    ModelTooLarge,
    UnknownPart,
    UnknownColor,
    ParamInvalid,
    LibraryMissing,
    LibraryCorrupt,
    PythonMissing,
    RenderUnavailable,
    IoError,
    Internal,
}

impl ErrorCode {
    /// The process exit code: 2 usage, 3 runtime, 4 environment (library or Python missing or damaged, render unavailable).
    pub fn exit_code(self) -> i32 {
        match self {
            Self::ProjectNotFound => 2,
            Self::LibraryMissing | Self::LibraryCorrupt | Self::PythonMissing | Self::RenderUnavailable => 4,
            _ => 3,
        }
    }
}

/// Where an error happened and what else helps fix it. Every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Details {
    /// The file: relative to the project root for model errors, otherwise the full path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 1-based line in `file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// The source line, as written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Close matches; part ids for `UNKNOWN_PART` (color and parameter suggestions are in the hint).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

/// A failure: what happened, where, and what to try.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub hint: String,
    // Boxed, so a `Result` carrying an `Error` stays small.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Box<Details>>,
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: hint.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Details) -> Self {
        self.details = Some(Box::new(details));
        self
    }

    /// An error about a file with no line, such as a path that couldn't be read.
    pub fn at_file(mut self, file: impl Into<String>) -> Self {
        self.details.get_or_insert_with(Default::default).file = Some(file.into());
        self
    }

    /// `IO_ERROR` for a failed filesystem operation on `path`.
    pub fn io(path: &std::path::Path, err: std::io::Error) -> Self {
        Self::new(
            ErrorCode::IoError,
            format!("{}: {err}", path.display()),
            "Check that the path exists and you can write to it, then try again.",
        )
        .at_file(path.display().to_string())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::Internal,
            message,
            "This is a bug in Emmet. Run the command again with --json and include the output in a bug report.",
        )
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A non-fatal note attached to a success.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Warning {
    pub code: String,
    pub message: String,
}

/// Facts about the run that produced a result.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Meta {
    /// The Emmet version.
    pub emmet: String,
    /// The project root, when the command ran in one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// sha256 of the model's inputs (plan section 07), after a run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_hash: Option<String>,
    /// The project's run counter, after a run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    /// Wall time of the command in milliseconds.
    pub ms: u64,
}

/// `{ ok, data | error, warnings, meta }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Envelope<T> {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
    pub warnings: Vec<Warning>,
    pub meta: Meta,
}

impl<T> Envelope<T> {
    pub fn from_result(result: Result<T>, warnings: Vec<Warning>, meta: Meta) -> Self {
        match result {
            Ok(data) => Self {
                ok: true,
                data: Some(data),
                error: None,
                warnings,
                meta,
            },
            Err(error) => Self {
                ok: false,
                data: None,
                error: Some(error),
                warnings,
                meta,
            },
        }
    }
}
