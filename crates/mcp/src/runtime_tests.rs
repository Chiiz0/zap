use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::post;
use rmcp::model::{ErrorCode, ErrorData, Resource, ServerCapabilities, Tool};
use warp_errors::{AnyhowErrorExt as _, ErrorExt as _};

use super::{
    build_client_with_headers, determine_transport, has_caller_supplied_credential,
    is_oauth_challenge, query_resources_for, query_tools_for, should_query_resources,
    should_query_tools,
};
use crate::oauth::McpAuthenticationError;

/// Build a `ServerCapabilities` with selected capability flags toggled on.
/// Each `Some(default)` mirrors how rmcp deserializes a capability the
/// server advertised with no inner flags set.
fn caps(tools: bool, resources: bool) -> ServerCapabilities {
    match (tools, resources) {
        (true, true) => ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build(),
        (true, false) => ServerCapabilities::builder().enable_tools().build(),
        (false, true) => ServerCapabilities::builder().enable_resources().build(),
        (false, false) => ServerCapabilities::builder().build(),
    }
}

fn test_tool(name: &str) -> Tool {
    serde_json::from_value(serde_json::json!({
        "name": name,
        "description": "test tool",
        "inputSchema": { "type": "object" },
    }))
    .expect("Tool deserialization")
}

fn test_resource(uri: &str) -> Resource {
    serde_json::from_value(serde_json::json!({
        "uri": uri,
        "name": "test resource",
    }))
    .expect("Resource deserialization")
}

/// Regression test for warpdotdev/warp#6798: each capability is queried
/// independently. Previously, asymmetric handling could cause `tools/list`
/// to be skipped when a server advertised both `tools` and `resources`,
/// resulting in "No tools available" even though the server had tools.
#[test]
fn each_capability_is_queried_independently() {
    for has_tools in [false, true] {
        for has_resources in [false, true] {
            let c = caps(has_tools, has_resources);
            assert_eq!(
                should_query_tools(Some(&c)),
                has_tools,
                "tools={has_tools}, resources={has_resources}",
            );
            assert_eq!(
                should_query_resources(Some(&c)),
                has_resources,
                "tools={has_tools}, resources={has_resources}",
            );
        }
    }
    assert!(!should_query_tools(None));
    assert!(!should_query_resources(None));
}

/// When `tools` is not advertised, the helper must skip the list call so
/// we don't waste a round trip and pollute the wire log with a request
/// that's destined to return `METHOD_NOT_FOUND`.
#[tokio::test]
async fn query_tools_for_skips_listing_when_capability_not_advertised() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();
    let no_caps = caps(false, false);

    let result = query_tools_for(Some(&no_caps), "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_tool("never")])
    })
    .await;

    assert!(result.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Skips `tools/list` when server info is absent.
#[tokio::test]
async fn query_tools_for_skips_listing_when_server_info_is_none() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();

    let result = query_tools_for(None, "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_tool("never")])
    })
    .await;

    assert!(result.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Returns listed tools when `tools` is advertised.
#[tokio::test]
async fn query_tools_for_returns_listed_tools_when_capability_advertised() {
    let c = caps(true, false);
    let expected = vec![test_tool("greet"), test_tool("review")];
    let to_return = expected.clone();

    let result = query_tools_for(Some(&c), "srv", || async move { Ok(to_return) }).await;

    assert_eq!(result, expected);
}

/// Returns an empty vector when the server lists no tools.
#[tokio::test]
async fn query_tools_for_returns_empty_vec_when_server_lists_no_tools() {
    let c = caps(true, false);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();

    let result = query_tools_for(Some(&c), "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    })
    .await;

    assert!(result.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// **The fail-soft test the bug ticket implicitly demands.** Transport-
/// closed errors must not abort server startup; the helper must log and
/// return an empty vec. This is the regression-protector for #6798's
/// underlying asymmetry — if anyone re-introduces a `return Err(...)` here,
/// this test fails.
#[tokio::test]
async fn query_tools_for_returns_empty_on_transport_error() {
    let c = caps(true, false);
    let result = query_tools_for(Some(&c), "srv", || async {
        Err(rmcp::ServiceError::TransportClosed)
    })
    .await;
    assert!(result.is_empty());
}

/// MCP-protocol errors (e.g. METHOD_NOT_FOUND from a misbehaving server
/// that advertised the capability but rejects the call) also fail soft,
/// so the rest of the server surface still comes up.
#[tokio::test]
async fn query_tools_for_returns_empty_on_mcp_error() {
    let c = caps(true, false);
    let result = query_tools_for(Some(&c), "srv", || async {
        Err(rmcp::ServiceError::McpError(ErrorData {
            code: ErrorCode::METHOD_NOT_FOUND,
            message: "tools/list not implemented".into(),
            data: None,
        }))
    })
    .await;
    assert!(result.is_empty());
}

/// Calls the `tools/list` function exactly once per query.
#[tokio::test]
async fn query_tools_for_calls_list_function_exactly_once() {
    let c = caps(true, false);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();

    let _ = query_tools_for(Some(&c), "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_tool("p")])
    })
    .await;

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// Keeps the tools-listing decision independent of resource capability state.
#[tokio::test]
async fn query_tools_for_decision_independent_of_other_capabilities() {
    let tools = vec![test_tool("x")];
    for has_tools in [false, true] {
        for has_resources in [false, true] {
            let c = caps(has_tools, has_resources);
            let to_return = tools.clone();
            let result = query_tools_for(Some(&c), "srv", || async move { Ok(to_return) }).await;

            if has_tools {
                assert_eq!(result, tools);
            } else {
                assert!(result.is_empty());
            }
        }
    }
}

/// Skips `resources/list` when `resources` is not advertised.
#[tokio::test]
async fn query_resources_for_skips_listing_when_capability_not_advertised() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();
    let no_caps = caps(false, false);

    let result = query_resources_for(Some(&no_caps), "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_resource("file:///nope")])
    })
    .await;

    assert!(result.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Skips `resources/list` when server info is absent.
#[tokio::test]
async fn query_resources_for_skips_listing_when_server_info_is_none() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();

    let result = query_resources_for(None, "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_resource("file:///nope")])
    })
    .await;

    assert!(result.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Returns listed resources when `resources` is advertised.
#[tokio::test]
async fn query_resources_for_returns_listed_resources_when_capability_advertised() {
    let c = caps(false, true);
    let expected = vec![test_resource("file:///a"), test_resource("file:///b")];
    let to_return = expected.clone();

    let result = query_resources_for(Some(&c), "srv", || async move { Ok(to_return) }).await;

    assert_eq!(result, expected);
}

/// Fails soft when `resources/list` sees a transport error.
#[tokio::test]
async fn query_resources_for_returns_empty_on_transport_error() {
    let c = caps(false, true);
    let result = query_resources_for(Some(&c), "srv", || async {
        Err(rmcp::ServiceError::TransportClosed)
    })
    .await;
    assert!(result.is_empty());
}

/// Fails soft when `resources/list` returns an MCP protocol error.
#[tokio::test]
async fn query_resources_for_returns_empty_on_mcp_error() {
    let c = caps(false, true);
    let result = query_resources_for(Some(&c), "srv", || async {
        Err(rmcp::ServiceError::McpError(ErrorData {
            code: ErrorCode::METHOD_NOT_FOUND,
            message: "resources/list not implemented".into(),
            data: None,
        }))
    })
    .await;
    assert!(result.is_empty());
}

/// Calls the `resources/list` function exactly once per query.
#[tokio::test]
async fn query_resources_for_calls_list_function_exactly_once() {
    let c = caps(false, true);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = calls.clone();

    let _ = query_resources_for(Some(&c), "srv", || async move {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        Ok(vec![test_resource("file:///a")])
    })
    .await;

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

#[test]
fn oauth_challenge_requires_resource_metadata_parameter() {
    assert!(is_oauth_challenge(
        r#"Bearer resource_metadata="https://example.com/.well-known/oauth-protected-resource""#
    ));
    assert!(is_oauth_challenge(
        r#"bearer RESOURCE_METADATA = "https://example.com/metadata""#
    ));
    assert!(!is_oauth_challenge("Bearer"));
    assert!(!is_oauth_challenge(r#"Bearer error="invalid_token""#));
    assert!(!is_oauth_challenge(r#"Basic realm="mcp""#));
}

#[test]
fn oauth_challenge_ignores_parameter_names_inside_quoted_values() {
    assert!(!is_oauth_challenge(
        r#"Bearer error_description="mentions \" resource_metadata=\"fake\"", error="invalid_token""#
    ));
    assert!(is_oauth_challenge(
        r#"Bearer error_description="missing resource_metadata", resource_metadata="https://example.com""#
    ));
    assert!(!is_oauth_challenge(
        r#"Bearer error_description="unterminated resource_metadata="https://example.com"#
    ));
}

#[test]
fn only_nonempty_static_auth_headers_count_as_credentials() {
    assert!(has_caller_supplied_credential(&headers(&[(
        "aUtHoRiZaTiOn",
        "Bearer token"
    )])));
    assert!(has_caller_supplied_credential(&headers(&[(
        "X-Api-Key",
        "token"
    )])));
    assert!(has_caller_supplied_credential(&headers(&[(
        "API-KEY", "token"
    )])));
    assert!(!has_caller_supplied_credential(&headers(&[(
        "Authorization",
        "   "
    )])));
    assert!(!has_caller_supplied_credential(&headers(&[(
        "Content-Type",
        "application/json"
    )])));
    assert!(!has_caller_supplied_credential(&HashMap::new()));
}

#[test]
fn discarded_invalid_headers_do_not_count_as_sent_credentials() {
    assert!(!has_caller_supplied_credential(&headers(&[(
        "Authorization",
        "Bearer invalid\ntoken"
    )])));
    assert!(!has_caller_supplied_credential(&headers(&[
        ("Authorization", "Bearer valid-token"),
        ("Invalid Header", "value"),
    ])));
}

/// 使用真实的本地 HTTP 服务器验证认证分支和未发送请求这一安全边界。
async fn serve_401(
    challenges: &'static [&'static str],
) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    crate::install_test_crypto_provider();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("测试服务器应能绑定");
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let handler_requests = requests.clone();
    let app = Router::new().route(
        "/mcp",
        post(move || {
            let requests = handler_requests.clone();
            async move {
                requests.fetch_add(1, Ordering::SeqCst);
                let mut response = axum::http::StatusCode::UNAUTHORIZED.into_response();
                for challenge in challenges {
                    response.headers_mut().append(
                        axum::http::header::WWW_AUTHENTICATE,
                        axum::http::HeaderValue::from_static(challenge),
                    );
                }
                response
            }
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (url, requests, server)
}

fn authentication_error(error: &rmcp::RmcpError) -> &McpAuthenticationError {
    let rmcp::RmcpError::TransportCreation { error, .. } = error else {
        panic!("应保留传输层认证错误：{error}");
    };
    error
        .downcast_ref::<McpAuthenticationError>()
        .expect("应保留可分类的认证错误")
}

#[tokio::test]
async fn bare_401_with_configured_credential_reports_rejection_not_oauth() {
    let (url, requests, server) = serve_401(&[]).await;

    let error = determine_transport(
        "static-credential-server".to_string(),
        &url,
        &headers(&[("Authorization", "Bearer rejected-private-token")]),
        None,
    )
    .await
    .err()
    .expect("被拒绝的凭据应阻止连接");

    assert!(matches!(
        authentication_error(&error),
        McpAuthenticationError::CredentialsRejected
    ));
    assert!(!format!("{error:?}").contains("rejected-private-token"));
    assert!(!authentication_error(&error).is_actionable());
    assert!(!anyhow::Error::new(error).is_actionable());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn plain_bearer_challenge_with_api_key_reports_rejection() {
    let (url, requests, server) = serve_401(&[r#"Bearer error="invalid_token""#]).await;

    let error = determine_transport(
        "api-key-server".to_string(),
        &url,
        &headers(&[("X-Api-Key", "rejected-api-key")]),
        None,
    )
    .await
    .err()
    .expect("普通 Bearer challenge 不应触发 OAuth");

    assert!(matches!(
        authentication_error(&error),
        McpAuthenticationError::CredentialsRejected
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn oauth_challenge_in_repeated_header_overrides_static_credential_rejection() {
    let (url, requests, server) = serve_401(&[
        r#"Bearer error="invalid_token""#,
        r#"Bearer resource_metadata = "https://example.com/metadata""#,
    ])
    .await;

    let error = determine_transport(
        "oauth-server".to_string(),
        &url,
        &headers(&[("Authorization", "Bearer expired-token")]),
        None,
    )
    .await
    .err()
    .expect("应走到缺少认证上下文的 OAuth 分支");

    assert!(matches!(
        authentication_error(&error),
        McpAuthenticationError::OAuthUnavailable
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn bare_401_without_configured_credentials_still_attempts_oauth() {
    let (url, requests, server) = serve_401(&[]).await;

    let error = determine_transport("oauth-server".to_string(), &url, &HashMap::new(), None)
        .await
        .err()
        .expect("应走到缺少认证上下文的 OAuth 分支");

    assert!(matches!(
        authentication_error(&error),
        McpAuthenticationError::OAuthUnavailable
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn unresolved_header_secret_fails_before_sending_a_request() {
    let (url, requests, server) = serve_401(&[]).await;

    let error = determine_transport(
        "unresolved-secret-server".to_string(),
        &url,
        &headers(&[(
            "Authorization",
            "Bearer sensitive-prefix-{{MISSING_SECRET}}",
        )]),
        None,
    )
    .await
    .err()
    .expect("未解析的密钥应阻止预检请求");

    let McpAuthenticationError::UnresolvedHeaderSecrets { header, secrets } =
        authentication_error(&error)
    else {
        panic!("应报告未解析密钥：{error}");
    };
    assert_eq!(header, "Authorization");
    assert_eq!(secrets, &["MISSING_SECRET"]);
    assert!(!format!("{error:?}").contains("sensitive-prefix"));
    assert!(!format!("{error}").contains("MISSING_SECRET"));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    server.abort();
}

#[test]
fn custom_headers_also_reject_unresolved_secrets() {
    let error = build_client_with_headers(&headers(&[(
        "X-Custom-Credential",
        "{{SECOND}}/{{FIRST}}/{{FIRST}}",
    )]))
    .expect_err("自定义头也必须检查密钥引用");

    let McpAuthenticationError::UnresolvedHeaderSecrets { header, secrets } =
        authentication_error(&error)
    else {
        panic!("应报告未解析密钥：{error}");
    };
    assert_eq!(header, "X-Custom-Credential");
    assert_eq!(secrets, &["FIRST", "SECOND"]);
}

#[test]
fn header_json_and_literal_braces_are_not_secret_references() {
    crate::install_test_crypto_provider();
    assert!(
        build_client_with_headers(&headers(&[
            ("X-Json", r#"{"kind":"test","nested":{"value":1}}"#),
            ("X-Literal", "{not-a-template}"),
            ("Authorization", "Bearer resolved-token"),
        ]))
        .is_ok()
    );
}
