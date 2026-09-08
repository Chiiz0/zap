use std::collections::HashMap;
use std::sync::Arc;

use chrono::Local;
use lazy_static::lazy_static;
use regex::Regex;
use warp_core::features::FeatureFlag;
use warpui::{AppContext, SingletonEntity};

use super::SessionContext;
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::{
    AIAgentAttachment, AIAgentContext, DocumentContentAttachmentSource, DriveObjectPayload,
};
use crate::ai::block_context::BlockContext;
use crate::ai::blocklist::BlocklistAIContextModel;
use crate::ai::document::ai_document_model::{AIDocumentId, AIDocumentModel};
use crate::ai::facts::AIFactObjectModel;
use crate::ai::skills::list_skills;
use crate::cloud_object::model::generic_string_model::{GenericStringObjectId, StoredStringObject};
use crate::cloud_object::model::persistence::ObjectStoreModel;
use crate::cloud_object::{
    GenericStoredObject, GenericStringObjectFormat, JsonObjectType, ObjectType,
};
use crate::terminal::TerminalView;
use crate::terminal::model::block::BlockId;
use crate::terminal::model::session::active_session::ActiveSession;
lazy_static! {
    // Regex to match <block:[block_id]> patterns
    pub static ref BLOCK_CONTEXT_ATTACHMENT_REGEX: Regex = Regex::new(r"<block:([^>]+)>")
        .expect("Block context attachment regex should be parsed");
    // Regex to match zap drive objects inserted via at-context. Ex: <notebook:[workflow_id]>
    pub static ref DRIVE_OBJECT_ATTACHMENT_REGEX: Regex = Regex::new(r"<(workflow|notebook|plan|rule):([^>]+)>")
        .expect("Drive object attachment regex should be parsed");
    // Regex to match <change:filename:line_start-line_end> patterns
    pub static ref DIFF_HUNK_ATTACHMENT_REGEX: Regex = Regex::new(r"<change:([^>]+)>")
        .expect("Diff hunk attachment regex should be parsed");
}

// Returns the context to be attached to the AIAgentInput sent in a request.
// If `is_user_query` is true, includes selected blocks, text, and images from the context model.
// Always includes base context like current time, execution environment, and codebase info.
pub(super) fn input_context_for_request(
    is_user_query: bool,
    context_model: &BlocklistAIContextModel,
    active_session: &ActiveSession,
    // 保留参数:多个上层调用方的签名暂不动;list_skills 不再需要会话 id 去重,
    // 但其他潜在用法可能仍需要 → 留 _ 前缀避免 unused 警告。
    _conversation_id: Option<AIConversationId>,
    additional_context: Vec<AIAgentContext>,
    app: &AppContext,
) -> Arc<[AIAgentContext]> {
    // 上游把 pwd 的"位置"(保留远端 host 身份)单独传进 `pending_context`,
    // 供 project rules 查找区分本地 / 远程仓库。
    let current_working_directory_location = active_session.current_working_directory_location(app);
    let mut context = context_model.pending_context(
        app,
        is_user_query,
        current_working_directory_location.as_ref(),
    );

    context.push(AIAgentContext::CurrentTime {
        current_time: Local::now(),
    });

    if let Some(env) = active_session.ai_execution_environment(app) {
        context.push(AIAgentContext::ExecutionEnvironment(env));
    }

    if FeatureFlag::ListSkills.is_enabled() {
        // 项目去云端后,system prompt 每轮在客户端完整重渲(BYOP 无状态),
        // skills 必须每轮全量送达,不再做差量。空列表时也不 push,保持
        // context 紧凑(模板侧 `{% if skills %}` 守卫即可正常省略 section)。
        let path_origin = SessionContext::from_session(active_session, app).skill_path_origin();
        let skills = list_skills(
            current_working_directory_location.as_ref(),
            &path_origin,
            app,
        );
        if !skills.is_empty() {
            context.push(AIAgentContext::Skills { skills });
        }
    }

    context.extend(additional_context);

    context.into()
}

/// Parses context reference strings like <block:123> from the user query and returns
/// a map of reference strings to AIAgentAttachment objects.
///
/// This searches across ALL TerminalModels, not just the active session, to find
/// the requested blocks.
pub(super) fn parse_context_attachments(
    query: &str,
    context_model: &BlocklistAIContextModel,
    ctx: &AppContext,
) -> HashMap<String, AIAgentAttachment> {
    let mut referenced_attachments = HashMap::new();

    // Parse block attachments
    for capture in BLOCK_CONTEXT_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(block_id_match)) = (capture.get(0), capture.get(1)) {
            let reference_string = full_match.as_str().to_string();
            let block_id_str = block_id_match.as_str();

            let block_id = BlockId::from(block_id_str.to_string());

            // Search across ALL TerminalModels to find the block
            if let Some(attachment) = find_block_attachment_in_all_terminals(&block_id, ctx) {
                referenced_attachments.insert(reference_string, attachment);
            }
        }
    }

    // Parse drive object attachments (notebooks, workflows, etc)
    for capture in DRIVE_OBJECT_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(object_type_match), Some(object_id_match)) =
            (capture.get(0), capture.get(1), capture.get(2))
        {
            let reference_string = full_match.as_str().to_string();
            let object_type_str = object_type_match.as_str();
            let id_str = object_id_match.as_str();

            if object_type_str == "plan" {
                if let Some(attachment) = plan_attachment_for_reference(id_str, ctx) {
                    referenced_attachments.insert(reference_string, attachment);
                }
            } else {
                let object_type = match object_type_str {
                    "workflow" => ObjectType::Workflow,
                    "notebook" => ObjectType::Notebook,
                    "rule" => ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                        JsonObjectType::AIFact,
                    )),
                    _ => continue, // Skip unknown object types
                };

                let attachment = drive_object_attachment_for_reference(id_str, object_type, ctx);
                referenced_attachments.insert(reference_string, attachment);
            }
        }
    }

    // Parse diff hunk attachments
    for capture in DIFF_HUNK_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(diff_hunk_match)) = (capture.get(0), capture.get(1)) {
            let reference_string = full_match.as_str().to_string();
            let diff_hunk_key = diff_hunk_match.as_str();

            // Check if we have a stored diff hunk attachment for this key
            if let Some(attachment) = context_model.get_diff_hunk_attachment(diff_hunk_key) {
                referenced_attachments.insert(reference_string, attachment.clone());
            }
        }
    }

    referenced_attachments.extend(context_model.referenced_at_context_attachments(query));

    // Add pending file attachments as FilePathReference.
    // Duplicate basenames get a (1), (2), ... suffix to avoid collisions,
    // matching the legacy attachment-key pattern.
    for file in context_model.pending_files().iter() {
        let attachment = AIAgentAttachment::FilePathReference {
            file_id: uuid::Uuid::new_v4().to_string(),
            file_name: file.file_name.clone(),
            file_path: file.file_path.to_string_lossy().to_string(),
        };
        let mut key = file.file_name.clone();
        if referenced_attachments.contains_key(&key) {
            let mut suffix = 1;
            loop {
                key = format!("{} ({suffix})", file.file_name);
                if !referenced_attachments.contains_key(&key) {
                    break;
                }
                suffix += 1;
            }
        }
        referenced_attachments.insert(key, attachment);
    }

    // Add pending AI document as attachment if present
    if let Some(document_id) = context_model.pending_document_id() {
        if let Some(content) = AIDocumentModel::as_ref(ctx).get_document_content(&document_id, ctx)
        {
            let document_id_str = document_id.to_string();
            let attachment = AIAgentAttachment::DocumentContent {
                document_id: document_id_str.clone(),
                content,
                source: DocumentContentAttachmentSource::PlanEdited,
                line_range: None,
            };
            // Use the document ID as the reference key
            referenced_attachments.insert(document_id_str, attachment);
        }
    }

    referenced_attachments
}

/// Searches for a block across all terminal models in the application.
/// Returns an AIAgentAttachment if the block is found.
fn find_block_attachment_in_all_terminals(
    block_id: &BlockId,
    ctx: &AppContext,
) -> Option<AIAgentAttachment> {
    // Iterate over all window IDs to search across all terminal views
    for window_id in ctx.window_ids() {
        // Try to get all terminal views for this window
        if let Some(terminal_views) = ctx.views_of_type::<TerminalView>(window_id) {
            for terminal_view_handle in terminal_views {
                let terminal_view = terminal_view_handle.as_ref(ctx);
                let terminal_model = terminal_view.model.lock();
                let block_list = terminal_model.block_list();

                if let Some(block) = block_list.block_with_id(block_id) {
                    // Create an AIAgentAttachment for the block
                    return Some(AIAgentAttachment::Block(BlockContext {
                        id: block.id().clone(),
                        index: block.index(),
                        command: block.command_to_string(),
                        output: block.output_to_string(),
                        exit_code: block.exit_code(),
                        is_auto_attached: false,
                        started_ts: block.start_ts().cloned(),
                        finished_ts: block.completed_ts().cloned(),
                        pwd: None,
                        shell: None,
                        username: None,
                        hostname: None,
                        git_branch: None,
                        os: None,
                        session_id: None,
                    }));
                }
            }
        }
    }

    None
}

pub(crate) fn drive_object_attachment_for_reference(
    uid: &str,
    object_type: ObjectType,
    ctx: &AppContext,
) -> AIAgentAttachment {
    AIAgentAttachment::DriveObject {
        uid: uid.to_string(),
        payload: get_object_attachment_payload(uid, object_type, ctx),
    }
}

pub(crate) fn plan_attachment_for_reference(
    ai_document_uid: &str,
    ctx: &AppContext,
) -> Option<AIAgentAttachment> {
    let ai_doc_id = match AIDocumentId::try_from(ai_document_uid) {
        Ok(id) => id,
        Err(_) => {
            log::warn!("Invalid ai_document_id in plan reference: {ai_document_uid}");
            return None;
        }
    };

    let content = AIDocumentModel::as_ref(ctx)
        .get_document_content(&ai_doc_id, ctx)
        .or_else(|| {
            ObjectStoreModel::as_ref(ctx)
                .get_all_active_notebooks()
                .find(|nb| nb.model().ai_document_id.as_ref() == Some(&ai_doc_id))
                .map(|nb| nb.model().data.clone())
        });

    if let Some(content) = content {
        return Some(AIAgentAttachment::DocumentContent {
            document_id: ai_document_uid.to_string(),
            content,
            source: DocumentContentAttachmentSource::UserAttached,
            line_range: None,
        });
    }

    log::warn!("Plan not found for ai_document_id: {ai_doc_id}");
    None
}

/// 从 ObjectStoreModel 中按 UID 和类型取对象 payload。
/// 找不到对象时返回 None。
fn get_object_attachment_payload(
    uid: &str,
    object_type: ObjectType,
    ctx: &AppContext,
) -> Option<DriveObjectPayload> {
    match object_type {
        ObjectType::Workflow => {
            ObjectStoreModel::as_ref(ctx)
                .get_workflow_by_uid(uid)
                .map(|workflow| {
                    let workflow_data = &workflow.model().data;
                    DriveObjectPayload::Workflow {
                        name: workflow_data.name().to_string(),
                        description: workflow_data.description().cloned().unwrap_or_default(),
                        command: workflow_data.content().to_string(),
                    }
                })
        }
        ObjectType::Notebook => {
            ObjectStoreModel::as_ref(ctx)
                .get_notebook_by_uid(uid)
                .map(|notebook| DriveObjectPayload::Notebook {
                    title: notebook.model().title.clone(),
                    content: notebook.model().data.clone(),
                })
        }
        ObjectType::GenericStringObject(_) => {
            // For generic string objects, we only support AI facts (rules) for now
            ObjectStoreModel::as_ref(ctx)
                .get_by_uid(&uid.to_string())
                .and_then(|object| {
                    if let Some(ai_fact) = object.as_any().downcast_ref::<GenericStoredObject<GenericStringObjectId, AIFactObjectModel>>() {
                        let string_object = ai_fact as &dyn StoredStringObject;
                        let object_type =
                            generic_string_object_format_name(string_object.generic_string_object_format());
                        Some(DriveObjectPayload::GenericStringObject {
                            payload: string_object.serialized().model_as_str().to_string(),
                            object_type,
                        })
                    } else {
                        None
                    }
                })
        }
        _ => None, // Other object types not supported for drive object attachments
    }
}

fn generic_string_object_format_name(format: GenericStringObjectFormat) -> String {
    match format {
        GenericStringObjectFormat::Json(JsonObjectType::Preference) => "JsonPreference",
        GenericStringObjectFormat::Json(JsonObjectType::EnvVarCollection) => "JsonEnvVarCollection",
        GenericStringObjectFormat::Json(JsonObjectType::WorkflowEnum) => "JsonWorkflowEnum",
        GenericStringObjectFormat::Json(JsonObjectType::AIFact) => "JsonAIFact",
        GenericStringObjectFormat::Json(JsonObjectType::MCPServer) => "JsonMCPServer",
        GenericStringObjectFormat::Json(JsonObjectType::AIExecutionProfile) => {
            "JsonAIExecutionProfile"
        }
        GenericStringObjectFormat::Json(JsonObjectType::TemplatableMCPServer) => {
            "JsonTemplatableMCPServer"
        }
        // Zap:这三个是 `cloud_objects` crate 里上游云端侧的对象类型
        // (云端环境 / 定时 ambient agent / 云端 agent 配置)。我方已剥离全部
        // 云端功能,本地 ObjectStore 不会产生这类对象,而且本函数唯一的调用点
        // 只在 `AIFactObjectModel` downcast 成功后才走到,所以该分支实际不可达。
        // 这里显式列出而不是用 `_ =>` 兜底,是为了上游枚举再扩展时能继续报编译错。
        GenericStringObjectFormat::Json(
            JsonObjectType::CloudEnvironment
            | JsonObjectType::ScheduledAmbientAgent
            | JsonObjectType::CloudAgentConfig,
        ) => "JsonUnsupportedCloudObject",
    }
    .to_string()
}
