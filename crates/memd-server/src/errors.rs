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

impl RecoveryAction {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            RecoveryAction::Retry => "retry",
            RecoveryAction::Reject => "reject",
            RecoveryAction::ReturnExisting => "return_existing",
        }
    }
}

impl MemdError {
    pub(crate) fn validation(field: impl Into<String>, reason: impl Into<String>) -> Self {
        MemdError::Validation {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn not_found(resource: impl Into<String>, id: impl std::fmt::Display) -> Self {
        MemdError::NotFound {
            resource: resource.into(),
            id: id.to_string(),
        }
    }

    pub(crate) fn conflict(reason: impl Into<String>) -> Self {
        MemdError::Conflict {
            reason: reason.into(),
        }
    }

    pub(crate) fn into_wire(self) -> (StatusCode, String) {
        self.into()
    }

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
            MemdError::Validation { reason, .. } => reason.clone(),
            MemdError::Conflict { reason } => reason.clone(),
            MemdError::Internal { message } => message.clone(),
        }
    }
}

impl From<anyhow::Error> for MemdError {
    fn from(err: anyhow::Error) -> Self {
        MemdError::Internal {
            message: format!("{err:#}"),
        }
    }
}

impl From<MemdError> for (StatusCode, String) {
    fn from(err: MemdError) -> Self {
        let status = err.status();
        let class = err.class();
        let recovery = err.recovery();
        let message = err.message();
        let status_code = status.as_u16();

        match err {
            MemdError::Internal { .. } | MemdError::Store { .. } => {
                tracing::error!(
                    error.class = class,
                    error.recovery = recovery.label(),
                    error.status = status_code,
                    error.message = %message,
                    "memd error emitted to client"
                );
            }
            _ => {
                tracing::warn!(
                    error.class = class,
                    error.recovery = recovery.label(),
                    error.status = status_code,
                    error.message = %message,
                    "memd error emitted to client"
                );
            }
        }
        (status, message)
    }
}
