//! This module contains state management logic for pending context, where "pending context"
//! is defined as additional context to be attached to the next AI query.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use ai::project_context::model::ProjectContextModel;
use parking_lot::FairMutex;
use warp_core::features::FeatureFlag;
use warp_util::local_or_remote_path::LocalOrRemotePath;
use warpui::{
    AppContext, Entity, EntityId, ModelContext, ModelHandle, SingletonEntity, WeakModelHandle,
};

use super::agent_view::{AgentViewEntryOrigin, EnterAgentViewError};
use super::block::DirectoryContext;
use super::{ConversationSelectionEvent, ConversationSelectionHandle};
use crate::ai::agent::conversation::{
    AIConversation, AIConversationAutoexecuteMode, AIConversationId, ConversationStatus,
};
use crate::ai::agent::todos::AIAgentTodoList;
use crate::ai::agent::{
    AIAgentAttachment, AIAgentContext, AnyFileContent, FileContext, ImageContext,
};
use crate::ai::block_context::BlockContext;
use crate::ai::document::ai_document_model::AIDocumentId;
use crate::ai::llms::{LLMPreferences, LLMPreferencesEvent};
use crate::code_review::github_repo_model::GitHubRepoModel;
use crate::terminal::TerminalModel;
use crate::terminal::event::{BlockCompletedEvent, BlockType};
use crate::terminal::model::block::{BlockId, BlockMetadata};
use crate::terminal::model::session::Sessions;
use crate::terminal::model_events::{ModelEvent, ModelEventDispatcher};
use crate::util::git::{PrInfo, RepositoryInfo};

/// A non-image file picked via the "attach file" button, stored until query submission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingFile {
    pub file_name: String,
    pub file_path: PathBuf,
    pub mime_type: String,
}

/// 单个 text-like PendingFile inline 进 prompt 的硬上限,超出直接 skip(避免拉爆 context)。
/// 与 `attachment_utils::MAX_ATTACHMENT_SIZE_BYTES`(10MB,用于二进制附件)区别:
/// 那个是字节上限,这个是 inline 进 LLM prompt 的 token 友好上限。
const MAX_INLINE_TEXT_FILE_BYTES: usize = 256 * 1024;

/// 单个 binary PendingFile(PDF / 音频 / 其它)送进 BYOP `Binary` ContentPart 的硬上限。
/// 跟二进制附件使用同一个 10MB 上限,避免一次请求 base64 后撑爆 HTTP body。
const MAX_INLINE_BINARY_FILE_BYTES: usize = 10 * 1024 * 1024;

/// 判断 PendingFile 是否"看起来是文本",决定 P0 是否 inline。
/// 走 mime + 扩展名双保险:`mime_guess` 对 Dockerfile/Makefile 这类无扩展名文件
/// 会返回 `application/octet-stream`,需要补扩展名/文件名匹配。
fn is_text_like(file: &PendingFile) -> bool {
    let mime = file.mime_type.as_str();
    if mime.starts_with("text/") {
        return true;
    }
    // 常见文本类 application/* mime
    matches!(
        mime,
        "application/json"
            | "application/xml"
            | "application/yaml"
            | "application/x-yaml"
            | "application/toml"
            | "application/javascript"
            | "application/typescript"
            | "application/x-sh"
            | "application/x-shellscript"
            | "application/sql"
            | "application/x-httpd-php"
            | "application/x-python"
            | "application/x-ruby"
            | "application/graphql"
    ) || is_text_like_by_filename(&file.file_name)
}

/// 文件名 / 扩展名兜底,覆盖无扩展名约定文件(Dockerfile / Makefile / .env 等)。
fn is_text_like_by_filename(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    // 无扩展名的约定文件
    if matches!(
        lower.as_str(),
        "dockerfile"
            | "makefile"
            | "rakefile"
            | "gemfile"
            | "procfile"
            | "vagrantfile"
            | "license"
            | "readme"
            | "changelog"
            | "authors"
            | "contributors"
            | "notice"
    ) {
        return true;
    }
    // 扩展名兜底
    let ext = match lower.rsplit_once('.') {
        Some((_, ext)) => ext,
        None => return false,
    };
    matches!(
        ext,
        "md" | "markdown"
            | "rst"
            | "txt"
            | "log"
            | "csv"
            | "tsv"
            | "ini"
            | "cfg"
            | "conf"
            | "config"
            | "env"
            | "properties"
            | "lock"
            | "gitignore"
            | "gitattributes"
            | "dockerignore"
            | "editorconfig"
            | "py"
            | "rb"
            | "rs"
            | "go"
            | "java"
            | "kt"
            | "kts"
            | "scala"
            | "swift"
            | "c"
            | "h"
            | "cc"
            | "cpp"
            | "cxx"
            | "hpp"
            | "hxx"
            | "cs"
            | "js"
            | "mjs"
            | "cjs"
            | "jsx"
            | "ts"
            | "tsx"
            | "vue"
            | "svelte"
            | "html"
            | "htm"
            | "xml"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "json"
            | "json5"
            | "jsonc"
            | "yaml"
            | "yml"
            | "toml"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "bat"
            | "cmd"
            | "sql"
            | "graphql"
            | "gql"
            | "proto"
            | "diff"
            | "patch"
    )
}

/// 读 PendingFile 的内容,转成 BYOP / warp-own 双路都能消费的 `FileContext`。
///
/// 三档路径:
/// 1. **text-like 命中 + UTF-8 ok + 不超 text cap** → `StringContent`,内联进 `<file>` XML
/// 2. **多模态 mime(image/pdf/audio)+ 不超 binary cap** → `BinaryContent(bytes)`,
///    BYOP 升级成 `ContentPart::Binary` 真正发给模型
/// 3. **其它 binary(.exe / .zip / 超大文件)** → `BinaryContent(空 Vec)` —— 不读 bytes
///    避免内存浪费,但仍创建 FileContext,让 AI 至少能在 prefix XML 里看到
///    path / mime / size,可调 read_files 等工具自己进一步处理
///
/// 关键修复:`file_name` 字段塞**完整绝对路径**而不是 basename。`FileContext.file_name`
/// 在 `convert.rs:750` 里已经被当 `file_path` 用,user_context 也按 `path` 渲染,
/// 这里塞完整路径让 AI 能用 read_files / shell 工具直接定位文件。
///
/// 设计权衡:warp-own 协议路径上 `BinaryContent` 在 `convert.rs:759` 里被 `Vec<api::FileContent>::from`
/// 直接丢弃(返回空 vec),所以即便我们在这里把所有 binary 都塞进 context 也不会
/// 污染 warp-own 数据流;只有 BYOP 的 `user_context::render_user_attachments` 会
/// 真正消费 BinaryContent 并升级成 `ContentPart::Binary`。
fn read_pending_file_for_context(file: &PendingFile) -> Option<FileContext> {
    let full_path = file.file_path.to_string_lossy().into_owned();
    let metadata_size = std::fs::metadata(&file.file_path).ok().map(|m| m.len());

    // 1) text-like 试 UTF-8
    if is_text_like(file) {
        if let Some(size) = metadata_size {
            if size as usize <= MAX_INLINE_TEXT_FILE_BYTES {
                match std::fs::read(&file.file_path) {
                    Ok(bytes) => {
                        if let Ok(content) = std::str::from_utf8(&bytes) {
                            return Some(FileContext::new(
                                full_path,
                                AnyFileContent::StringContent(content.to_owned()),
                                None,
                                None,
                            ));
                        }
                        // text-like 但内容不是 UTF-8 → 落到 binary 路径
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to read attached file {} for inline context: {e}",
                            file.file_path.display()
                        );
                        return None;
                    }
                }
            }
        }
    }

    // 2) 多模态 binary(image/pdf/audio):需要把 bytes 真送给模型,读取并落 BinaryContent
    let mime = file.mime_type.to_ascii_lowercase();
    let is_multimodal_mime =
        mime.starts_with("image/") || mime == "application/pdf" || mime.starts_with("audio/");
    if is_multimodal_mime {
        if let Some(size) = metadata_size {
            if size as usize <= MAX_INLINE_BINARY_FILE_BYTES {
                match std::fs::read(&file.file_path) {
                    Ok(bytes) => {
                        return Some(FileContext::new(
                            full_path,
                            AnyFileContent::BinaryContent(bytes),
                            None,
                            None,
                        ));
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to read attached file {} for inline context: {e}",
                            file.file_path.display()
                        );
                        return None;
                    }
                }
            } else {
                log::warn!(
                    "Attached file {} ({} bytes) exceeds {} byte multimodal cap; \
                     sending placeholder only (path/mime/size) — AI can use read_files instead",
                    file.file_path.display(),
                    size,
                    MAX_INLINE_BINARY_FILE_BYTES
                );
                // 超大多模态文件:落空 BinaryContent,placeholder 仍带 size(从 metadata 来)
                return Some(FileContext::new(
                    full_path,
                    AnyFileContent::BinaryContent(Vec::new()),
                    None,
                    None,
                ));
            }
        }
    }

    // 3) 其它 binary(.exe / .zip / 未知类型 / metadata 读不到):空 BinaryContent
    // 不读 bytes,避免 100MB exe 占用内存;AI 通过 prefix XML 拿到 path/mime/size
    // 即可决定是否调 read_files 或 shell 工具进一步处理。
    Some(FileContext::new(
        full_path,
        AnyFileContent::BinaryContent(Vec::new()),
        None,
        None,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentType {
    Image,
    File,
}

/// Lightweight metadata for rendering a pending attachment without cloning its payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingAttachmentSummary {
    pub index: usize,
    pub attachment_type: AttachmentType,
    pub file_name: String,
}

/// A pending attachment — either an image (base64 in memory) or a file (path reference).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingAttachment {
    Image(ImageContext),
    File(PendingFile),
}

/// 自动摘要期间用于确认草稿上下文是否仍是原输入，防止续发时清除后来添加的附件。
#[derive(Clone, PartialEq, Eq)]
pub struct PendingContextSnapshot {
    block_ids: HashSet<BlockId>,
    selected_text: Option<String>,
    attachments: Vec<PendingAttachment>,
    diff_hunks: HashMap<String, AIAgentAttachment>,
    at_context: HashMap<String, AIAgentAttachment>,
    document_id: Option<AIDocumentId>,
    auto_attached_blocks: Vec<BlockId>,
}

impl PendingAttachment {
    pub fn file_name(&self) -> &str {
        match self {
            PendingAttachment::Image(img) => &img.file_name,
            PendingAttachment::File(file) => &file.file_name,
        }
    }

    pub fn attachment_type(&self) -> AttachmentType {
        match self {
            PendingAttachment::Image(_) => AttachmentType::Image,
            PendingAttachment::File(_) => AttachmentType::File,
        }
    }
}
/// Model responsible for keeping track of session context to be attached to the next AI query.
pub struct BlocklistAIContextModel {
    terminal_model: Arc<FairMutex<TerminalModel>>,
    directory_context: DirectoryContext,
    github_repo_model: Option<WeakModelHandle<GitHubRepoModel>>,

    /// `BlockId`s corresponding to blocks to be included as context with the next AI query.
    pending_context_block_ids: HashSet<BlockId>,

    /// Selected text to be included as context with the next AI query.
    pending_context_selected_text: Option<String>,

    /// Images and files to be included as attachments with the next AI query.
    pending_attachments: Vec<PendingAttachment>,

    /// Storage for diff hunk attachments that can be referenced in queries
    pending_inline_diff_hunk_attachments: HashMap<String, AIAgentAttachment>,

    /// 输入框中以可见 @名称 展示的上下文附件。
    pending_inline_at_context_attachments: HashMap<String, AIAgentAttachment>,

    conversation_selection: ConversationSelectionHandle,

    /// The ID of the terminal surface this model is associated with.
    terminal_surface_id: EntityId,

    /// AI document ID to be included as context with the next AI query.
    /// When set, the document content will be attached as plain text context.
    pending_document_id: Option<AIDocumentId>,

    /// Block IDs of user-executed commands to be auto-attached as context.
    /// When `AgentViewBlockContext` is enabled, completed user commands are tracked here
    /// and automatically included as context with the next user query.
    auto_attached_agent_view_user_block_ids: Vec<BlockId>,
}

pub fn block_context_from_terminal_model(
    terminal_model: &TerminalModel,
    block_id: &BlockId,
    is_auto_attached: bool,
) -> Option<BlockContext> {
    let block = terminal_model
        .block_list()
        .block_index_for_id(block_id)
        .and_then(|block_id| terminal_model.block_list().block_at(block_id))?;

    // Note, if the user has explicitly asked Agent Mode to include a block as context, we do NOT
    // _force_ secrets to be obfuscated. It will respect the user's settings for secret redaction.
    let output = block.output_grid().content_summary(5000, 5000, false);

    Some(BlockContext {
        id: block_id.clone(),
        index: block.index(),
        command: block.command_to_string(),
        output,
        exit_code: block.exit_code(),
        is_auto_attached,
        started_ts: block.start_ts().cloned(),
        finished_ts: block.completed_ts().cloned(),
        pwd: None,
        shell: None,
        username: None,
        hostname: None,
        git_branch: None,
        os: None,
        session_id: None,
    })
}

impl BlocklistAIContextModel {
    /// Creates pending context state for a terminal surface.
    pub fn new(
        sessions: ModelHandle<Sessions>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        terminal_surface_id: EntityId,
        conversation_selection: ConversationSelectionHandle,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(
            model_event_dispatcher,
            move |me, _, event, ctx| match event {
                ModelEvent::BlockCompleted(BlockCompletedEvent {
                    block_type: BlockType::User(user_block_completed),
                    block_id,
                    ..
                }) => {
                    // If AgentViewBlockContext is enabled and we're in agent view, track user-executed
                    // blocks for auto-attachment as context.
                    if FeatureFlag::AgentViewBlockContext.is_enabled()
                        && me
                            .conversation_selection
                            .as_ref(ctx)
                            .is_conversation_fullscreen(ctx)
                        && !user_block_completed.was_part_of_agent_interaction
                    {
                        me.auto_attached_agent_view_user_block_ids
                            .push(block_id.clone());
                    }

                    // If the block that finished was part of an agent interaction (i.e. LRC finishing),
                    // we should preserve input context.
                    if !FeatureFlag::AgentViewBlockContext.is_enabled()
                        && !user_block_completed.was_part_of_agent_interaction
                    {
                        me.reset_context_to_default(ctx);
                    }
                }
                ModelEvent::BlockMetadataReceived(e) => {
                    me.apply_block_metadata_directory_context(&e.block_metadata, &sessions, ctx);
                }
                ModelEvent::BlockWorkingDirectoryUpdated(e) => {
                    me.apply_block_metadata_directory_context(&e.block_metadata, &sessions, ctx);
                }
                _ => {}
            },
        );

        ctx.subscribe_to_model(&LLMPreferences::handle(ctx), |me, _, event, ctx| {
            if let LLMPreferencesEvent::UpdatedActiveAgentModeLLM = event {
                let llm_prefs = LLMPreferences::as_ref(ctx);
                let vision_supported =
                    llm_prefs.vision_supported(ctx, Some(me.terminal_surface_id));
                if !vision_supported {
                    me.clear_pending_images(ctx);
                }
            }
        });

        ctx.subscribe_to_model(&conversation_selection, |me, _, event, ctx| match event {
            ConversationSelectionEvent::Changed => {
                ctx.emit(BlocklistAIContextEvent::PendingQueryStateUpdated);
            }
            ConversationSelectionEvent::Activated { .. }
            | ConversationSelectionEvent::Deactivated { .. } => {
                me.auto_attached_agent_view_user_block_ids.clear();
            }
        });

        Self {
            terminal_model,
            directory_context: Default::default(),
            github_repo_model: None,
            pending_context_block_ids: HashSet::new(),
            pending_context_selected_text: None,
            pending_attachments: Default::default(),
            conversation_selection,
            terminal_surface_id,
            pending_inline_diff_hunk_attachments: Default::default(),
            pending_inline_at_context_attachments: Default::default(),
            pending_document_id: None,
            auto_attached_agent_view_user_block_ids: Vec::new(),
        }
    }

    /// Test-only constructor that skips production subscriptions and singleton lookups.
    #[cfg(any(test, feature = "test-util"))]
    pub(crate) fn new_for_test(
        terminal_model: Arc<FairMutex<TerminalModel>>,
        terminal_surface_id: EntityId,
        conversation_selection: ConversationSelectionHandle,
    ) -> Self {
        Self {
            terminal_model,
            directory_context: Default::default(),
            github_repo_model: None,
            pending_context_block_ids: HashSet::new(),
            pending_context_selected_text: None,
            pending_attachments: Default::default(),
            conversation_selection,
            terminal_surface_id,
            pending_inline_diff_hunk_attachments: Default::default(),
            pending_inline_at_context_attachments: Default::default(),
            pending_document_id: None,
            auto_attached_agent_view_user_block_ids: Vec::new(),
        }
    }

    /// Resets the set of blocks to be included as context to an empty list.
    /// Also removes any selected text that was to be included as context.
    pub fn reset_context_to_default(&mut self, ctx: &mut ModelContext<Self>) {
        self.set_pending_context_block_ids(vec![], true, ctx);
        self.set_pending_context_selected_text(None, true, ctx);
        self.clear_pending_attachments(ctx);
        self.clear_diff_hunk_attachments();
        self.clear_at_context_attachments();
        self.set_pending_document(None, ctx);
        self.auto_attached_agent_view_user_block_ids.clear();
    }

    pub fn pending_context_snapshot(&self) -> PendingContextSnapshot {
        PendingContextSnapshot {
            block_ids: self.pending_context_block_ids.clone(),
            selected_text: self.pending_context_selected_text.clone(),
            attachments: self.pending_attachments.clone(),
            diff_hunks: self.pending_inline_diff_hunk_attachments.clone(),
            at_context: self.pending_inline_at_context_attachments.clone(),
            document_id: self.pending_document_id,
            auto_attached_blocks: self.auto_attached_agent_view_user_block_ids.clone(),
        }
    }

    /// Returns `true` if the next AI query has any context that should force the input to be
    /// locked in AI mode (skipping NLD): a pending image or file attachment, or a pending
    /// inline `@` context attachment.
    ///
    /// 待附加的 block(`pending_context_block_ids`)刻意不算:上游 #10416 已把它移出锁定条件,
    /// 仅选中一个 block 不应该把输入框锁死在 AI 模式。
    pub fn has_locking_attachment(&self) -> bool {
        !self.pending_attachments.is_empty()
            || !self.pending_inline_at_context_attachments.is_empty()
    }

    /// Returns the set `BlockId`s corresponding to blocks to be included as context with the next
    /// query.
    pub fn pending_context_block_ids(&self) -> &HashSet<BlockId> {
        &self.pending_context_block_ids
    }

    /// Returns selected text to be included as context with the next query.
    pub fn pending_context_selected_text(&self) -> Option<&String> {
        self.pending_context_selected_text.as_ref()
    }

    /// Returns all pending attachments (images and files) for the next query.
    pub fn pending_attachments(&self) -> &[PendingAttachment] {
        &self.pending_attachments
    }

    /// Returns lightweight metadata for all pending attachments.
    pub fn pending_attachment_summaries(&self) -> Vec<PendingAttachmentSummary> {
        self.pending_attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| PendingAttachmentSummary {
                index,
                attachment_type: attachment.attachment_type(),
                file_name: attachment.file_name().to_owned(),
            })
            .collect()
    }

    /// Returns only the pending images for the next query.
    pub fn pending_images(&self) -> Vec<&ImageContext> {
        self.pending_attachments
            .iter()
            .filter_map(|a| match a {
                PendingAttachment::Image(img) => Some(img),
                PendingAttachment::File(_) => None,
            })
            .collect()
    }

    /// Returns only the pending files for the next query.
    pub fn pending_files(&self) -> Vec<&PendingFile> {
        self.pending_attachments
            .iter()
            .filter_map(|a| match a {
                PendingAttachment::File(file) => Some(file),
                PendingAttachment::Image(_) => None,
            })
            .collect()
    }

    /// Given a block ID, transform it into an AIAgentContext::Block.
    pub fn transform_block_to_context(
        &self,
        block_id: &BlockId,
        is_auto_attached_in_agent_view: bool,
    ) -> Option<AIAgentContext> {
        let terminal_model = self.terminal_model.lock();
        block_context_from_terminal_model(&terminal_model, block_id, is_auto_attached_in_agent_view)
            .map(Box::new)
            .map(AIAgentContext::Block)
    }

    /// Returns `AIAgentContext` for the blocks to be included in the current AI query.
    /// If `is_user_query` is true, includes blocks, selected text, and images as context.
    /// If false, excludes these user-specific contexts but includes everything else.
    pub fn pending_context(
        &self,
        app: &AppContext,
        is_user_query: bool,
        current_working_directory_location: Option<&LocalOrRemotePath>,
    ) -> Vec<AIAgentContext> {
        // `pwd` is the shell-reported path used for directory context and local indexing.
        // The location is passed separately because it preserves remote host identity for rules.
        let pwd = self.current_pwd();
        // Zap:原会查 RepoOutlines 判断当前 pwd 下仓库是否已建索引,以便
        // 可选择“使用代码库语义搜索”作为上下文。现 outline 已下线,总是为 false。
        let is_pwd_indexed = false;

        let project_rules = match current_working_directory_location {
            // Zap:本地 pwd 优先走正常路径(零 IO,异步索引完成后从 HashMap 拿结果);
            // 未就绪时同步 fast-path stat + 读 cwd/祖先目录的规则文件。
            // 对齐 opencode `findUp` 模式,保证 cd 后立即发问也能拿到 AGENTS.md 。
            // fast-path 内部有 cache + 时间预算,UI 绝不阻塞。详见
            // `crates/ai/src/project_context/model.rs::find_rules_with_fast_path`。
            Some(LocalOrRemotePath::Local(path)) => {
                let canonical: PathBuf = path.canonicalize().unwrap_or_else(|_| path.clone());
                ProjectContextModel::as_ref(app).find_rules_with_fast_path(&canonical)
            }
            // 远程 pwd 走上游的常规查找,保留 host 身份。
            Some(location) => ProjectContextModel::as_ref(app).find_applicable_rules(location),
            None => None,
        };

        let mut context = Vec::new();

        // Always include directory context
        context.push(AIAgentContext::Directory {
            pwd,
            home_dir: self.home_directory(),
            are_file_symbols_indexed: is_pwd_indexed,
        });

        let (head, branch) = {
            let terminal_model = self.terminal_model.lock();
            let active_block = terminal_model.block_list().active_block();
            (
                active_block.git_branch().cloned(),
                active_block.git_branch_name().cloned(),
            )
        };
        if head.is_some() || branch.is_some() {
            context.push(AIAgentContext::Git {
                head: head.unwrap_or_default(),
                branch,
            });
        }

        // Include repository info from the origin remote URL if available.
        if let Some(repo_context) = self.repository_context(app) {
            context.push(repo_context);
        }
        if let Some(pull_request_context) = self.pull_request_context(app) {
            context.push(pull_request_context);
        }

        // Always include project rules if available
        if let Some(rules) = project_rules {
            context.push(AIAgentContext::ProjectRules {
                root_path: rules.root_path.display_path(),
                active_rules: rules
                    .active_rules
                    .into_iter()
                    .map(|rule| {
                        let line_count = rule.content.lines().count();
                        FileContext {
                            file_name: rule.path.display_path(),
                            content: AnyFileContent::StringContent(rule.content.clone()),
                            line_range: None,
                            last_modified: None,
                            line_count,
                        }
                    })
                    .collect(),
                additional_rule_paths: rules.additional_rule_paths,
            });
        }

        // If this is a user query, add user-selected contexts
        if is_user_query {
            // Add selected blocks (manually attached)
            for block_id in &self.pending_context_block_ids {
                if let Some(block_context) = self.transform_block_to_context(block_id, false) {
                    context.push(block_context);
                }
            }

            // Add auto-attached user-executed blocks (when AgentViewBlockContext is enabled)
            if FeatureFlag::AgentViewBlockContext.is_enabled() {
                for block_id in &self.auto_attached_agent_view_user_block_ids {
                    // Skip if already in pending_context_block_ids to avoid duplicates
                    if !self.pending_context_block_ids.contains(block_id)
                        && let Some(block_context) = self.transform_block_to_context(block_id, true)
                    {
                        context.push(block_context);
                    }
                }
            }

            // Add selected text
            if let Some(selected_text) = &self.pending_context_selected_text {
                context.push(AIAgentContext::SelectedText(selected_text.clone()));
            }

            // Add images from pending attachments
            for attachment in &self.pending_attachments {
                if let PendingAttachment::Image(image) = attachment {
                    context.push(AIAgentContext::Image(image.clone()));
                }
            }

            // Zap P0/P1: 把 PendingFile 同步读入并以 AIAgentContext::File 推进 context。
            // - text-like (UTF-8 解析成功) → StringContent → 走 user_context.rs::render_file
            //   渲染成 <file> XML 块(BYOP)/ api::input_context::File(warp-own)
            // - binary (PDF / 音频 / 其它) → BinaryContent → 走 BYOP user_context Binary
            //   ContentPart 升级路径(warp-own 在 convert.rs:759 直接丢弃,无副作用)
            for attachment in &self.pending_attachments {
                if let PendingAttachment::File(file) = attachment {
                    if let Some(file_context) = read_pending_file_for_context(file) {
                        context.push(AIAgentContext::File(file_context));
                    }
                }
            }
        }

        context
    }

    pub fn current_pwd(&self) -> Option<String> {
        self.directory_context.pwd.clone()
    }

    pub fn home_directory(&self) -> Option<String> {
        self.directory_context.home_dir.clone()
    }

    /// Updates the context model's stored directory context.
    pub fn update_directory_context(
        &mut self,
        pwd: Option<String>,
        home_dir: Option<String>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.directory_context = DirectoryContext { pwd, home_dir };
        ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
            previous_block_ids: self.pending_context_block_ids.clone(),
            requires_block_resync: true,
            requires_text_resync: false,
        });
    }

    fn apply_block_metadata_directory_context(
        &mut self,
        block_metadata: &BlockMetadata,
        sessions: &ModelHandle<Sessions>,
        ctx: &mut ModelContext<Self>,
    ) {
        let pwd = block_metadata
            .current_working_directory()
            .map(|s| PathBuf::from(s.to_owned()));
        if let Some(session_id) = block_metadata.session_id()
            && let Some(active_session) = sessions.as_ref(ctx).get(session_id)
        {
            self.update_directory_context(
                pwd.map(|p| p.to_string_lossy().to_string()),
                active_session.home_dir().map(|sq| sq.to_owned()),
                ctx,
            );
        }
    }

    /// Set `requires_visual_resync` to `false` only if the pending context was modified as a result
    /// of manual user selections. In such cases, a visual resync won't be required because the
    /// pending context was synchronized to the manual selection.
    pub fn set_pending_context_block_ids(
        &mut self,
        ids: impl IntoIterator<Item = BlockId>,
        requires_visual_resync: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        // Filter out blocks that can't be used as AI context
        let filtered_ids: Vec<BlockId> = {
            let terminal_model = self.terminal_model.lock();
            ids.into_iter()
                .filter(|block_id| {
                    terminal_model
                        .block_list()
                        .block_with_id(block_id)
                        .map(|block| {
                            block.can_be_ai_context(terminal_model.block_list().transcript_scope())
                        })
                        .unwrap_or(false)
                })
                .collect()
        };

        let new_pending_context_block_ids = HashSet::from_iter(filtered_ids);

        // Maintain the invariant that we can't simultaneously use both blocks and selected text
        // as context for the next AI request.
        if !new_pending_context_block_ids.is_empty() {
            self.pending_context_selected_text = None;
        }

        if new_pending_context_block_ids != self.pending_context_block_ids {
            let previous_block_ids = self.pending_context_block_ids.clone();
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids,
                requires_block_resync: requires_visual_resync,
                requires_text_resync: !new_pending_context_block_ids.is_empty(),
            });
        }
        self.pending_context_block_ids = new_pending_context_block_ids;
    }

    /// Set `requires_visual_resync` to `false` only if the pending context was modified as a result
    /// of manual user selections. In such cases, a visual resync won't be required because the
    /// pending context was synchronized to the manual selection.
    pub fn set_pending_context_selected_text(
        &mut self,
        text: Option<String>,
        requires_visual_resync: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        // It doesn't make sense to allow empty text as AI context.
        // Enforcing this assertion here ensures we don't run into weird behaviour with `Some("")` later.
        debug_assert!(!matches!(text.as_deref(), Some("")));

        let previous_block_ids = self.pending_context_block_ids.clone();
        // Maintain the invariant that we can't simultaneously use both blocks and selected text
        // as context for the next AI request.
        if text.is_some() {
            self.pending_context_block_ids = HashSet::new();
        }

        if text != self.pending_context_selected_text {
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids,
                requires_block_resync: text.is_some(),
                requires_text_resync: requires_visual_resync,
            });
        }
        self.pending_context_selected_text = text;
    }

    /// Set the pending AI document to be included as context with the next AI query.
    pub fn set_pending_document(
        &mut self,
        document_id: Option<AIDocumentId>,
        ctx: &mut ModelContext<Self>,
    ) {
        if document_id != self.pending_document_id {
            self.pending_document_id = document_id;
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
    }

    /// Get the pending AI document ID if one is set.
    pub fn pending_document_id(&self) -> Option<AIDocumentId> {
        self.pending_document_id
    }

    pub fn clear_pending_images(&mut self, ctx: &mut ModelContext<Self>) {
        let original_attachment_count = self.pending_attachments.len();
        self.pending_attachments
            .retain(|a| !matches!(a, PendingAttachment::Image(_)));
        if self.pending_attachments.len() < original_attachment_count {
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
    }

    pub fn append_pending_images(
        &mut self,
        images: Vec<ImageContext>,
        ctx: &mut ModelContext<Self>,
    ) {
        if !images.is_empty() {
            let attachments: Vec<PendingAttachment> =
                images.into_iter().map(PendingAttachment::Image).collect();
            self.append_pending_attachments(attachments, ctx);
        }
    }

    pub fn remove_pending_image(&mut self, index: usize, ctx: &mut ModelContext<Self>) {
        // Find the nth image in the combined list and remove it.
        let position = self
            .pending_attachments
            .iter()
            .enumerate()
            .filter(|(_, a)| matches!(a, PendingAttachment::Image(_)))
            .nth(index)
            .map(|(i, _)| i);
        if let Some(pos) = position {
            self.remove_pending_attachment(pos, ctx);
        }
    }

    /// Returns the number of images removed
    pub fn remove_last_pending_images(
        &mut self,
        images_to_remove: usize,
        ctx: &mut ModelContext<Self>,
    ) -> usize {
        let image_indices: Vec<usize> = self
            .pending_attachments
            .iter()
            .enumerate()
            .filter(|(_, a)| matches!(a, PendingAttachment::Image(_)))
            .map(|(i, _)| i)
            .collect();
        let len = image_indices.len();

        if images_to_remove == 0 || len == 0 {
            return 0;
        }

        let to_remove = images_to_remove.min(len);
        // Remove from the end to avoid shifting indices.
        for &idx in image_indices.iter().rev().take(to_remove) {
            self.pending_attachments.remove(idx);
        }

        ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
            previous_block_ids: self.pending_context_block_ids.clone(),
            requires_block_resync: false,
            requires_text_resync: false,
        });

        to_remove
    }

    /// Convenience function to set pending query state to continue an existing conversation by ID.
    pub fn set_pending_query_state_for_existing_conversation(
        &mut self,
        conversation_id: AIConversationId,
        origin: AgentViewEntryOrigin,
        ctx: &mut ModelContext<Self>,
    ) {
        self.conversation_selection.update(ctx, |selection, ctx| {
            selection.select_existing_conversation(conversation_id, origin, ctx);
        });
    }

    /// Sets the pending query state to the defaults for a *new* conversation (i.e. not a
    /// followup).
    pub fn set_pending_query_state_for_new_conversation(
        &mut self,
        origin: AgentViewEntryOrigin,
        ctx: &mut ModelContext<Self>,
    ) {
        self.conversation_selection.update(ctx, |selection, ctx| {
            selection.select_new_conversation(origin, ctx);
        });
    }

    /// Starts and selects a new conversation, entering Agent View when this is a GUI selection.
    pub(crate) fn try_start_new_conversation(
        &mut self,
        origin: AgentViewEntryOrigin,
        ctx: &mut ModelContext<Self>,
    ) -> Result<AIConversationId, EnterAgentViewError> {
        self.conversation_selection.update(ctx, |selection, ctx| {
            selection.try_start_new_conversation(origin, ctx)
        })
    }

    /// Returns `true` if a new conversation may be created.
    pub fn can_start_new_conversation(&self) -> bool {
        let terminal_model = self.terminal_model.lock();
        if FeatureFlag::AgentView.is_enabled() {
            !terminal_model
                .block_list()
                .active_block()
                .is_active_and_long_running()
        } else {
            !terminal_model
                .block_list()
                .active_block()
                .is_agent_in_control()
        }
    }

    /// Returns the conversation ID the pending query is following up for, if any.
    /// None if the pending query should start a new conversation.
    pub fn selected_conversation_id(&self, ctx: &AppContext) -> Option<AIConversationId> {
        self.conversation_selection
            .as_ref(ctx)
            .selected_conversation_id(ctx)
    }

    pub fn selected_conversation<'a>(&self, ctx: &'a AppContext) -> Option<&'a AIConversation> {
        self.conversation_selection
            .as_ref(ctx)
            .selected_conversation(ctx)
    }

    pub fn selected_conversation_todolist<'a>(
        &self,
        ctx: &'a AppContext,
    ) -> Option<&'a AIAgentTodoList> {
        self.selected_conversation(ctx)
            .and_then(|c| c.active_todo_list())
            .and_then(|todo_list| {
                // Don't show todo list if it's empty or finished
                if todo_list.is_empty() || todo_list.is_finished() {
                    None
                } else {
                    Some(todo_list)
                }
            })
    }

    pub fn pending_query_autoexecute_override(
        &self,
        ctx: &AppContext,
    ) -> AIConversationAutoexecuteMode {
        self.conversation_selection
            .as_ref(ctx)
            .pending_query_autoexecute_override(ctx)
    }

    pub fn toggle_pending_query_autoexecute(&mut self, ctx: &mut ModelContext<Self>) {
        self.conversation_selection.update(ctx, |selection, ctx| {
            selection.toggle_pending_query_autoexecute(ctx);
        });
    }

    pub fn set_pending_query_autoexecute_override(
        &mut self,
        mode: AIConversationAutoexecuteMode,
        ctx: &mut ModelContext<Self>,
    ) {
        self.conversation_selection.update(ctx, |selection, ctx| {
            selection.set_pending_query_autoexecute_override(mode, ctx);
        });
    }

    /// Returns true if the pending query targets an existing conversation
    /// (as opposed to starting a new one).
    pub fn is_targeting_existing_conversation(&self, ctx: &AppContext) -> bool {
        self.conversation_selection
            .as_ref(ctx)
            .selected_conversation_id(ctx)
            .is_some()
    }

    /// Returns the status of the selected conversation for purposes of rendering the input hint
    /// text, or `None` if there is no selected conversation to display (either because no
    /// conversation is selected, or because the selected conversation is empty/passive/untitled
    /// and should be treated as a "new" conversation). Mirrors the `agent_indicator` pattern in
    /// `app/src/tab.rs`.
    pub fn selected_conversation_status_for_hint(
        &self,
        app: &AppContext,
    ) -> Option<ConversationStatus> {
        let conversation = self.selected_conversation(app)?;
        if conversation.is_empty()
            || conversation.is_entirely_passive()
            || conversation.title().is_none()
        {
            return None;
        }
        Some(conversation.status().clone())
    }

    /// Returns true if there are any blocks that can be used as AI context.
    pub fn can_attach_blocks(&self) -> bool {
        let terminal_model = self.terminal_model.lock();
        terminal_model
            .block_list()
            .blocks()
            .iter()
            .any(|block| block.can_be_ai_context(terminal_model.block_list().transcript_scope()))
    }

    /// Register a diff hunk attachment that can be referenced in future queries
    pub fn register_diff_hunk_attachment(
        &mut self,
        diff_hunk_id: String,
        attachment: AIAgentAttachment,
    ) {
        self.pending_inline_diff_hunk_attachments
            .insert(diff_hunk_id, attachment);
    }

    /// Get a diff hunk attachment by its ID
    pub fn get_diff_hunk_attachment(&self, diff_hunk_id: &str) -> Option<&AIAgentAttachment> {
        self.pending_inline_diff_hunk_attachments.get(diff_hunk_id)
    }

    /// Clear all diff hunk attachments (should be called after each request)
    pub fn clear_diff_hunk_attachments(&mut self) {
        self.pending_inline_diff_hunk_attachments.clear();
    }

    /// 登记一个可在后续 query 中按 @名称 引用的上下文附件。
    pub fn register_at_context_attachment(
        &mut self,
        reference: String,
        attachment: AIAgentAttachment,
    ) {
        self.pending_inline_at_context_attachments
            .insert(reference, attachment);
    }

    /// 返回按可见引用字符串索引的 @ 上下文附件。
    pub fn pending_at_context_attachments(&self) -> &HashMap<String, AIAgentAttachment> {
        &self.pending_inline_at_context_attachments
    }

    fn at_context_references_in_query(&self, query: &str) -> HashSet<String> {
        let mut references = self
            .pending_inline_at_context_attachments
            .keys()
            .collect::<Vec<_>>();
        references
            .sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));

        let mut used_ranges = Vec::new();
        let mut matched_references = HashSet::new();

        for reference in references {
            let mut search_start = 0;
            while search_start <= query.len() {
                let Some(index) = query[search_start..].find(reference.as_str()) else {
                    break;
                };
                let start = search_start + index;
                let end = start + reference.len();
                let overlaps_used_range = used_ranges
                    .iter()
                    .any(|range: &std::ops::Range<usize>| start < range.end && end > range.start);

                if !overlaps_used_range {
                    used_ranges.push(start..end);
                    matched_references.insert(reference.clone());
                }

                search_start = end;
            }
        }

        matched_references
    }

    /// 返回当前 query 中仍然存在的 @ 上下文附件。
    pub fn referenced_at_context_attachments(
        &self,
        query: &str,
    ) -> HashMap<String, AIAgentAttachment> {
        self.at_context_references_in_query(query)
            .into_iter()
            .filter_map(|reference| {
                self.pending_inline_at_context_attachments
                    .get(&reference)
                    .cloned()
                    .map(|attachment| (reference, attachment))
            })
            .collect()
    }

    /// 删除输入框中已经不存在的 @ 上下文附件。
    pub fn retain_at_context_attachments_in_query(&mut self, query: &str) {
        let references = self.at_context_references_in_query(query);
        self.pending_inline_at_context_attachments
            .retain(|reference, _attachment| references.contains(reference));
    }

    /// 清空所有 @ 上下文附件。
    pub fn clear_at_context_attachments(&mut self) {
        self.pending_inline_at_context_attachments.clear();
    }

    /// Appends attachments to the pending list.
    pub fn append_pending_attachments(
        &mut self,
        attachments: Vec<PendingAttachment>,
        ctx: &mut ModelContext<Self>,
    ) {
        if !attachments.is_empty() {
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
        self.pending_attachments.extend(attachments);
    }

    /// Removes an attachment by index.
    pub fn remove_pending_attachment(&mut self, index: usize, ctx: &mut ModelContext<Self>) {
        if index < self.pending_attachments.len() {
            self.pending_attachments.remove(index);
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
    }

    pub fn set_github_repo_model(&mut self, handle: Option<WeakModelHandle<GitHubRepoModel>>) {
        self.github_repo_model = handle;
    }

    /// Builds an `AIAgentContext::Repository` from cached git remote metadata, if available.
    fn repository_context(&self, app: &AppContext) -> Option<AIAgentContext> {
        let handle = self.github_repo_model.as_ref()?.upgrade(app)?;
        let repository_info = handle.as_ref(app).repository_info(app)?;
        Some(Self::repository_context_from_repository_info(
            repository_info,
        ))
    }
    fn repository_context_from_repository_info(repository_info: &RepositoryInfo) -> AIAgentContext {
        AIAgentContext::Repository {
            name: repository_info.name.clone(),
            owner: repository_info.owner.clone(),
            host: repository_info.host.clone(),
        }
    }

    fn pull_request_context(&self, app: &AppContext) -> Option<AIAgentContext> {
        let handle = self.github_repo_model.as_ref()?.upgrade(app)?;
        let pr_info = handle.as_ref(app).pr_info(app)?;
        Self::pull_request_context_from_pr_info(pr_info)
    }
    fn pull_request_context_from_pr_info(pr_info: &PrInfo) -> Option<AIAgentContext> {
        Some(AIAgentContext::PullRequest {
            number: i32::try_from(pr_info.number).ok()?,
            state: pr_info.state.clone(),
            draft: pr_info.draft,
            base_branch: pr_info.base_branch.clone(),
            url: pr_info.url.clone(),
        })
    }

    /// Clears all pending attachments.
    pub fn clear_pending_attachments(&mut self, ctx: &mut ModelContext<Self>) {
        if !self.pending_attachments.is_empty() {
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
        self.pending_attachments.clear();
    }

    /// Drains all pending attachments, returning them, and emits the same update event as
    /// [`Self::clear_pending_attachments`] so the input's attachment chips disappear. Used to
    /// move staged attachments onto a queued prompt row at enqueue time.
    pub fn take_pending_attachments(
        &mut self,
        ctx: &mut ModelContext<Self>,
    ) -> Vec<PendingAttachment> {
        if !self.pending_attachments.is_empty() {
            ctx.emit(BlocklistAIContextEvent::UpdatedPendingContext {
                previous_block_ids: self.pending_context_block_ids.clone(),
                requires_block_resync: false,
                requires_text_resync: false,
            });
        }
        std::mem::take(&mut self.pending_attachments)
    }
}

pub enum BlocklistAIContextEvent {
    /// The bool fields determine whether a visual resync is needed for each respective selection type.
    /// For example, if selected text is cleared via the `BlocklistAIContextModel` **only**, then
    /// the `TerminalView`'s current text selection should be visually cleared as well.
    UpdatedPendingContext {
        previous_block_ids: HashSet<BlockId>,
        requires_block_resync: bool,
        requires_text_resync: bool,
    },
    /// Emitted whenever the value changes.
    PendingQueryStateUpdated,
}

impl Entity for BlocklistAIContextModel {
    type Event = BlocklistAIContextEvent;
}

#[cfg(test)]
#[path = "context_model_tests.rs"]
mod tests;
