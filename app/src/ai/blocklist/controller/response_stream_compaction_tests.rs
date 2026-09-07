use super::*;
use crate::ai::agent::AIAgentInput;
use crate::ai::agent_providers::request_budget::input_token_budget;

#[test]
fn byop_auto_compaction_uses_actual_window_and_stops_after_recovery() {
    let mut stream = ResponseStream::new_for_test(ResponseStreamId::new_local());
    stream.byop_context_window = Some(Some(32_768));
    stream.allow_auto_compaction = true;
    let threshold = input_token_budget(Some(32_768)).saturating_mul(4) / 5;
    assert!(!stream.should_auto_compact(threshold - 1, true));
    assert!(stream.should_auto_compact(threshold, true));
    assert!(!stream.should_auto_compact(threshold, false));
    assert!(!stream.should_auto_compact(0, true));

    stream.allow_auto_compaction = false;
    assert!(!stream.should_auto_compact(usize::MAX, true));
    stream.allow_auto_compaction = true;
    stream
        .params
        .input
        .push(AIAgentInput::SummarizeConversation {
            prompt: None,
            overflow: false,
            context: Default::default(),
        });
    assert!(!stream.should_auto_compact(usize::MAX, true));
}

#[test]
fn byop_auto_compaction_excludes_non_byop_and_handles_unknown_window() {
    let mut stream = ResponseStream::new_for_test(ResponseStreamId::new_local());
    stream.allow_auto_compaction = true;
    assert!(!stream.should_auto_compact(usize::MAX, true));

    stream.byop_context_window = Some(None);
    let threshold = input_token_budget(None).saturating_mul(4) / 5;
    assert!(!stream.should_auto_compact(threshold - 1, true));
    assert!(stream.should_auto_compact(threshold, true));
}
