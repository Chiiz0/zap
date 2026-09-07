use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use http_client::{Client, RequestBuilder};
use reqwest_eventsource::Event;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;

use super::{
    CompactResponse, ConversationItemPage, ConversationObject, DeletedResponse, InputTokenCount,
    ResponseApiError, ResponseCreateRequest, ResponseItem, ResponseItemPage, ResponseProtocolError,
    ResponseRequestValidationError, ResponseStreamEvent,
};

#[cfg(not(target_family = "wasm"))]
pub type ResponseEventStream =
    Pin<Box<dyn Stream<Item = Result<ResponseStreamEvent, ResponsesClientError>> + Send + 'static>>;

#[cfg(target_family = "wasm")]
pub type ResponseEventStream =
    Pin<Box<dyn Stream<Item = Result<ResponseStreamEvent, ResponsesClientError>> + 'static>>;

#[derive(Debug, thiserror::Error)]
pub enum ResponsesClientError {
    #[error("Responses API base URL 无效")]
    InvalidBaseUrl(#[source] url::ParseError),
    #[error(transparent)]
    InvalidRequest(#[from] ResponseRequestValidationError),
    #[error("Responses HTTP 传输失败")]
    Transport(#[source] reqwest::Error),
    #[error("Responses HTTP {status}: {message} (code={code:?})")]
    Http {
        status: http::StatusCode,
        code: Option<String>,
        message: String,
    },
    #[error("Responses SSE 失败: {0}")]
    Stream(String),
    #[error("Responses SSE 在终止事件前结束，最后序号为 {last_sequence:?}")]
    StreamEnded { last_sequence: Option<u64> },
    #[error("Responses 返回无效 JSON")]
    Decode(#[source] serde_json::Error),
    #[error(transparent)]
    Protocol(#[from] ResponseProtocolError),
}

/// 原生 Responses HTTP 客户端。API key 仅保存在内存中，且本类型不实现 `Debug`。
#[derive(Clone)]
pub struct ResponsesClient {
    http: Arc<Client>,
    base_url: Url,
    api_key: String,
    extra_headers: Vec<(String, String)>,
    multi_agent_beta: bool,
}

impl ResponsesClient {
    pub fn new(
        http: Arc<Client>,
        base_url: &str,
        api_key: impl Into<String>,
        extra_headers: Vec<(String, String)>,
    ) -> Result<Self, ResponsesClientError> {
        let mut base_url =
            Url::parse(base_url.trim()).map_err(ResponsesClientError::InvalidBaseUrl)?;
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        Ok(Self {
            http,
            base_url,
            api_key: api_key.into(),
            extra_headers,
            multi_agent_beta: false,
        })
    }

    pub fn with_multi_agent_beta(mut self, enabled: bool) -> Self {
        self.multi_agent_beta = enabled;
        self
    }

    pub async fn create(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<super::ResponseObject, ResponsesClientError> {
        request.validate_http()?;
        request.stream = Some(false);
        self.post_json("responses", &request).await
    }

    pub fn create_stream(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<ResponseEventStream, ResponsesClientError> {
        request.validate_http()?;
        request.stream = Some(true);
        let url = self.endpoint("responses")?;
        let builder = self.authorize(self.http.post(url)).json(&request);
        Ok(parse_event_stream(builder.eventsource()))
    }

    pub async fn retrieve(
        &self,
        response_id: &str,
        include: &[String],
    ) -> Result<super::ResponseObject, ResponsesClientError> {
        let url = self.response_url(response_id, include, false, None)?;
        self.get_json(url).await
    }

    pub fn resume_stream(
        &self,
        response_id: &str,
        starting_after: Option<u64>,
        include: &[String],
    ) -> Result<ResponseEventStream, ResponsesClientError> {
        let url = self.response_url(response_id, include, true, starting_after)?;
        let builder = self.authorize(self.http.get(url));
        Ok(parse_event_stream(builder.eventsource()))
    }

    pub async fn delete(&self, response_id: &str) -> Result<DeletedResponse, ResponsesClientError> {
        let url = self.endpoint(&format!("responses/{response_id}"))?;
        let response = self.authorize(self.http.delete(url)).send().await;
        parse_response(response).await
    }

    pub async fn cancel(
        &self,
        response_id: &str,
    ) -> Result<super::ResponseObject, ResponsesClientError> {
        self.post_json(&format!("responses/{response_id}/cancel"), &Value::Null)
            .await
    }

    pub async fn list_input_items(
        &self,
        response_id: &str,
        after: Option<&str>,
        limit: Option<u8>,
        ascending: bool,
        include: &[String],
    ) -> Result<ResponseItemPage, ResponsesClientError> {
        let mut url = self.endpoint(&format!("responses/{response_id}/input_items"))?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(after) = after {
                query.append_pair("after", after);
            }
            if let Some(limit) = limit {
                query.append_pair("limit", &limit.clamp(1, 100).to_string());
            }
            query.append_pair("order", if ascending { "asc" } else { "desc" });
            for value in include {
                query.append_pair("include[]", value);
            }
        }
        self.get_json(url).await
    }

    pub async fn count_input_tokens(
        &self,
        request: &ResponseCreateRequest,
    ) -> Result<InputTokenCount, ResponsesClientError> {
        let mut request = request.clone();
        request.stream = None;
        request.background = None;
        request.generate = None;
        self.post_json("responses/input_tokens", &request).await
    }

    pub async fn compact(
        &self,
        request: &ResponseCreateRequest,
    ) -> Result<CompactResponse, ResponsesClientError> {
        let mut request = request.clone();
        request.stream = None;
        request.background = None;
        request.generate = None;
        self.post_json("responses/compact", &request).await
    }

    pub async fn create_conversation(
        &self,
        metadata: Option<Value>,
        items: Vec<ResponseItem>,
    ) -> Result<ConversationObject, ResponsesClientError> {
        self.post_json(
            "conversations",
            &serde_json::json!({ "metadata": metadata, "items": items }),
        )
        .await
    }

    pub async fn retrieve_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<ConversationObject, ResponsesClientError> {
        self.get_json(self.endpoint(&format!("conversations/{conversation_id}"))?)
            .await
    }

    pub async fn update_conversation(
        &self,
        conversation_id: &str,
        metadata: Value,
    ) -> Result<ConversationObject, ResponsesClientError> {
        let url = self.endpoint(&format!("conversations/{conversation_id}"))?;
        let response = self
            .authorize(self.http.post(url))
            .json(&serde_json::json!({ "metadata": metadata }))
            .send()
            .await;
        parse_response(response).await
    }

    pub async fn delete_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<DeletedResponse, ResponsesClientError> {
        let url = self.endpoint(&format!("conversations/{conversation_id}"))?;
        let response = self.authorize(self.http.delete(url)).send().await;
        parse_response(response).await
    }

    pub async fn create_conversation_items(
        &self,
        conversation_id: &str,
        items: Vec<ResponseItem>,
    ) -> Result<ConversationItemPage, ResponsesClientError> {
        self.post_json(
            &format!("conversations/{conversation_id}/items"),
            &serde_json::json!({ "items": items }),
        )
        .await
    }

    pub async fn list_conversation_items(
        &self,
        conversation_id: &str,
        after: Option<&str>,
        limit: Option<u8>,
        ascending: bool,
    ) -> Result<ConversationItemPage, ResponsesClientError> {
        let mut url = self.endpoint(&format!("conversations/{conversation_id}/items"))?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(after) = after {
                query.append_pair("after", after);
            }
            if let Some(limit) = limit {
                query.append_pair("limit", &limit.clamp(1, 100).to_string());
            }
            query.append_pair("order", if ascending { "asc" } else { "desc" });
        }
        self.get_json(url).await
    }

    pub async fn retrieve_conversation_item(
        &self,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<ResponseItem, ResponsesClientError> {
        self.get_json(self.endpoint(&format!("conversations/{conversation_id}/items/{item_id}"))?)
            .await
    }

    pub async fn delete_conversation_item(
        &self,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<DeletedResponse, ResponsesClientError> {
        let url = self.endpoint(&format!("conversations/{conversation_id}/items/{item_id}"))?;
        let response = self.authorize(self.http.delete(url)).send().await;
        parse_response(response).await
    }

    pub(crate) fn websocket_url(&self) -> Result<Url, ResponsesClientError> {
        let mut url = self.endpoint("responses")?;
        let scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            _ => {
                return Err(ResponsesClientError::InvalidBaseUrl(
                    url::ParseError::RelativeUrlWithoutBase,
                ));
            }
        };
        url.set_scheme(scheme).map_err(|_| {
            ResponsesClientError::InvalidBaseUrl(url::ParseError::RelativeUrlWithoutBase)
        })?;
        Ok(url)
    }

    pub(crate) fn websocket_connection_key(&self) -> String {
        let fingerprint = serde_json::json!({
            "base_url": self.base_url.as_str(),
            "api_key": self.api_key,
            "extra_headers": self.extra_headers,
            "multi_agent_beta": self.multi_agent_beta,
        });
        hex::encode(Sha256::digest(fingerprint.to_string().as_bytes()))
    }

    pub(crate) fn websocket_headers(&self) -> Vec<(&str, String)> {
        let mut headers = Vec::new();
        if !self.api_key.trim().is_empty() {
            headers.push(("Authorization", format!("Bearer {}", self.api_key)));
        }
        if self.multi_agent_beta && !self.has_multi_agent_beta_header() {
            headers.push(("OpenAI-Beta", "responses_multi_agent=v1".to_owned()));
        }
        for (name, value) in &self.extra_headers {
            headers.push((name.as_str(), value.clone()));
        }
        headers
    }

    fn response_url(
        &self,
        response_id: &str,
        include: &[String],
        stream: bool,
        starting_after: Option<u64>,
    ) -> Result<Url, ResponsesClientError> {
        let mut url = self.endpoint(&format!("responses/{response_id}"))?;
        {
            let mut query = url.query_pairs_mut();
            if stream {
                query.append_pair("stream", "true");
            }
            if let Some(sequence) = starting_after {
                query.append_pair("starting_after", &sequence.to_string());
            }
            for value in include {
                query.append_pair("include[]", value);
            }
        }
        Ok(url)
    }

    fn endpoint(&self, path: &str) -> Result<Url, ResponsesClientError> {
        let original_query = self.base_url.query().map(str::to_owned);
        let mut url = self
            .base_url
            .join(path)
            .map_err(ResponsesClientError::InvalidBaseUrl)?;
        if url.query().is_none() {
            url.set_query(original_query.as_deref());
        }
        Ok(url)
    }

    fn authorize<'a>(&self, mut builder: RequestBuilder<'a>) -> RequestBuilder<'a> {
        if !self.api_key.trim().is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }
        if self.multi_agent_beta && !self.has_multi_agent_beta_header() {
            builder = builder.header("OpenAI-Beta", "responses_multi_agent=v1");
        }
        for (name, value) in &self.extra_headers {
            builder = builder.header(name, value);
        }
        builder
    }

    fn has_multi_agent_beta_header(&self) -> bool {
        self.extra_headers.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("OpenAI-Beta")
                && value
                    .split(',')
                    .map(str::trim)
                    .any(|feature| feature == "responses_multi_agent=v1")
        })
    }

    async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T, ResponsesClientError> {
        let response = self.authorize(self.http.get(url)).send().await;
        parse_response(response).await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ResponsesClientError> {
        let url = self.endpoint(path)?;
        let response = self.authorize(self.http.post(url)).json(body).send().await;
        parse_response(response).await
    }
}

async fn parse_response<T: DeserializeOwned>(
    response: Result<http_client::Response, reqwest::Error>,
) -> Result<T, ResponsesClientError> {
    let response = response.map_err(ResponsesClientError::Transport)?;
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .map_err(ResponsesClientError::Transport)?;
        return Err(http_error(status, &body));
    }
    response
        .json()
        .await
        .map_err(ResponsesClientError::Transport)
}

fn http_error(status: http::StatusCode, body: &str) -> ResponsesClientError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let error = parsed
        .as_ref()
        .and_then(|value| value.get("error"))
        .cloned()
        .and_then(|value| serde_json::from_value::<ResponseApiError>(value).ok());
    ResponsesClientError::Http {
        status,
        code: error.as_ref().and_then(|error| error.code.clone()),
        message: error
            .map(|error| error.message)
            .unwrap_or_else(|| status.canonical_reason().unwrap_or("HTTP error").to_owned()),
    }
}

fn parse_event_stream(mut source: http_client::EventSourceStream) -> ResponseEventStream {
    Box::pin(async_stream::stream! {
        let mut terminal = false;
        let mut last_sequence = None;
        while let Some(event) = source.next().await {
            match event {
                Ok(Event::Open) => {}
                Ok(Event::Message(message)) => {
                    if message.data == "[DONE]" {
                        yield Err(ResponsesClientError::StreamEnded { last_sequence });
                        return;
                    }
                    let raw = match serde_json::from_str::<Value>(&message.data) {
                        Ok(raw) => raw,
                        Err(error) => {
                            yield Err(ResponsesClientError::Decode(error));
                            return;
                        }
                    };
                    let parsed = match ResponseStreamEvent::from_value(raw) {
                        Ok(parsed) => parsed,
                        Err(error) => {
                            yield Err(ResponsesClientError::Protocol(error));
                            return;
                        }
                    };
                    if let Some(sequence) = parsed.sequence_number {
                        last_sequence = Some(sequence);
                    }
                    terminal = parsed.kind.is_terminal();
                    yield Ok(parsed);
                    if terminal {
                        return;
                    }
                }
                Err(reqwest_eventsource::Error::InvalidStatusCode(status, response)) => {
                    let body = response.text().await.unwrap_or_default();
                    yield Err(http_error(status, &body));
                    return;
                }
                Err(reqwest_eventsource::Error::Transport(error)) => {
                    yield Err(ResponsesClientError::Transport(error));
                    return;
                }
                Err(error) => {
                    yield Err(ResponsesClientError::Stream(error.to_string()));
                    return;
                }
            }
        }
        if !terminal {
            yield Err(ResponsesClientError::StreamEnded { last_sequence });
        }
    })
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
