use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum DashScopeError {
    #[error("http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to deserialize api response: {source}")]
    JSONDeserialize {
        source: serde_json::Error,
        raw_response: Vec<u8>,
    },
    #[error("{0}")]
    ApiError(ApiError),
    #[error("invalid argument:{0}")]
    InvalidArgument(String),
    #[error("stream error:{0}")]
    StreamError(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiError {
    pub message: String,
    pub request_id: Option<String>,
    pub code: Option<String>,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if let Some(code) = &self.code {
            parts.push(format!("code: {}", code));
        }
        if let Some(request_id) = &self.request_id {
            parts.push(format!("request_id: {}", request_id));
        }
        write!(f, "{}", parts.join(" "))
    }
}

impl From<crate::operation::common::ParametersBuilderError> for DashScopeError {
    fn from(error: crate::operation::common::ParametersBuilderError) -> Self {
        DashScopeError::InvalidArgument(error.to_string())
    }
}

pub(crate) fn map_deserialization_error(e: serde_json::Error, bytes: &[u8]) -> DashScopeError {
    tracing::error!(
        "failed deserialization of: {}",
        String::from_utf8_lossy(bytes)
    );
    DashScopeError::JSONDeserialize {
        source: e,
        raw_response: bytes.to_vec(),
    }
}

pub type Result<T> = std::result::Result<T, DashScopeError>;
