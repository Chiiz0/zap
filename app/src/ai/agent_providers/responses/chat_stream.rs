use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures::{Stream, StreamExt};
use genai::chat::{
    ChatStreamEvent, CompletionTokensDetails, ContentPart, MessageContent, PromptTokensDetails,
    StopReason, StreamChunk, StreamEnd, ToolCall, ToolChunk, Usage,
};
use serde_json::Value;

use super::{
    ResponseCreateRequest, ResponseEventKind, ResponseStreamEvent, ResponsesClient,
    ResponsesClientError, ResponsesWebSocket, ResponsesWebSocketError,
};

#[cfg(not(target_family = "wasm"))]
pub type NativeChatEventStream = Pin<
    Box<dyn Stream<Item = Result<ChatStreamEvent, NativeResponsesStreamError>> + Send + 'static>,
>;

#[cfg(target_family = "wasm")]
pub type NativeChatEventStream =
    Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, NativeResponsesStreamError>> + 'static>>;

#[derive(Debug, thiserror::Error)]
pub enum NativeResponsesStreamError {
    #[error(transparent)]
    Client(#[from] ResponsesClientError),
    #[error(transparent)]
    WebSocket(#[from] ResponsesWebSocketError),
    #[error("Responses {event}: {message} (code={code:?})")]
    Terminal {
        event: String,
        response_id: Option<String>,
        code: Option<String>,
        message: String,
    },
    #[error("Responses 完成事件缺少 response object")]
    MissingTerminalResponse,
    #[error("Responses 事件流在终止事件前结束")]
    StreamEnded { last_sequence_number: Option<u64> },
}

/// 与正在生成的 Responses 请求共享的取消句柄。
///
/// 普通 HTTP/WS 请求由调用方丢弃流来取消；后台请求还必须在拿到 response id 后调用
/// `/cancel`，否则本地 UI 停止后服务端仍会继续运行和计费。
#[derive(Clone)]
pub struct NativeResponseControl {
    client: ResponsesClient,
    response_id: Arc<Mutex<Option<String>>>,
    background: bool,
}

impl NativeResponseControl {
    pub fn new(client: ResponsesClient, background: bool) -> Self {
        Self {
            client,
            response_id: Arc::new(Mutex::new(None)),
            background,
        }
    }

    fn observe_response_id(&self, response_id: Option<&str>) {
        let Some(response_id) = response_id else {
            return;
        };
        if let Ok(mut current) = self.response_id.lock() {
            *current = Some(response_id.to_owned());
        }
    }

    pub async fn cancel(&self) -> Result<(), ResponsesClientError> {
        if !self.background {
            return Ok(());
        }
        let response_id = self
            .response_id
            .lock()
            .ok()
            .and_then(|response_id| response_id.clone());
        if let Some(response_id) = response_id {
            self.client.cancel(&response_id).await?;
        }
        Ok(())
    }
}

/// 创建能完整保留 Responses item/event 语义的 ChatStreamEvent 流。
/// ChatStreamEvent 只负责复用现有 Agent UI；原始 output items 会放进 StreamEnd。
pub async fn create_native_chat_stream(
    client: ResponsesClient,
    mut request: ResponseCreateRequest,
    mut state_fallback_request: Option<ResponseCreateRequest>,
    use_websocket: bool,
    control: NativeResponseControl,
) -> Result<NativeChatEventStream, NativeResponsesStreamError> {
    let background = request.background == Some(true);
    let events = if use_websocket {
        request.stream = None;
        request.background = None;
        if let Some(fallback) = &mut state_fallback_request {
            fallback.stream = None;
            fallback.background = None;
        }
        websocket_events(client, request, state_fallback_request, control).await?
    } else {
        http_events(client, request, state_fallback_request, background, control)?
    };
    Ok(translate_events(events))
}

fn http_events(
    client: ResponsesClient,
    request: ResponseCreateRequest,
    mut state_fallback_request: Option<ResponseCreateRequest>,
    background: bool,
    control: NativeResponseControl,
) -> Result<RawEventStream, NativeResponsesStreamError> {
    let mut initial = Some(client.create_stream(request)?);
    Ok(Box::pin(async_stream::stream! {
        let mut response_id = None;
        let mut last_sequence = None;
        let mut resume_attempts = 0_u8;
        let mut saw_event = false;
        let mut source = initial.take().expect("初始 Responses stream 只会取一次");
        loop {
            let mut should_resume = false;
            while let Some(event) = source.next().await {
                match event {
                    Ok(event) => {
                        saw_event = true;
                        if let Some(id) = &event.response_id {
                            response_id = Some(id.clone());
                        }
                        control.observe_response_id(event.response_id.as_deref());
                        if let Some(sequence) = event.sequence_number {
                            last_sequence = Some(sequence);
                        }
                        let terminal = event.kind.is_terminal();
                        yield Ok(event);
                        if terminal {
                            return;
                        }
                    }
                    Err(error) => {
                        if !saw_event
                            && previous_response_is_missing(&error)
                            && let Some(fallback_request) = state_fallback_request.take()
                        {
                            source = match client.create_stream(fallback_request) {
                                Ok(source) => source,
                                Err(error) => {
                                    yield Err(NativeResponsesStreamError::Client(error));
                                    return;
                                }
                            };
                            continue;
                        }
                        if background && response_id.is_some() && resume_attempts < 3 {
                            should_resume = true;
                            break;
                        }
                        yield Err(NativeResponsesStreamError::Client(error));
                        return;
                    }
                }
            }
            if background && response_id.is_some() && resume_attempts < 3 {
                should_resume = true;
            }
            if !should_resume {
                return;
            }
            resume_attempts = resume_attempts.saturating_add(1);
            let id = response_id.as_deref().expect("续流前已确认 response id");
            source = match client.resume_stream(
                id,
                last_sequence,
                &["reasoning.encrypted_content".to_owned()],
            ) {
                Ok(source) => source,
                Err(error) => {
                    yield Err(NativeResponsesStreamError::Client(error));
                    return;
                }
            };
        }
    }))
}

async fn websocket_events(
    client: ResponsesClient,
    request: ResponseCreateRequest,
    mut state_fallback_request: Option<ResponseCreateRequest>,
    control: NativeResponseControl,
) -> Result<RawEventStream, NativeResponsesStreamError> {
    let (mut socket, connection_key) = ResponsesWebSocket::connect_reused(&client).await?;
    if socket.create(request.clone()).await.is_err() {
        socket = ResponsesWebSocket::connect(&client).await?;
        socket.create(request).await?;
    }
    Ok(Box::pin(async_stream::stream! {
        let mut saw_event = false;
        loop {
            match socket.next_event().await {
                Ok(event) => {
                    if !saw_event
                        && event_previous_response_is_missing(&event)
                        && let Some(fallback_request) = state_fallback_request.take()
                    {
                        if let Err(error) = socket.create(fallback_request).await {
                            yield Err(NativeResponsesStreamError::WebSocket(error));
                            return;
                        }
                        continue;
                    }
                    saw_event = true;
                    control.observe_response_id(event.response_id.as_deref());
                    let terminal = event.kind.is_terminal();
                    yield Ok(event);
                    if terminal {
                        socket.recycle(connection_key).await;
                        return;
                    }
                }
                Err(error) => {
                    yield Err(NativeResponsesStreamError::WebSocket(error));
                    return;
                }
            }
        }
    }))
}

fn previous_response_is_missing(error: &ResponsesClientError) -> bool {
    matches!(
        error,
        ResponsesClientError::Http { code: Some(code), .. }
            if code == "previous_response_not_found"
    )
}

fn event_previous_response_is_missing(event: &ResponseStreamEvent) -> bool {
    ["/response/error/code", "/error/code", "/code"]
        .into_iter()
        .filter_map(|pointer| event.raw.pointer(pointer).and_then(Value::as_str))
        .any(|code| code == "previous_response_not_found")
}

#[cfg(not(target_family = "wasm"))]
type RawEventStream = Pin<
    Box<
        dyn Stream<Item = Result<ResponseStreamEvent, NativeResponsesStreamError>> + Send + 'static,
    >,
>;

#[cfg(target_family = "wasm")]
type RawEventStream =
    Pin<Box<dyn Stream<Item = Result<ResponseStreamEvent, NativeResponsesStreamError>> + 'static>>;

fn translate_events(mut events: RawEventStream) -> NativeChatEventStream {
    Box::pin(async_stream::stream! {
        let mut started = false;
        let mut calls: BTreeMap<u64, ToolCall> = BTreeMap::new();
        let mut text = String::new();
        let mut reasoning = String::new();
        let mut unknown_events = Vec::new();
        let mut web_citations = Vec::new();
        let mut last_sequence = None;

        while let Some(event) = events.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    yield Err(error);
                    return;
                }
            };
            if let Some(sequence) = event.sequence_number {
                last_sequence = Some(sequence);
            }
            if !started {
                started = true;
                yield Ok(ChatStreamEvent::Start);
            }

            match event.kind {
                ResponseEventKind::Created
                | ResponseEventKind::Queued
                | ResponseEventKind::InProgress
                | ResponseEventKind::ContentPartAdded
                | ResponseEventKind::ContentPartDone
                | ResponseEventKind::OutputTextDone
                | ResponseEventKind::RefusalDone
                | ResponseEventKind::ReasoningSummaryPartAdded
                | ResponseEventKind::ReasoningSummaryPartDone
                | ResponseEventKind::ReasoningSummaryTextDone
                | ResponseEventKind::ReasoningTextDone
                | ResponseEventKind::FileSearchCallInProgress
                | ResponseEventKind::FileSearchCallSearching
                | ResponseEventKind::FileSearchCallCompleted
                | ResponseEventKind::WebSearchCallInProgress
                | ResponseEventKind::WebSearchCallSearching
                | ResponseEventKind::WebSearchCallCompleted
                | ResponseEventKind::ImageGenerationCallInProgress
                | ResponseEventKind::ImageGenerationCallGenerating
                | ResponseEventKind::ImageGenerationCallPartialImage
                | ResponseEventKind::ImageGenerationCallCompleted
                | ResponseEventKind::McpCallArgumentsDelta
                | ResponseEventKind::McpCallArgumentsDone
                | ResponseEventKind::McpCallInProgress
                | ResponseEventKind::McpCallCompleted
                | ResponseEventKind::McpListToolsInProgress
                | ResponseEventKind::McpListToolsCompleted
                | ResponseEventKind::CodeInterpreterCallInProgress
                | ResponseEventKind::CodeInterpreterCallInterpreting
                | ResponseEventKind::CodeInterpreterCallCompleted
                | ResponseEventKind::CodeInterpreterCallCodeDelta
                | ResponseEventKind::CodeInterpreterCallCodeDone
                | ResponseEventKind::CustomToolCallInputDelta
                | ResponseEventKind::CustomToolCallInputDone
                | ResponseEventKind::AudioDone
                | ResponseEventKind::AudioTranscriptDone
                | ResponseEventKind::InjectCreated => {}
                ResponseEventKind::OutputTextAnnotationAdded => {
                    collect_web_citations(&event.raw, &mut web_citations);
                }
                ResponseEventKind::OutputTextDelta
                | ResponseEventKind::RefusalDelta
                | ResponseEventKind::AudioTranscriptDelta => {
                    if let Some(delta) = event.raw.get("delta").and_then(Value::as_str) {
                        text.push_str(delta);
                        yield Ok(ChatStreamEvent::Chunk(StreamChunk {
                            content: delta.to_owned(),
                        }));
                    }
                }
                ResponseEventKind::ReasoningSummaryTextDelta
                | ResponseEventKind::ReasoningTextDelta => {
                    if let Some(delta) = event.raw.get("delta").and_then(Value::as_str) {
                        reasoning.push_str(delta);
                        yield Ok(ChatStreamEvent::ReasoningChunk(StreamChunk {
                            content: delta.to_owned(),
                        }));
                    }
                }
                ResponseEventKind::AudioDelta => {
                    unknown_events.push(event.raw);
                }
                ResponseEventKind::OutputItemAdded | ResponseEventKind::OutputItemDone => {
                    if let Some(item) = event.raw.get("item")
                        && item.get("type").and_then(Value::as_str) == Some("function_call")
                    {
                        let index = event.output_index.unwrap_or_default();
                        let arguments = item
                            .get("arguments")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let call = calls.entry(index).or_insert_with(|| ToolCall {
                            call_id: item
                                .get("call_id")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            fn_name: item
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            fn_arguments: Value::String(String::new()),
                            thought_signatures: None,
                        });
                        if !arguments.is_empty() {
                            call.fn_arguments = parse_arguments(arguments);
                            yield Ok(ChatStreamEvent::ToolCallChunk(ToolChunk {
                                tool_call: call.clone(),
                            }));
                        }
                    }
                }
                ResponseEventKind::FunctionCallArgumentsDelta => {
                    let index = event.output_index.unwrap_or_default();
                    if let Some(call) = calls.get_mut(&index)
                        && let Some(delta) = event.raw.get("delta").and_then(Value::as_str)
                    {
                        let mut arguments = call
                            .fn_arguments
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        arguments.push_str(delta);
                        call.fn_arguments = Value::String(arguments);
                        yield Ok(ChatStreamEvent::ToolCallChunk(ToolChunk {
                            tool_call: call.clone(),
                        }));
                    }
                }
                ResponseEventKind::FunctionCallArgumentsDone => {
                    let index = event.output_index.unwrap_or_default();
                    if let Some(call) = calls.get_mut(&index)
                        && let Some(arguments) = event.raw.get("arguments").and_then(Value::as_str)
                    {
                        call.fn_arguments = parse_arguments(arguments);
                        yield Ok(ChatStreamEvent::ToolCallChunk(ToolChunk {
                            tool_call: call.clone(),
                        }));
                    }
                }
                ResponseEventKind::Completed => {
                    let Some(response) = event.response() else {
                        yield Err(NativeResponsesStreamError::MissingTerminalResponse);
                        return;
                    };
                    let response = match response {
                        Ok(response) => response,
                        Err(error) => {
                            yield Err(NativeResponsesStreamError::Client(
                                ResponsesClientError::Decode(error),
                            ));
                            return;
                        }
                    };
                    let mut content_parts = Vec::new();
                    let completed_text = response_output_text(&response.output);
                    if !completed_text.is_empty() {
                        text = completed_text;
                    }
                    if reasoning.is_empty() {
                        reasoning = response_reasoning_text(&response.output);
                    }
                    for (index, item) in response.output.iter().enumerate() {
                        collect_web_citations(&item.raw, &mut web_citations);
                        if item.raw.get("type").and_then(Value::as_str) == Some("function_call") {
                            calls.insert(index as u64, tool_call_from_item(&item.raw));
                        }
                    }
                    let signatures = response
                        .output
                        .iter()
                        .filter_map(|item| genai::responses::reasoning_item_signature(&item.raw))
                        .collect::<Vec<_>>();
                    content_parts.extend(
                        signatures
                            .iter()
                            .cloned()
                            .map(ContentPart::ThoughtSignature),
                    );
                    if !text.is_empty() {
                        content_parts.push(ContentPart::Text(text.clone()));
                    }
                    for call in calls.values_mut() {
                        if let Some(arguments) = call.fn_arguments.as_str() {
                            call.fn_arguments = parse_arguments(arguments);
                        }
                        content_parts.push(ContentPart::ToolCall(call.clone()));
                    }
                    yield Ok(ChatStreamEvent::End(StreamEnd {
                        captured_usage: response.usage.as_ref().map(response_usage),
                        captured_stop_reason: Some(StopReason::from(response.status.clone())),
                        captured_content: (!content_parts.is_empty())
                            .then(|| MessageContent::from_parts(content_parts)),
                        captured_reasoning_content: (!reasoning.is_empty()).then_some(reasoning),
                        captured_response_id: Some(response.id),
                        captured_response_items: response
                            .output
                            .into_iter()
                            .map(|item| item.raw)
                            .collect(),
                        captured_last_sequence_number: last_sequence,
                        captured_unknown_events: unknown_events,
                        captured_web_citations: web_citations,
                    }));
                    return;
                }
                ResponseEventKind::Failed
                | ResponseEventKind::Incomplete
                | ResponseEventKind::Cancelled
                | ResponseEventKind::Error => {
                    let error = terminal_error(&event);
                    yield Err(error);
                    return;
                }
                ResponseEventKind::InjectFailed
                | ResponseEventKind::McpCallFailed
                | ResponseEventKind::McpListToolsFailed => unknown_events.push(event.raw),
                ResponseEventKind::Unknown(_) => unknown_events.push(event.raw),
            }
        }
        yield Err(NativeResponsesStreamError::StreamEnded {
            last_sequence_number: last_sequence,
        });
    })
}

fn parse_arguments(arguments: &str) -> Value {
    serde_json::from_str(arguments).unwrap_or_else(|_| Value::String(arguments.to_owned()))
}

fn tool_call_from_item(item: &Value) -> ToolCall {
    ToolCall {
        call_id: item
            .get("call_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        fn_name: item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        fn_arguments: item
            .get("arguments")
            .and_then(Value::as_str)
            .map(parse_arguments)
            .unwrap_or(Value::Null),
        thought_signatures: None,
    }
}

fn response_output_text(items: &[super::ResponseItem]) -> String {
    items
        .iter()
        .filter(|item| item.raw.get("type").and_then(Value::as_str) == Some("message"))
        .filter_map(|item| item.raw.get("content").and_then(Value::as_array))
        .flatten()
        .filter_map(|part| {
            let kind = part.get("type").and_then(Value::as_str)?;
            match kind {
                "output_text" => part.get("text").and_then(Value::as_str),
                "refusal" => part.get("refusal").and_then(Value::as_str),
                _ => None,
            }
        })
        .collect()
}

fn response_reasoning_text(items: &[super::ResponseItem]) -> String {
    items
        .iter()
        .filter(|item| item.raw.get("type").and_then(Value::as_str) == Some("reasoning"))
        .filter_map(|item| item.raw.get("summary").and_then(Value::as_array))
        .flatten()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect()
}

fn collect_web_citations(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("url_citation")
                && let Some(url) = object.get("url").and_then(Value::as_str)
                && !output.iter().any(|existing| existing == url)
            {
                output.push(url.to_owned());
            }
            for value in object.values() {
                collect_web_citations(value, output);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_web_citations(value, output);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn response_usage(response: &super::ResponseUsage) -> Usage {
    let cached_tokens = response
        .input_tokens_details
        .as_ref()
        .and_then(|details| details.get("cached_tokens"))
        .and_then(Value::as_u64)
        .and_then(|value| i32::try_from(value).ok());
    let cache_creation_tokens = response
        .input_tokens_details
        .as_ref()
        .and_then(|details| {
            details
                .get("cache_write_tokens")
                .or_else(|| details.get("cache_creation_tokens"))
        })
        .and_then(Value::as_u64)
        .and_then(|value| i32::try_from(value).ok());
    let reasoning_tokens = response
        .output_tokens_details
        .as_ref()
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(Value::as_u64)
        .and_then(|value| i32::try_from(value).ok());
    Usage {
        prompt_tokens: response
            .input_tokens
            .and_then(|value| i32::try_from(value).ok()),
        prompt_tokens_details: cached_tokens
            .map(|cached_tokens| PromptTokensDetails {
                cache_creation_tokens,
                cached_tokens: Some(cached_tokens),
                ..Default::default()
            })
            .or_else(|| {
                cache_creation_tokens.map(|cache_creation_tokens| PromptTokensDetails {
                    cache_creation_tokens: Some(cache_creation_tokens),
                    ..Default::default()
                })
            }),
        completion_tokens: response
            .output_tokens
            .and_then(|value| i32::try_from(value).ok()),
        completion_tokens_details: reasoning_tokens.map(|reasoning_tokens| {
            CompletionTokensDetails {
                reasoning_tokens: Some(reasoning_tokens),
                ..Default::default()
            }
        }),
        total_tokens: response
            .total_tokens
            .and_then(|value| i32::try_from(value).ok()),
    }
}

fn terminal_error(event: &ResponseStreamEvent) -> NativeResponsesStreamError {
    let response = event.response().and_then(Result::ok);
    let response_id = response
        .as_ref()
        .map(|response| response.id.clone())
        .or_else(|| event.response_id.clone());
    let api_error = response
        .as_ref()
        .and_then(|response| response.error.as_ref());
    let message = api_error
        .map(|error| error.message.clone())
        .or_else(|| {
            response
                .as_ref()
                .and_then(|response| response.incomplete_details.as_ref())
                .and_then(|details| details.get("reason"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .or_else(|| {
            event
                .raw
                .pointer("/error/message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .or_else(|| {
            event
                .raw
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Responses 返回非成功终止事件".to_owned());
    NativeResponsesStreamError::Terminal {
        event: event
            .raw
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("error")
            .to_owned(),
        response_id,
        // SSE/WS 的独立 error 事件没有 response object，但错误码仍用于决定是否重试。
        code: api_error.and_then(|error| error.code.clone()).or_else(|| {
            ["/error/code", "/code"]
                .into_iter()
                .find_map(|pointer| event.raw.pointer(pointer).and_then(Value::as_str))
                .map(str::to_owned)
        }),
        message,
    }
}

#[cfg(test)]
#[path = "chat_stream_tests.rs"]
mod tests;
