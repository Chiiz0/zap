use super::*;
use crate::ai::api_error::is_context_window_exceeded_message;

#[test]
fn endpoint_preserves_version_path_and_query() {
    let client = ResponsesClient::new(
        Arc::new(Client::new_for_test()),
        "https://example.test/openai/v1?api-version=preview",
        "secret",
        Vec::new(),
    )
    .unwrap();

    let endpoint = client.endpoint("responses/input_tokens").unwrap();

    assert_eq!(endpoint.path(), "/openai/v1/responses/input_tokens");
    assert_eq!(endpoint.query(), Some("api-version=preview"));
}

#[test]
fn http_error_extracts_structured_code_without_echoing_body() {
    let error = http_error(
        http::StatusCode::BAD_REQUEST,
        r#"{"error":{"code":"invalid_request","message":"bad field","param":"input"}}"#,
    );

    match error {
        ResponsesClientError::Http {
            status,
            code,
            message,
        } => {
            assert_eq!(status, http::StatusCode::BAD_REQUEST);
            assert_eq!(code.as_deref(), Some("invalid_request"));
            assert_eq!(message, "bad field");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn http_context_overflow_code_survives_display_conversion() {
    let error = http_error(
        http::StatusCode::BAD_GATEWAY,
        r#"{"error":{"code":"context_window_exceeded","message":"Request too large"}}"#,
    );

    assert!(is_context_window_exceeded_message(&error.to_string()));
}

#[test]
fn websocket连接池键隔离凭据且不泄漏secret() {
    let first = ResponsesClient::new(
        Arc::new(Client::new_for_test()),
        "https://example.test/v1",
        "secret-one",
        vec![("X-Route".to_owned(), "a".to_owned())],
    )
    .unwrap();
    let same = ResponsesClient::new(
        Arc::new(Client::new_for_test()),
        "https://example.test/v1",
        "secret-one",
        vec![("X-Route".to_owned(), "a".to_owned())],
    )
    .unwrap();
    let other = ResponsesClient::new(
        Arc::new(Client::new_for_test()),
        "https://example.test/v1",
        "secret-two",
        vec![("X-Route".to_owned(), "a".to_owned())],
    )
    .unwrap();

    let key = first.websocket_connection_key();
    assert_eq!(key, same.websocket_connection_key());
    assert_ne!(key, other.websocket_connection_key());
    assert!(!key.contains("secret-one"));
}

#[test]
fn multi_agent_beta_header不会重复发送() {
    let client = ResponsesClient::new(
        Arc::new(Client::new_for_test()),
        "https://example.test/v1",
        "secret",
        vec![(
            "OpenAI-Beta".to_owned(),
            "other=v1, responses_multi_agent=v1".to_owned(),
        )],
    )
    .unwrap()
    .with_multi_agent_beta(true);

    assert_eq!(
        client
            .websocket_headers()
            .into_iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("OpenAI-Beta"))
            .count(),
        1
    );
}
