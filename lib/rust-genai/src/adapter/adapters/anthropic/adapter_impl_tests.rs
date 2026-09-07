use super::*;
use crate::chat::{ChatOptions, Tool};
use crate::resolver::AuthData;

#[test]
fn short_output_cap_preserves_interleaved_tool_thinking() {
	for with_tools in [false, true] {
		let chat_options = ChatOptions::default()
			.with_reasoning_effort(ReasoningEffort::High)
			.with_max_tokens(8_000)
			.with_extra_headers((
				"anthropic-beta".to_owned(),
				"interleaved-thinking-2025-05-14".to_owned(),
			));
		let mut request = ChatRequest::from_user("检查当前目录");
		if with_tools {
			request = request.with_tools([Tool::new("list_directory").with_schema(json!({
				"type": "object",
				"properties": {},
			}))]);
		}
		let target = ServiceTarget {
			endpoint: AnthropicAdapter::default_endpoint(),
			auth: AuthData::from_single("test-key"),
			model: ModelIden::new(AdapterKind::Anthropic, "claude-sonnet-4-5"),
		};
		let options_set = ChatOptionsSet::default().with_chat_options(Some(&chat_options));
		let web_request = AnthropicAdapter::to_web_request_data(target, ServiceType::ChatStream, request, options_set)
			.expect("有效的交错思考请求应可序列化");
		assert_eq!(web_request.payload["max_tokens"], 8_000);
		assert_eq!(
			web_request.payload["thinking"]["budget_tokens"],
			if with_tools { 24_000 } else { 4_000 },
		);
		assert!(
			web_request
				.headers
				.iter()
				.any(|(name, value)| { name == "anthropic-beta" && value == "interleaved-thinking-2025-05-14" })
		);
	}
}
