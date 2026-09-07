use super::*;
use crate::ai::agent_providers::responses::ResponseUsage;
use crate::ai::api_error::is_context_window_exceeded_message;

#[test]
fn 原生事件流保留文本工具原始条目和序号() {
    futures::executor::block_on(async {
        let values = vec![
            serde_json::json!({
                "type": "response.created",
                "sequence_number": 1,
                "response": {"id": "resp_1", "status": "in_progress"}
            }),
            serde_json::json!({
                "type": "response.output_item.added",
                "sequence_number": 2,
                "output_index": 0,
                "item": {
                    "type": "function_call",
                    "id": "fc_1",
                    "call_id": "call_1",
                    "name": "read_files",
                    "arguments": ""
                }
            }),
            serde_json::json!({
                "type": "response.function_call_arguments.delta",
                "sequence_number": 3,
                "output_index": 0,
                "delta": "{\"path\":\"README.md\"}"
            }),
            serde_json::json!({
                "type": "response.output_text.delta",
                "sequence_number": 4,
                "delta": "完成"
            }),
            serde_json::json!({
                "type": "response.completed",
                "sequence_number": 5,
                "response": {
                    "id": "resp_1",
                    "status": "completed",
                    "output": [{
                        "type": "function_call",
                        "id": "fc_1",
                        "call_id": "call_1",
                        "name": "read_files",
                        "arguments": "{\"path\":\"README.md\"}",
                        "caller_id": "prog_1"
                    }, {
                        "type": "message",
                        "role": "assistant",
                        "content": [{
                            "type": "output_text",
                            "text": "完成",
                            "annotations": [{
                                "type": "url_citation",
                                "url": "https://example.test/source"
                            }]
                        }]
                    }],
                    "usage": {
                        "input_tokens": 10,
                        "output_tokens": 4,
                        "total_tokens": 14
                    }
                }
            }),
        ];
        let raw: RawEventStream =
            Box::pin(futures::stream::iter(values.into_iter().map(|value| {
                Ok(ResponseStreamEvent::from_value(value).expect("事件应合法"))
            })));
        let mut stream = translate_events(raw);
        let mut saw_text = false;
        let mut saw_tool = false;
        let mut end = None;
        while let Some(event) = stream.next().await {
            match event.expect("翻译不应失败") {
                ChatStreamEvent::Chunk(chunk) => saw_text |= chunk.content == "完成",
                ChatStreamEvent::ToolCallChunk(chunk) => {
                    saw_tool |= chunk.tool_call.call_id == "call_1"
                }
                ChatStreamEvent::End(value) => end = Some(value),
                ChatStreamEvent::Start
                | ChatStreamEvent::ReasoningChunk(_)
                | ChatStreamEvent::ThoughtSignatureChunk(_) => {}
            }
        }
        let end = end.expect("应有完成事件");
        assert!(saw_text);
        assert!(saw_tool);
        assert_eq!(end.captured_response_id.as_deref(), Some("resp_1"));
        assert_eq!(end.captured_last_sequence_number, Some(5));
        assert_eq!(end.captured_response_items.len(), 2);
        assert_eq!(
            end.captured_response_items[0]
                .get("caller_id")
                .and_then(Value::as_str),
            Some("prog_1")
        );
        assert_eq!(
            end.captured_web_citations,
            vec!["https://example.test/source"]
        );
    });
}

#[test]
fn 非成功终止事件保留恢复元数据() {
    let event = ResponseStreamEvent::from_value(serde_json::json!({
        "type": "response.incomplete",
        "sequence_number": 9,
        "response": {
            "id": "resp_incomplete",
            "status": "incomplete",
            "incomplete_details": {"reason": "max_output_tokens"}
        }
    }))
    .expect("事件应合法");
    let NativeResponsesStreamError::Terminal {
        response_id,
        message,
        ..
    } = terminal_error(&event)
    else {
        panic!("应翻译成终止错误");
    };
    assert_eq!(response_id.as_deref(), Some("resp_incomplete"));
    assert_eq!(message, "max_output_tokens");
}

#[test]
fn 上下文超限结构化错误码在字符串转换后仍可识别() {
    let event = ResponseStreamEvent::from_value(serde_json::json!({
        "type": "response.failed",
        "response": {
            "id": "resp_failed",
            "status": "failed",
            "error": {
                "code": "context_length_exceeded",
                "message": "Request too large"
            }
        }
    }))
    .expect("事件应合法");

    assert!(is_context_window_exceeded_message(
        &terminal_error(&event).to_string()
    ));
}

#[test]
fn 独立错误事件保留嵌套错误码和消息() {
    let event = ResponseStreamEvent::from_value(serde_json::json!({
        "type": "error",
        "error": {
            "code": "context_window_exceeded",
            "message": "Request too large"
        }
    }))
    .expect("事件应合法");
    let error = terminal_error(&event);

    assert!(is_context_window_exceeded_message(&error.to_string()));
    let NativeResponsesStreamError::Terminal { code, message, .. } = error else {
        panic!("应翻译成终止错误");
    };
    assert_eq!(code.as_deref(), Some("context_window_exceeded"));
    assert_eq!(message, "Request too large");
}

#[test]
fn 输出预算耗尽不能误判为输入上下文超限() {
    let event = ResponseStreamEvent::from_value(serde_json::json!({
        "type": "response.incomplete",
        "response": {
            "id": "resp_incomplete",
            "status": "incomplete",
            "incomplete_details": {"reason": "max_output_tokens"}
        }
    }))
    .expect("事件应合法");

    assert!(!is_context_window_exceeded_message(
        &terminal_error(&event).to_string()
    ));
}

#[test]
fn 仅识别明确的previous_response缺失错误() {
    let missing = ResponsesClientError::Http {
        status: http::StatusCode::BAD_REQUEST,
        code: Some("previous_response_not_found".to_owned()),
        message: "missing".to_owned(),
    };
    let other = ResponsesClientError::Http {
        status: http::StatusCode::BAD_REQUEST,
        code: Some("invalid_request".to_owned()),
        message: "invalid".to_owned(),
    };

    assert!(previous_response_is_missing(&missing));
    assert!(!previous_response_is_missing(&other));
    assert!(event_previous_response_is_missing(
        &ResponseStreamEvent::from_value(serde_json::json!({
            "type": "error",
            "error": {"code": "previous_response_not_found", "message": "missing"}
        }))
        .expect("事件应合法")
    ));
}

#[test]
fn gpt56缓存写入token映射到统一usage() {
    let usage = response_usage(&ResponseUsage {
        input_tokens: Some(1_500),
        input_tokens_details: Some(serde_json::json!({
            "cached_tokens": 1_000,
            "cache_write_tokens": 400
        })),
        ..Default::default()
    });
    let details = usage.prompt_tokens_details.expect("应有缓存明细");

    assert_eq!(details.cached_tokens, Some(1_000));
    assert_eq!(details.cache_creation_tokens, Some(400));
}

#[test]
fn 完成事件可从最终response重建漏失的正文推理和工具调用() {
    futures::executor::block_on(async {
        let completed = ResponseStreamEvent::from_value(serde_json::json!({
            "type": "response.completed",
            "sequence_number": 7,
            "response": {
                "id": "resp_final",
                "status": "completed",
                "output": [{
                    "type": "reasoning",
                    "id": "reasoning_1",
                    "summary": [{"type": "summary_text", "text": "已检查上下文"}]
                }, {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "最终答案"}]
                }, {
                    "type": "function_call",
                    "call_id": "call_final",
                    "name": "read_files",
                    "arguments": "{\"path\":\"README.md\"}"
                }]
            }
        }))
        .expect("事件应合法");
        let raw: RawEventStream = Box::pin(futures::stream::iter([Ok(completed)]));
        let events = translate_events(raw).collect::<Vec<_>>().await;
        let end = events
            .into_iter()
            .find_map(|event| match event.expect("翻译不应失败") {
                ChatStreamEvent::End(end) => Some(end),
                ChatStreamEvent::Start
                | ChatStreamEvent::Chunk(_)
                | ChatStreamEvent::ReasoningChunk(_)
                | ChatStreamEvent::ToolCallChunk(_)
                | ChatStreamEvent::ThoughtSignatureChunk(_) => None,
            })
            .expect("应有完成事件");

        assert_eq!(end.captured_first_text(), Some("最终答案"));
        assert_eq!(
            end.captured_reasoning_content.as_deref(),
            Some("已检查上下文")
        );
        let calls = end.captured_tool_calls().expect("应有工具调用");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call_id, "call_final");
        assert_eq!(calls[0].fn_arguments["path"], "README.md");
    });
}

#[test]
fn 托管工具失败事件保留追踪但不提前终止response() {
    futures::executor::block_on(async {
        let values = [
            serde_json::json!({
                "type": "response.mcp_call.failed",
                "sequence_number": 1,
                "item_id": "mcp_1",
                "error": {"message": "upstream unavailable"}
            }),
            serde_json::json!({
                "type": "response.completed",
                "sequence_number": 2,
                "response": {"id": "resp_1", "status": "completed", "output": []}
            }),
        ];
        let raw: RawEventStream =
            Box::pin(futures::stream::iter(values.map(|value| {
                Ok(ResponseStreamEvent::from_value(value).expect("事件应合法"))
            })));
        let events = translate_events(raw).collect::<Vec<_>>().await;
        let end = events
            .into_iter()
            .find_map(|event| match event.expect("托管工具失败不应终止响应") {
                ChatStreamEvent::End(end) => Some(end),
                ChatStreamEvent::Start
                | ChatStreamEvent::Chunk(_)
                | ChatStreamEvent::ReasoningChunk(_)
                | ChatStreamEvent::ToolCallChunk(_)
                | ChatStreamEvent::ThoughtSignatureChunk(_) => None,
            })
            .expect("仍应收到完成事件");

        assert_eq!(end.captured_unknown_events.len(), 1);
        assert_eq!(
            end.captured_unknown_events[0]
                .get("type")
                .and_then(Value::as_str),
            Some("response.mcp_call.failed")
        );
    });
}

#[test]
fn 无终止事件的静默断流返回协议错误() {
    futures::executor::block_on(async {
        let created = ResponseStreamEvent::from_value(serde_json::json!({
            "type": "response.created",
            "sequence_number": 4,
            "response": {"id": "resp_1", "status": "in_progress"}
        }))
        .expect("事件应合法");
        let raw: RawEventStream = Box::pin(futures::stream::iter([Ok(created)]));
        let events = translate_events(raw).collect::<Vec<_>>().await;

        assert!(matches!(
            events.last(),
            Some(Err(NativeResponsesStreamError::StreamEnded {
                last_sequence_number: Some(4)
            }))
        ));
    });
}
