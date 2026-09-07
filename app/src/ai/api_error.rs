use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use warp_errors::{AnyhowErrorExt, ErrorExt, register_error};

use crate::ai::byop_readiness::BlockedByopReadinessError;

const INFINISHELL_ERROR_CODE_HEADER: &str = "X-InfiniShell-Error-Code";
const LEGACY_ZAP_ERROR_CODE_HEADER: &str = "X-Zap-Error-Code";
const WARP_ERROR_CODE_OUT_OF_CREDITS: &str = "OUT_OF_CREDITS";

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
#[error("{error}")]
pub struct ClientError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum DeserializationError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum AIApiError {
    #[error("Request failed due to lack of AI quota.")]
    QuotaLimit,

    #[error("InfiniShell is currently overloaded. Please try again later.")]
    ServerOverloaded,

    #[error("Internal error occurred at transport layer.")]
    Transport(#[source] reqwest::Error),

    #[error("Failed to deserialize API response.")]
    Deserialization(#[source] DeserializationError),

    #[error("No context found on context search.")]
    NoContextFound,

    #[error("Failed with status code {0}: {1}")]
    ErrorStatus(http::StatusCode, String),

    #[error("Provider response protocol error: {0}")]
    ProviderProtocol(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("Got error when streaming {stream_type}: {source:#}")]
    Stream {
        stream_type: &'static str,
        #[source]
        source: anyhow::Error,
    },
}

impl From<http_client::ResponseError> for AIApiError {
    fn from(err: http_client::ResponseError) -> Self {
        Self::from_response_error(err.source, &err.headers)
    }
}

impl From<reqwest::Error> for AIApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::from_transport_error(err)
    }
}

impl From<serde_json::Error> for AIApiError {
    fn from(err: serde_json::Error) -> Self {
        AIApiError::Deserialization(err.into())
    }
}

impl AIApiError {
    /// 兼容网关可能把上下文超限包装成 502 或流错误，不能仅按 HTTP 状态重试。
    pub fn is_context_window_exceeded(&self) -> bool {
        match self {
            Self::ErrorStatus(_, message) | Self::ProviderProtocol(message) => {
                is_context_window_exceeded_message(message)
            }
            Self::Other(error) | Self::Stream { source: error, .. } => error
                .chain()
                .any(|source| is_context_window_exceeded_message(&source.to_string())),
            Self::QuotaLimit
            | Self::ServerOverloaded
            | Self::Transport(_)
            | Self::Deserialization(_)
            | Self::NoContextFound => false,
        }
    }

    fn from_response_error(err: reqwest::Error, headers: &::http::HeaderMap) -> Self {
        if err.status() == Some(http::StatusCode::TOO_MANY_REQUESTS) {
            return Self::error_for_429(headers);
        }

        Self::from_transport_error(err)
    }

    fn from_transport_error(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            return AIApiError::Transport(err);
        }
        if err.is_decode() {
            #[cfg(not(target_family = "wasm"))]
            {
                use std::error::Error as _;
                let mut source = err.source();
                while let Some(underlying) = source {
                    if underlying.is::<hyper::Error>() {
                        return AIApiError::Transport(err);
                    }

                    source = underlying.source();
                }
            }

            return AIApiError::Deserialization(DeserializationError::Transport(err));
        }

        AIApiError::Transport(err)
    }

    fn error_for_429(headers: &::http::HeaderMap) -> Self {
        if headers
            .get(INFINISHELL_ERROR_CODE_HEADER)
            .or_else(|| headers.get(LEGACY_ZAP_ERROR_CODE_HEADER))
            .and_then(|v| v.to_str().ok())
            == Some(WARP_ERROR_CODE_OUT_OF_CREDITS)
        {
            AIApiError::QuotaLimit
        } else {
            AIApiError::ServerOverloaded
        }
    }

    pub(crate) async fn from_stream_error(
        stream_type: &'static str,
        err: reqwest_eventsource::Error,
    ) -> Self {
        match err {
            reqwest_eventsource::Error::InvalidStatusCode(
                http::StatusCode::TOO_MANY_REQUESTS,
                ref res,
            ) => Self::error_for_429(res.headers()),
            reqwest_eventsource::Error::InvalidStatusCode(status, res) => Self::ErrorStatus(
                status,
                res.text()
                    .await
                    .unwrap_or_else(|e| format!("(no response body: {e:#})")),
            ),
            reqwest_eventsource::Error::Transport(err) => Self::from_transport_error(err),
            err => AIApiError::Stream {
                stream_type,
                #[cfg(target_family = "wasm")]
                source: anyhow!("{err:#?}"),
                #[cfg(not(target_family = "wasm"))]
                source: anyhow!(err),
            },
        }
    }

    pub fn is_retryable(&self) -> bool {
        if self.is_context_window_exceeded() {
            return false;
        }

        fn is_retryable_status(status: http::StatusCode) -> bool {
            !status.is_client_error()
                || status == http::StatusCode::REQUEST_TIMEOUT
                || status == http::StatusCode::TOO_MANY_REQUESTS
        }

        match self {
            AIApiError::ErrorStatus(status, _) => is_retryable_status(*status),
            AIApiError::Transport(e) => {
                if let Some(status) = e.status() {
                    return is_retryable_status(status);
                }
                true
            }
            AIApiError::QuotaLimit
            | AIApiError::ServerOverloaded
            | AIApiError::Deserialization(_)
            | AIApiError::NoContextFound
            | AIApiError::Stream { .. } => true,
            AIApiError::ProviderProtocol(_) => false,
            AIApiError::Other(error) => error.downcast_ref::<BlockedByopReadinessError>().is_none(),
        }
    }
}

#[cfg(test)]
#[path = "api_error_tests.rs"]
mod tests;

impl ErrorExt for AIApiError {
    fn is_actionable(&self) -> bool {
        if self.is_context_window_exceeded() {
            return false;
        }

        match self {
            AIApiError::Deserialization(_) => true,
            AIApiError::Transport(error) => error.is_actionable(),
            AIApiError::Other(error) => error.is_actionable(),
            AIApiError::Stream { source, .. } => source.is_actionable(),
            AIApiError::ErrorStatus(_, _) => self.is_retryable(),
            AIApiError::QuotaLimit
            | AIApiError::ServerOverloaded
            | AIApiError::NoContextFound
            | AIApiError::ProviderProtocol(_) => false,
        }
    }
}

/// 只接受明确的超限错误码或输入超限描述，避免把输出 token 耗尽和普通 502 误分类。
pub(crate) fn is_context_window_exceeded_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message
        .split(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_'))
        .any(|part| matches!(part, "context_window_exceeded" | "context_length_exceeded"))
        || [
            "input exceeds the context window",
            "input exceeded the context window",
            "input exceeds context window",
            "input exceeded context window",
        ]
        .iter()
        .any(|phrase| message.contains(phrase))
}

register_error!(AIApiError);
