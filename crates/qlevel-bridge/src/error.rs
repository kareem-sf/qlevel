//! Structured IPC-safe error types for `qlevel-bridge` with zero secret leakage.

use qlevel_ai::AiError;
use qlevel_engine::EngineError;
use qlevel_parser::ParserError;
use qlevel_workbook::WorkbookError;
use serde::{Serialize, Serializer};

use crate::dto::{BridgeErrorCode, BridgeErrorPayload};

/// Canonical error returned by `qlevel-bridge` commands and services.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Authentication required or expired: {0}")]
    AuthExpired(String),

    #[error("ChatGPT subscription usage cap reached; manage quota at {manage_url}")]
    OpenAiUsageLimitExceeded { manage_url: String },

    #[error("Gemini Free Tier confidentiality notice must be acknowledged before sending tender documents")]
    GeminiFreeTierUnacknowledged,

    #[error("File '{file_name}' is password-protected; please save an unencrypted copy first")]
    FilePasswordProtected { file_name: String },

    #[error("File '{file_name}' could not be parsed: {reason}")]
    FileCorrupted { file_name: String, reason: String },

    #[error("Target workbook '{locked_path}' is currently open in Excel")]
    ExcelFileLocked {
        locked_path: String,
        suggested_fallback_path: String,
    },

    #[error("At least one vendor quotation is required to start comparison")]
    MissingVendorQuote,

    #[error("Leveling session was cancelled")]
    Cancelled,

    #[error("Network connection error: {0}")]
    Network(String),

    #[error("OS credential store unavailable: {0}")]
    VaultUnavailable(String),

    #[error("Application folder '{path}' is read-only; move portable executable to a user folder to update")]
    UpdateDirNotWritable { path: String },

    #[error("Update SHA-256 checksum mismatch (expected {expected}, got {actual})")]
    UpdateChecksumMismatch { expected: String, actual: String },

    #[error("{0}")]
    Internal(String),
}

impl BridgeError {
    /// Converts this error into a structured `BridgeErrorPayload` for the TypeScript UI.
    #[must_use]
    pub fn to_payload(&self) -> BridgeErrorPayload {
        match self {
            Self::AuthExpired(msg) => BridgeErrorPayload {
                code: BridgeErrorCode::AuthExpired,
                message: msg.clone(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::OpenAiUsageLimitExceeded { manage_url } => BridgeErrorPayload {
                code: BridgeErrorCode::OpenaiUsageLimitExceeded,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: Some(manage_url.clone()),
                suggested_fallback_path: None,
            },
            Self::GeminiFreeTierUnacknowledged => BridgeErrorPayload {
                code: BridgeErrorCode::GeminiFreeTierBlocked,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::FilePasswordProtected { file_name } => BridgeErrorPayload {
                code: BridgeErrorCode::FilePasswordProtected,
                message: self.to_string(),
                affected_file_name: Some(file_name.clone()),
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::FileCorrupted { file_name, .. } => BridgeErrorPayload {
                code: BridgeErrorCode::FileCorrupted,
                message: self.to_string(),
                affected_file_name: Some(file_name.clone()),
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::ExcelFileLocked {
                suggested_fallback_path,
                ..
            } => BridgeErrorPayload {
                code: BridgeErrorCode::ExcelFileLocked,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: Some(suggested_fallback_path.clone()),
            },
            Self::MissingVendorQuote => BridgeErrorPayload {
                code: BridgeErrorCode::MissingVendorQuote,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::Cancelled => BridgeErrorPayload {
                code: BridgeErrorCode::Cancelled,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::Network(msg) => BridgeErrorPayload {
                code: BridgeErrorCode::NetworkError,
                message: msg.clone(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
            Self::VaultUnavailable(_)
            | Self::UpdateDirNotWritable { .. }
            | Self::UpdateChecksumMismatch { .. }
            | Self::Internal(_) => BridgeErrorPayload {
                code: BridgeErrorCode::InternalError,
                message: self.to_string(),
                affected_file_name: None,
                manage_usage_url: None,
                suggested_fallback_path: None,
            },
        }
    }
}

impl From<ParserError> for BridgeError {
    fn from(err: ParserError) -> Self {
        match err {
            ParserError::PasswordProtected { file_name } => {
                Self::FilePasswordProtected { file_name }
            }
            ParserError::WorkbookCorrupted { file_name, reason }
            | ParserError::PdfCorrupted { file_name, reason } => {
                Self::FileCorrupted { file_name, reason }
            }
            other => Self::Internal(other.to_string()),
        }
    }
}

impl From<AiError> for BridgeError {
    fn from(err: AiError) -> Self {
        match err {
            AiError::Unauthorized { detail, .. } => Self::AuthExpired(detail),
            AiError::OpenAiUsageLimitExceeded { manage_url, .. } => {
                Self::OpenAiUsageLimitExceeded { manage_url }
            }
            AiError::Network { detail, .. } => Self::Network(detail),
            AiError::Cancelled => Self::Cancelled,
            other => Self::Internal(other.to_string()),
        }
    }
}

impl From<EngineError> for BridgeError {
    fn from(err: EngineError) -> Self {
        match err {
            EngineError::Parser(p) => Self::from(p),
            EngineError::Ai(a) => Self::from(a),
            EngineError::NoVendorQuotes => Self::MissingVendorQuote,
            EngineError::Cancelled => Self::Cancelled,
            other => Self::Internal(other.to_string()),
        }
    }
}

impl From<WorkbookError> for BridgeError {
    fn from(err: WorkbookError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl Serialize for BridgeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}

/// Convenience result alias for `qlevel-bridge`.
pub type BridgeResult<T> = Result<T, BridgeError>;
