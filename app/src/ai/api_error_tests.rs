use std::sync::Arc;

use super::*;
use crate::ai::agent::RenderableAIError;
use crate::ai::byop_readiness::{BlockedByopReadinessError, ReadinessCategory};

#[test]
fn byop_blocked_readiness_error_is_not_retryable() {
    let error = AIApiError::Other(
        BlockedByopReadinessError::new(ReadinessCategory::MissingResultWithoutRepairSource).into(),
    );

    assert!(!error.is_retryable());
}

#[test]
fn provider_protocol_error_is_not_retried_blindly() {
    let error = AIApiError::ProviderProtocol("response.incomplete".to_owned());

    assert!(!error.is_retryable());
}

#[test]
fn context_overflow_wrapped_in_bad_gateway_is_not_retried() {
    let error = AIApiError::ErrorStatus(
        http::StatusCode::BAD_GATEWAY,
        r#"{"error":{"message":"Your input exceeds the context window of this model."}}"#
            .to_owned(),
    );

    assert!(error.is_context_window_exceeded());
    assert!(!error.is_retryable());
    assert!(!error.is_actionable());
}

#[test]
fn ordinary_bad_gateway_remains_retryable() {
    let error = AIApiError::ErrorStatus(
        http::StatusCode::BAD_GATEWAY,
        "Upstream connection reset".to_owned(),
    );

    assert!(!error.is_context_window_exceeded());
    assert!(error.is_retryable());
}

#[test]
fn structured_context_error_codes_are_recognized_without_matching_longer_identifiers() {
    assert!(is_context_window_exceeded_message(
        r#"{"error":{"code":"context_length_exceeded","message":"Request too large"}}"#
    ));
    assert!(is_context_window_exceeded_message(
        "CONTEXT_WINDOW_EXCEEDED: local request budget exceeded"
    ));
    assert!(!is_context_window_exceeded_message(
        "not_context_window_exceeded"
    ));
    assert!(!is_context_window_exceeded_message(
        "context_window_exceeded_count"
    ));
}

#[test]
fn context_overflow_in_other_error_source_stops_resume() {
    let error = AIApiError::Other(
        anyhow!("context_window_exceeded: local request budget exceeded")
            .context("BYOP request conversion failed"),
    );

    assert!(error.is_context_window_exceeded());
    assert!(!error.is_retryable());
    assert!(!error.is_actionable());
}

#[test]
fn context_overflow_in_protocol_errors_uses_specific_classification() {
    let error = AIApiError::ProviderProtocol(
        "Responses response.failed: Your input exceeds the context window".to_owned(),
    );

    assert!(error.is_context_window_exceeded());
    assert!(!error.is_retryable());
}

#[test]
fn context_overflow_in_stream_errors_is_not_retried() {
    let error = AIApiError::Stream {
        stream_type: "BYOP",
        source: anyhow!("Your input exceeded context window limit"),
    };

    assert!(error.is_context_window_exceeded());
    assert!(!error.is_retryable());
}

#[test]
fn output_limit_and_unrelated_errors_are_not_context_overflow() {
    assert!(!is_context_window_exceeded_message(
        "Responses response.incomplete: max_output_tokens"
    ));
    assert!(!is_context_window_exceeded_message(
        "Rate limit exceeded for tokens per minute"
    ));
    assert!(!is_context_window_exceeded_message(
        "The input does not exceed the context window"
    ));
    assert!(!is_context_window_exceeded_message(
        "Failed to read model context window configuration"
    ));
}

#[test]
fn context_overflow_uses_context_error_presentation() {
    let error = Arc::new(AIApiError::ErrorStatus(
        http::StatusCode::BAD_GATEWAY,
        "Your input exceeds the context window. Upstream diagnostic details".to_owned(),
    ));
    let renderable = RenderableAIError::from(&error);

    let RenderableAIError::ContextWindowExceeded(message) = renderable else {
        panic!("上下文超限应使用专门的错误展示");
    };
    assert!(!message.contains("Upstream diagnostic details"));
}

#[test]
fn current_infinishell_quota_header_is_recognized() {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        INFINISHELL_ERROR_CODE_HEADER,
        http::HeaderValue::from_static(WARP_ERROR_CODE_OUT_OF_CREDITS),
    );

    assert!(matches!(
        AIApiError::error_for_429(&headers),
        AIApiError::QuotaLimit
    ));
}

#[test]
fn legacy_zap_quota_header_is_recognized() {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        LEGACY_ZAP_ERROR_CODE_HEADER,
        http::HeaderValue::from_static(WARP_ERROR_CODE_OUT_OF_CREDITS),
    );

    assert!(matches!(
        AIApiError::error_for_429(&headers),
        AIApiError::QuotaLimit
    ));
}
