// K2.2a scaffolds the error taxonomy; variants/methods get consumed in
// K2.2b (explicit NotFound/Validation sites) and K2.2c (handler signature flip).
#![allow(dead_code)]

use axum::http::StatusCode;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "class", rename_all = "snake_case")]
pub(crate) enum MemdError {
    Store { message: String },
    NotFound { resource: String, id: String },
    Validation { field: String, reason: String },
    Conflict { reason: String },
    Internal { message: String },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecoveryAction {
    Retry,
    Reject,
    ReturnExisting,
}

impl MemdError {
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            MemdError::NotFound { .. } => StatusCode::NOT_FOUND,
            MemdError::Validation { .. } => StatusCode::BAD_REQUEST,
            MemdError::Conflict { .. } => StatusCode::CONFLICT,
            MemdError::Store { .. } | MemdError::Internal { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub(crate) fn class(&self) -> &'static str {
        match self {
            MemdError::Store { .. } => "store",
            MemdError::NotFound { .. } => "not_found",
            MemdError::Validation { .. } => "validation",
            MemdError::Conflict { .. } => "conflict",
            MemdError::Internal { .. } => "internal",
        }
    }

    pub(crate) fn recovery(&self) -> RecoveryAction {
        match self {
            MemdError::Store { .. } | MemdError::Internal { .. } => RecoveryAction::Retry,
            MemdError::Conflict { .. } => RecoveryAction::ReturnExisting,
            MemdError::NotFound { .. } | MemdError::Validation { .. } => RecoveryAction::Reject,
        }
    }

    pub(crate) fn message(&self) -> String {
        match self {
            MemdError::Store { message } => message.clone(),
            MemdError::NotFound { resource, id } => format!("{resource} not found: {id}"),
            MemdError::Validation { field, reason } => format!("{field}: {reason}"),
            MemdError::Conflict { reason } => reason.clone(),
            MemdError::Internal { message } => message.clone(),
        }
    }
}

impl From<anyhow::Error> for MemdError {
    fn from(err: anyhow::Error) -> Self {
        let message = format!("{err:#}");
        tracing::error!(
            error.class = "internal",
            error.recovery = "retry",
            error.message = %message,
            "anyhow error converted to MemdError::Internal"
        );
        MemdError::Internal { message }
    }
}

impl From<MemdError> for (StatusCode, String) {
    fn from(err: MemdError) -> Self {
        (err.status(), err.message())
    }
}
