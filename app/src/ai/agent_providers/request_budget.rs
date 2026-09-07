//! 只裁剪发给提供商的工具结果副本；终端、持久化历史和工具调用配对保持完整。
//!
//! 字节预算不是模型 tokenizer。按每 token 两个序列化 UTF-8 字节估算文本，
//! 再留出输出和协议余量；未知模型也必须有工具结果的硬上限。

use genai::chat::{ChatRequest, ContentPart};
use serde::Serialize;
use serde_json::{Value, json};

const MAX_RESULT_BYTES: usize = 32 * 1024;
const MAX_TOTAL_RESULT_BYTES: usize = 128 * 1024;
const MIN_RESULT_BYTES: usize = 512;
const UNKNOWN_CONTEXT_WINDOW: u32 = 200_000;
const TRUNCATION_KEY: &str = "_infinishell_truncated";
// 这是发给模型的协议提示，不是界面文案。不得建议重新执行可能有副作用的命令。
const TRUNCATION_NOTE: &str = "Partial tool output: the middle was omitted to fit the context budget. Full output remains in the local conversation. Use bounded reads or searches for missing details; do not repeat state-changing commands.";

#[derive(Debug, thiserror::Error)]
#[error(
    "context_window_exceeded: the local request budget cannot fit the conversation; compact it or reduce attached content"
)]
pub(super) struct RequestBudgetExceeded;

/// 有裁剪时，Responses 必须完整回放裁剪后的本地历史，不能续接仍含原文的服务端状态。
#[derive(Debug, Default)]
pub(super) struct BudgetReport {
    pub truncated_results: usize,
    pub original_result_bytes: usize,
    pub sent_result_bytes: usize,
}

pub(crate) fn input_token_budget(context_window: Option<u32>) -> usize {
    let context = context_window
        .filter(|limit| *limit > 0)
        .unwrap_or(UNKNOWN_CONTEXT_WINDOW) as usize;
    // 小窗口不能被固定的输出预留量吃光。
    context.saturating_sub((context / 4).max(8_000).min(context / 2))
}

/// 与出站硬预算使用同一口径；仅用于压缩决策，不冒充提供商的计费用量。
pub(super) fn estimated_input_tokens(request: &ChatRequest) -> usize {
    let mut bytes = serialized_size(request);
    for message in &request.messages {
        for part in &message.content {
            if matches!(part, ContentPart::Binary(_)) {
                bytes = bytes
                    .saturating_sub(serialized_size(part))
                    .saturating_add(16_384);
            }
        }
    }
    bytes.div_ceil(2)
}

fn serialized_size(value: &impl Serialize) -> usize {
    // 只计数，不为数 MB 的输出额外分配一份序列化缓冲区。
    #[derive(Default)]
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter::default();
    if serde_json::to_writer(&mut counter, value).is_err() {
        return usize::MAX;
    }
    counter.0
}

/// 保留 JSON 的控制字段，只替换过大的 output 字段。其他结构返回明确标注的
/// JSON 预览封套，避免将截断到一半的 JSON 冒充完整工具结果。
fn bounded_result(content: &str, budget: usize) -> String {
    if serialized_size(&content) <= budget {
        return content.to_owned();
    }
    let mut object = serde_json::from_str::<Value>(content).ok();
    let output = object
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|object| object.remove("output"));
    let (mut envelope, source, field) = match (object, output) {
        (Some(Value::Object(object)), Some(Value::String(output))) => {
            (Value::Object(object), output, "output")
        }
        (
            Some(Value::Object(_))
            | Some(Value::Array(_))
            | Some(Value::String(_))
            | Some(Value::Number(_))
            | Some(Value::Bool(_))
            | Some(Value::Null)
            | None,
            _,
        ) => (json!({}), content.to_owned(), "preview"),
    };
    envelope[TRUNCATION_KEY] = json!({"original_bytes": content.len(), "note": TRUNCATION_NOTE});
    envelope[field] = Value::String(String::new());
    if serialized_size(&envelope.to_string()) > budget {
        // 长脚本或其他元数据不能挤掉继续操作命令所需的控制字段。
        // 最小封套仍超预算时，由调用方拒绝发送，而不是悄悄丢失 command_id。
        let mut minimal =
            json!({TRUNCATION_KEY: {"original_bytes": content.len(), "note": TRUNCATION_NOTE}});
        for key in ["status", "exit_code", "command_id", "is_alt_screen_active"] {
            if let Some(value) = envelope.get(key) {
                minimal[key] = value.clone();
            }
        }
        envelope = minimal;
    }
    let field = if envelope.get(field).is_some() {
        field
    } else {
        "preview"
    };
    let mut low = 0;
    let mut high = source.len().min(budget);
    let mut best = envelope.to_string();
    // 对最终出站 JSON 字符串计数，包含引号、反斜杠、控制字符的二次转义。
    while low <= high {
        let keep = low + (high - low) / 2;
        let mut head_end = keep / 2;
        while !source.is_char_boundary(head_end) {
            head_end -= 1;
        }
        let mut tail_start = source.len() - (keep - keep / 2);
        while !source.is_char_boundary(tail_start) {
            tail_start += 1;
        }
        envelope[field] = Value::String(format!(
            "{}\n[... omitted ...]\n{}",
            &source[..head_end],
            &source[tail_start..]
        ));
        let candidate = envelope.to_string();
        if serialized_size(&candidate) <= budget {
            best = candidate;
            low = keep + 1;
        } else if keep == 0 {
            break;
        } else {
            high = keep - 1;
        }
    }
    best
}

pub(super) fn apply(
    request: &mut ChatRequest,
    context_window: Option<u32>,
) -> Result<BudgetReport, RequestBudgetExceeded> {
    let request_budget = input_token_budget(context_window).saturating_mul(2);
    let mut fixed_bytes = serialized_size(request);
    let mut minimum_results = 0usize;
    for message in &request.messages {
        for part in &message.content {
            match part {
                ContentPart::ToolResponse(response) => {
                    let size = serialized_size(&response.content);
                    fixed_bytes = fixed_bytes.saturating_sub(size);
                    minimum_results = minimum_results.saturating_add(size.min(MIN_RESULT_BYTES));
                }
                ContentPart::Binary(_) => {
                    // base64 长度不等于多模态 token 数；保留附件并预留预算，精确计数
                    // 仍由提供商完成，不能按文件字节数误拒绝正常图片。
                    fixed_bytes = fixed_bytes
                        .saturating_sub(serialized_size(part))
                        .saturating_add(16_384);
                }
                ContentPart::Text(_)
                | ContentPart::ToolCall(_)
                | ContentPart::ThoughtSignature(_)
                | ContentPart::ReasoningContent(_)
                | ContentPart::Custom(_) => {}
            }
        }
    }
    let mut remaining = request_budget
        .saturating_sub(fixed_bytes)
        .min(MAX_TOTAL_RESULT_BYTES);
    if fixed_bytes > request_budget || minimum_results > remaining {
        return Err(RequestBudgetExceeded);
    }
    let mut report = BudgetReport::default();
    // 优先保留最近的结果，同时为每个旧结果留下明确提示，绝不删除 tool_call_id 配对。
    for message in request.messages.iter_mut().rev() {
        for part in message.content.iter_mut().rev() {
            if let ContentPart::ToolResponse(response) = part {
                let size = serialized_size(&response.content);
                report.original_result_bytes += response.content.len();
                minimum_results -= size.min(MIN_RESULT_BYTES);
                let budget = remaining
                    .saturating_sub(minimum_results)
                    .min(MAX_RESULT_BYTES);
                if size > budget {
                    response.content = bounded_result(&response.content, budget);
                    report.truncated_results += 1;
                }
                let sent_size = serialized_size(&response.content);
                if sent_size > budget {
                    return Err(RequestBudgetExceeded);
                }
                remaining -= sent_size;
                report.sent_result_bytes += response.content.len();
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
#[path = "request_budget_tests.rs"]
mod tests;
