# InfiniShell — English (source-of-truth locale)
# Add each new key to the matching surface section and add the same key to zh-CN/warp.ftl.
# Keep Fluent variables aligned across locales; app/src/i18n_tests.rs enforces key and variable parity.
#
# 命名规范:kebab-case,前缀按 surface,例 settings-ai-title / drive-folder-rename-title
# 变量插值用 Fluent { $name } 语法,不要拼接

# =============================================================================
# SECTION: common
# =============================================================================

app-name = InfiniShell
app-tagline = The local-first agentic terminal for developers

common-ok = OK
common-cancel = Cancel
common-apply = Apply
common-save = Save
common-delete = Delete
common-confirm = Confirm
common-close = Close
common-reset = Reset
common-back = Back
common-next = Next
common-yes = Yes
common-no = No
common-continue = Continue
common-approve = Approve
common-deny = Deny
common-import = Import
common-upgrade = Upgrade
common-default = Default
common-editing = Editing
common-viewing = Viewing
common-tooltip-enter-edit-mode = Click to start editing
common-tooltip-exit-edit-mode = Click to exit editing
common-restored = Restored
common-continued = Continued
common-deleted = Deleted
common-send-feedback = Send Feedback
common-something-went-wrong = Something went wrong
common-no-results-found = No results found.
common-edit = Edit
common-add = Add
common-remove = Remove
common-rename = Rename
common-copy = Copy
common-paste = Paste
common-search = Search
common-view = View
common-loading = Loading…
common-error = Error
common-warning = Warning
common-info = Info
common-success = Success
common-all = All
common-none = None
common-unknown = Unknown
common-open = Open
common-restore = Restore
common-duplicate = Duplicate
common-export = Export
common-trash = Trash
common-copy-link = Copy link
common-untitled = Untitled
common-retry = Retry
common-maximize = Maximize
common-discard = Discard
common-undo = Undo
common-commit = Commit
common-push = Push
common-publish = Publish
common-create = Create
common-configure = Configure
common-dismiss = Dismiss
common-manage = Manage
common-failed = Failed
common-done = Done
common-working = Working
common-cut = Cut
common-previous = Previous
common-suggested = Suggested
common-copied-to-clipboard = Copied to clipboard
common-new = New
common-no-results = No results
common-learn-more = Learn more
common-user = User
common-show-in-file-explorer = Show in file explorer
common-skip = Skip
common-get-warping = Get started
common-try-again = Try again
common-settings = Settings
common-recommended = Recommended
common-enabled = Enabled
common-disabled = Disabled
common-free = Free
common-list-prefix = {" - "}
common-current-directory = the current directory

# =============================================================================
# SECTION: agent-management
# Files: app/src/ai/agent_management/**
# =============================================================================

agent-management-filter-all-tooltip = View your agent tasks plus all shared team tasks
agent-management-filter-personal = Personal
agent-management-filter-personal-tooltip = View agent tasks you created
agent-management-get-started = Get started
agent-management-view-agents = View Agents
agent-management-clear-filters = Clear filters
agent-management-clear-all = Clear all
agent-management-new-agent = New agent
agent-management-status = Status
agent-management-source = Source
agent-management-created-on = Created on
agent-management-has-artifact = Has artifact
agent-management-harness = Harness
agent-management-environment = Environment
agent-management-created-by = Created by
agent-management-last-24-hours = Last 24 hours
agent-management-past-3-days = Past 3 days
agent-management-last-week = Last week
agent-management-artifact-pull-request = Pull Request
agent-management-artifact-plan = Plan
agent-management-artifact-screenshot = Screenshot
agent-management-artifact-file = File
agent-management-source-scheduled = Scheduled
agent-management-source-local-agent = InfiniShell (local agent)
agent-management-source-cloud-agent = InfiniShell Agent
agent-management-source-oz-web = InfiniShell Agent
agent-management-source-github-action = GitHub Action
agent-management-no-session-available = No session available
agent-management-session-expired = Session expired
agent-management-session-expired-tooltip = Sessions expire after one week and cannot be opened.
agent-management-metadata-source = Source: { $source }
agent-management-metadata-harness = Harness: { $harness }
agent-management-metadata-run-time = Run time: { $run_time }
agent-management-metadata-credits-used = Credits used: { $usage }
agent-management-environment-selected = Environment: { $environment }
agent-management-loading-cloud-runs = Loading agent runs

# =============================================================================
# SECTION: workspace-runtime
# Files: app/src/workspace/view.rs
# =============================================================================

workspace-menu-update-warp-manually = Update InfiniShell manually
workspace-menu-whats-new = What's new
workspace-menu-settings = Settings
workspace-menu-keyboard-shortcuts = Keyboard shortcuts
workspace-application-menu-tooltip = Application menu
workspace-menu-documentation = Documentation
workspace-menu-feedback = Feedback
workspace-menu-view-warp-logs = View InfiniShell logs
workspace-menu-slack = Slack
workspace-toast-failed-load-conversation = Failed to load conversation.
workspace-toast-failed-load-conversation-for-forking = Failed to load conversation for forking.
workspace-toast-conversation-forking-failed = Conversation forking failed.
workspace-toast-no-terminal-pane-for-context = No terminal pane open. Open a new pane to attach as context.
workspace-toast-plan-already-in-context = This plan is already in context.
workspace-toast-command-still-running = A command in this session is still running.
workspace-toast-cannot-open-terminal-session = Cannot open a new terminal session
workspace-toast-out-of-ai-credits = Looks like you're out of AI credits.
workspace-toast-upgrade-more-credits = Upgrade for more credits.
workspace-toast-disabled-synchronized-inputs = Disabled all synchronized inputs.
workspace-toast-conversation-deleted = Conversation deleted
workspace-search-repos-placeholder = Search repos
workspace-search-tabs-placeholder = Search tabs…
terminal-onekey-search-placeholder = Search saved SSH credentials…
terminal-onekey-search-no-results = No matching SSH credentials
workspace-rearrange-toolbar-items = Rearrange toolbar items
workspace-new-session-agent = Agent
workspace-new-session-terminal = Terminal
workspace-new-session-cloud-oz = Agent tab
workspace-new-session-local-docker-sandbox = Local Docker Sandbox
workspace-new-worktree-config = New worktree config
workspace-new-tab-config = New tab config
workspace-reopen-closed-session = Reopen closed session
workspace-session-config-tab-config-chip = Access your tab configs here.
workspace-toast-failed-remove-tab-config = Failed to remove tab config: { $error }
workspace-toast-failed-load-tab-config = Failed to load tab config { $path }: { $error }
workspace-toast-failed-load-model-config = Failed to load model config { $path }: { $error }
workspace-heap-profile-written = Wrote heap profile to { $path }
workspace-heap-profile-write-failed = Failed to write heap profile: { $error }
workspace-local-conversation-unavailable = Conversation is not available in local InfiniShell history.
workspace-untitled-pane = Untitled pane
workspace-switch-team = Switch team
workspace-new-tab-group = New tab group
workspace-open-tab-config-title = Open: { $name }
workspace-new-group = New Group
workspace-rename-pane = Rename pane
workspace-reset-pane-name = Reset pane name
workspace-rename-active-pane = Rename active pane
workspace-reset-active-pane-name = Reset active pane name
workspace-current-version = Current version is { $version }
workspace-install-update = Install update ({ $version })
workspace-cli-agent-installed = Installed the InfiniShell Agent CLI globally. You can now run '{ $command }' from any terminal outside InfiniShell.
workspace-cli-agent-removed = Removed the global InfiniShell Agent CLI installation—it still works inside InfiniShell.
workspace-cli-agent-install-failed = Failed to install the InfiniShell Agent CLI command
workspace-cli-agent-uninstall-failed = Failed to uninstall the InfiniShell Agent CLI command
workspace-auto-handoff-success = Handed session off to the cloud
workspace-toast-forked-conversation = Forked “{ $title }”
workspace-toast-mouse-reporting-enabled = You enabled mouse reporting.
workspace-toast-mouse-reporting-disabled = You disabled mouse reporting.
workspace-toast-sync-all-enabled = You enabled synchronized inputs in all tabs.
workspace-toast-sync-all-disabled = You disabled synchronized inputs in all tabs.
workspace-toast-sync-tab-enabled = You enabled synchronized inputs in this tab.
workspace-toast-sync-tab-disabled = You disabled synchronized inputs in this tab.
workspace-toast-press-to-undo = {" "}Press { $key } to undo.
workspace-process-sample-saved = Process sample saved to { $path }
workspace-process-sample-failed = Failed to sample process (check logs)
ai-conversation-renamed = Conversation renamed to { $title }
ai-recording-open-failed = Failed to open recording.
ai-code-diff-revert-failed = Failed to revert changes to { $file }
ai-requested-command-request = Request
ai-requested-command-response = Response
ai-requested-command-no-arguments = (no arguments)
ai-requested-command-cancelled = Cancelled
ai-requested-command-error = Error: { $error }
ai-requested-command-copy-json = Copy JSON
ai-orchestration-new-environment = New environment
ai-usage-credits-spent-total = Credits spent (total)
ai-usage-credits-spent = Credits spent
ai-usage-tool-calls = Tool calls
ai-usage-context-window-used = Context window used
ai-usage-files-changed = Files changed
ai-usage-diffs-applied = Diffs applied
ai-usage-commands-executed = Commands executed
ai-usage-time-to-first-token = Time to first token
ai-usage-total-response-time = Total agent response time
terminal-custom-models = Custom models
terminal-custom-model-router-title = Custom Model Router
terminal-custom-model-router-description = Routes each request to a specific model according to your routing rules instead of using one fixed model.
terminal-save-ssh-access-path = Save SSH access path
terminal-secrets-search-or-create = Search secrets or create a new one
terminal-secrets-new = New { $name }
terminal-secrets-none-found = No secrets found
terminal-secrets-load-failed = Unable to load secrets
terminal-queued-to-send = to send
app-menu-new-window = New Window
app-menu-save-new = Save New…
app-menu-launch-configurations = Launch Configurations
app-menu-warp = InfiniShell
app-menu-preferences = Preferences
app-menu-privacy-policy = Privacy Policy…
app-menu-debug = Debug
app-menu-set-default-terminal = Set InfiniShell as Default Terminal
app-menu-file = File
app-menu-edit = Edit
app-menu-use-warp-prompt = Use InfiniShell's Prompt
app-menu-copy-on-select-terminal = Copy on Select within the Terminal
app-menu-synchronize-inputs = Synchronize Inputs
app-menu-view = View
app-menu-toggle-mouse-reporting = Toggle Mouse Reporting
app-menu-toggle-scroll-reporting = Toggle Scroll Reporting
app-menu-toggle-focus-reporting = Toggle Focus Reporting
app-menu-compact-mode = Compact Mode
app-menu-tab = Tab
app-menu-ai = AI
app-menu-blocks = Blocks
app-menu-drive = Drive
app-menu-show-in-band-command-blocks = Show In-band Command Blocks
app-menu-hide-in-band-command-blocks = Hide In-band Command Blocks
app-menu-show-warpified-ssh-blocks = Show Warpified SSH Blocks
app-menu-hide-warpified-ssh-blocks = Hide Warpified SSH Blocks
app-menu-show-initialization-block = Show Initialization Block
app-menu-hide-initialization-block = Hide Initialization Block
app-menu-window = Window
app-menu-enable-shell-debug-mode = Enable Shell Debug Mode (-x) for New Sessions
app-menu-disable-shell-debug-mode = Disable Shell Debug Mode (-x) for New Sessions
app-menu-enable-pty-recording = Enable PTY Recording Mode (warp.pty.recording)
app-menu-disable-pty-recording = Disable PTY Recording Mode (warp.pty.recording)
app-menu-enable-in-band-generators = Enable in-band generators for new sessions
app-menu-disable-in-band-generators = Disable in-band generators for new sessions
app-menu-manually-toggle-network-status = Manually Toggle Network Status
app-menu-export-default-settings-csv = Export Default Settings as CSV to home dir
app-menu-create-anonymous-user = Create anonymous user
app-menu-send-feedback = Send Feedback…
app-menu-help = Help
app-menu-warp-documentation = InfiniShell Documentation…
app-menu-github-issues = GitHub Issues…
app-menu-warp-slack-community = InfiniShell Slack Community…
app-menu-cleanup-storage = Clean Up Storage…
storage-cleanup-title = Clean Up Storage
storage-cleanup-no-remote = Open and focus a connected SSH session to scan its installed InfiniShell extension versions.
storage-cleanup-unsupported-shell = Remote storage cleanup currently supports POSIX SSH hosts only.
storage-cleanup-scan-failed = Could not scan the remote server: { $error }
storage-cleanup-no-unused = No unused remote extension versions were found on { $host }.
storage-cleanup-confirm-title = Clean up unused extensions on { $host }?
storage-cleanup-confirm-info = The following versions are not current or running and will be removed ({ $size }):
storage-cleanup-confirm-button = Clean Up
storage-cleanup-success = Removed { $count } unused remote extension versions and freed { $size }.
storage-cleanup-failed = Failed to clean up remote extension versions: { $error }
storage-cleanup-partial = Removed { $removed } versions; { $skipped } were protected or could not be removed.
workspace-update-and-relaunch-warp = Update and relaunch InfiniShell
workspace-updating-to-version = Updating to ({ $version })
workspace-update-warp-manually = Update InfiniShell manually
pane-get-started-title = Get started
pane-new-tab-title = New tab

# =============================================================================
# SECTION: terminal-runtime
# Files: app/src/terminal/view.rs
# =============================================================================

terminal-banner-completions-not-working-prefix = Seems like your completions are not working (
terminal-banner-more-info-lower = more info
terminal-banner-more-info = More info
terminal-banner-completions-not-working-middle = ). Enabling tmux Warpify in {" "}
terminal-banner-settings = settings
terminal-banner-completions-not-working-suffix =  may resolve this issue.
terminal-banner-shell-config-incompatible = Your shell configuration is incompatible with InfiniShell…{"  "}
terminal-banner-did-you-intend = Did you intend {" "}
terminal-banner-move-cursor =  to move the cursor?
terminal-toast-powershell-subshells-not-supported = PowerShell subshells not supported
terminal-dont-ask-again = Don't ask me this again
terminal-clear-upload = Clear upload
terminal-manage-defaults = Manage defaults
terminal-free-credits = Free credits
terminal-cloud-agent-run = Agent run
terminal-agent-header-for-terminal = for terminal
ssh-remote-choice-title = Choose your experience for this remote session:
ssh-remote-choice-install-extension = Install InfiniShell's SSH extension
ssh-remote-choice-install-extension-desc = Install the InfiniShell SSH extension to enable agent features such as file browsing, code review, and intelligent command completions in this session.
ssh-remote-choice-continue-without-installing = Continue without installing
ssh-remote-choice-continue-without-installing-desc = You'll still get a Warpified experience just without the agent features.
ssh-remote-choice-manage-warpify-settings = Manage Warpify settings
ai-document-show-version-history = Show version history
ai-document-update-agent = Update Agent
ai-document-save-and-sync-tooltip = Save and auto-sync this plan to your InfiniShell Drive
ai-document-show-in-warp-drive = Show in InfiniShell Drive
ai-document-save-as-markdown-file = Save as Markdown file
ai-document-attach-to-active-session = Attach to active session
ai-document-copy-plan-id = Copy plan ID
ai-document-plan-id-copied = Plan ID copied to clipboard
ai-document-copy-as-markdown = Copy as Markdown
ai-document-copied-as-markdown = Copied to clipboard as Markdown
ai-conversation-view-in-oz = View run
ai-conversation-view-in-oz-tooltip = View this agent run
ai-block-open-in-github = Open in GitHub
ai-block-open-in-code-review = Open in code review
ai-block-manage-rules = Manage rules
ai-block-review-changes = Review changes
ai-block-open-all-in-code-review = Open all in code review
ai-block-dont-show-again = Don't show again
ai-block-rewind = Rewind
ai-block-rewind-tooltip = Rewind to before this block
ai-block-remove-queued-prompt = Remove queued prompt
ai-block-send-now = Send now
ai-block-check-now =  · Check now
ai-block-check-now-tooltip = Ask the agent to check this command now, skipping its timer.
ai-block-resume-conversation = Resume conversation
ai-block-continue-conversation = Continue conversation
ai-block-fork-conversation = Fork conversation
ai-block-show-credit-usage-details = Show credit usage details
ai-block-follow-up-existing-conversation = Follow up with existing conversation
ai-block-accept = Accept
ai-block-run = Run
ai-block-auto-approve = Auto-approve
ai-block-auto-approve-this-conversation = Auto Approve This Conversation
ai-block-full-access-this-conversation = Full Access This Conversation
ai-block-full-access-description = Skips local approval prompts and may bypass the local command denylist when enabled in Settings. Organization and sandbox policies still apply.
ai-rule-add-rule = Add rule
ai-rule-edit-rule = Edit rule
ai-rule-delete-rule = Delete rule
ai-aws-refresh-credentials = Refresh AWS Credentials
ai-footer-enable-notifications = Enable notifications
ai-footer-enable-notifications-tooltip = Install the Warp plugin to enable rich agent notifications within InfiniShell
ai-footer-notifications-setup-instructions = Notifications setup instructions
ai-footer-install-plugin-instructions-tooltip = View instructions to install the Warp plugin
ai-footer-update-warp-plugin = Update Warp plugin
ai-footer-plugin-update-available-tooltip = A new version of the Warp plugin is available
ai-footer-plugin-update-instructions = Plugin update instructions
ai-footer-plugin-update-instructions-tooltip = View instructions to update the Warp plugin
ai-footer-context-window-usage-tooltip = Context window usage
ai-footer-choose-environment-tooltip = Choose an environment
ai-footer-reasoning-depth-tooltip = Reasoning depth
ai-footer-file-explorer = File explorer
ai-footer-open-file-explorer = Open file explorer
ai-footer-rich-input = Rich Input
ai-footer-open-rich-input = Open Rich Input
ai-footer-open-coding-agent-settings = Open coding agent settings
ai-footer-turn-off-full-access = Full Access is on. Click to return to approval prompts.

settings-ai-full-access-bypass-command-denylist = Allow Full Access to bypass command denylist
settings-ai-full-access-bypass-command-denylist-description = When enabled, Full Access runs locally denylisted commands without asking for confirmation. Organization and sandbox policies still apply.
settings-ai-auto-approve-bypass-command-denylist = Allow auto-approve to bypass command denylist
settings-ai-auto-approve-bypass-command-denylist-description = When enabled, fast forward and auto-approve run denylisted commands without asking for confirmation.
ai-ask-user-question-placeholder = Type your answer and press Enter
ai-ask-user-questions-skipped = Questions skipped
ai-ask-user-answered-question = Answered question
ai-ask-user-answered-all-questions = Answered all { $total } questions
ai-ask-user-answered-count = Answered { $answered_count } of { $total } questions
ai-code-diff-requested-edit-title = Requested Edit
ai-cloud-setup-visit-oz = Open agent setup
ai-inline-code-diff-review-changes = Review changes
ai-execution-profile-name-placeholder = e.g. "YOLO code"
ai-execution-profile-delete-profile = Delete profile
ai-notifications-mark-all-as-read = Mark all as read
ai-assistant-copy-transcript-tooltip = Copy transcript to clipboard
code-comment = Comment
code-copy-file-path = Copy file path
code-select-all = Select all
code-replace-all = Replace all
code-goto-line-placeholder = Line number:Column
code-open-file-unavailable-remote-tooltip = Opening files is unavailable for remote sessions
code-view-markdown-preview = View Markdown preview
markdown-display-mode-rendered = Rendered
markdown-display-mode-raw = Raw
code-review-commit-and-create-pr = Commit and create PR
notebook-link-text-placeholder = Text
notebook-link-url-placeholder = Link (web or file)
notebook-block-embed = Embed
notebook-block-divider = Divider
notebook-insert-block-tooltip = Insert block
notebook-refresh-notebook = Refresh notebook
notebook-refresh-file = Refresh file
notebook-open-in-editor = Open in editor
notebook-sign-in-to-edit = Sign in to edit
editor-custom-keybinding = Custom…
editor-change-keybinding = Change keybinding
autosuggestion-ignore-this-suggestion = Ignore this suggestion
codex-use-latest-model = Use latest codex model
infinishell-launch-visit-repo = Visit the repo
infinishell-launch-title = InfiniShell is now open source
infinishell-launch-description = You, our community, can participate in building InfiniShell using an agent-first workflow.
infinishell-launch-contribute-title = Contribute
infinishell-launch-contribute-description = InfiniShell's client code is now open source. Get started by using the /feedback skill to open an issue, and follow the contribution guidelines here.
infinishell-launch-contribute-link-text = here
infinishell-launch-oad-title = Open Automated Development
infinishell-launch-oad-description = The InfiniShell repo is managed by an agent-first local workflow powered by InfiniShell Agent.
infinishell-launch-auto-model-title = Introducing 'auto (open-weights)'
infinishell-launch-auto-model-description = We've added a new auto model that picks the best open weight model for a task, like Kimi or MiniMax.
hoa-see-whats-new = See what's new
hoa-finish = Finish
session-config-get-warping = Get started
uri-custom-uri-invalid = Custom URI is invalid.
context-node-install-nvm = Install nvm
context-node-install-node = nvm install node
context-node-installed = Installed
context-chip-change-git-branch = Change git branch
context-chip-view-pull-request = View pull request
context-chip-change-working-directory = Change working directory
context-chip-working-directory = Working directory
settings-ai-repo-placeholder = e.g. ~/code-repos/repo
settings-ai-commands-comma-separated-placeholder = Commands, comma separated
settings-ai-regex-example-placeholder = e.g. ls .*
settings-ai-command-supports-regex-placeholder = command (supports regex)
settings-ai-aws-login-placeholder = aws login
settings-ai-default-placeholder = default
settings-working-directory-path-placeholder = Directory path
settings-startup-shell-executable-path-placeholder = Executable path
settings-agent-providers-base-url-placeholder = https://api.deepseek.com/v1
drive-sharing-only-people-invited = Only people invited
drive-sharing-anyone-with-link = Anyone with the link
drive-sharing-only-invited-teammates = Local access only
drive-sharing-teammates-with-link = Local access with link
terminal-warpify-subshell = Warpify subshell
terminal-warpify-subshell-tooltip = Enable InfiniShell shell integration in this session
terminal-use-agent = Use agent
terminal-use-agent-tooltip = Ask InfiniShell Agent to assist
terminal-give-control-back-to-agent = Give control back to agent
terminal-resume-agent-tooltip = Ask InfiniShell Agent to resume
terminal-voice-input-tooltip = Voice input
terminal-attach-file-tooltip = Attach file
terminal-slash-commands-tooltip = Slash commands
terminal-manage-api-keys-tooltip = Manage API keys
terminal-profiles = Profiles
terminal-manage-profiles = Manage profiles
terminal-continue-locally = Continue locally
terminal-fork-conversation-locally-tooltip = Fork this conversation locally
terminal-open-in-warp = Open in InfiniShell
terminal-open-conversation-in-warp-tooltip = Open this conversation in the InfiniShell desktop app
terminal-stop-sharing = Stop sharing
terminal-copy-session-sharing-link = Copy session sharing link
terminal-shared-session-make-editor = Make editor
terminal-shared-session-make-viewer = Make viewer
terminal-shared-session-change-role = Change role
terminal-choose-execution-profile-tooltip = Choose an AI execution profile
terminal-choose-agent-model-tooltip = Choose an agent model
terminal-input-cli-agent-rich-input-hint = Tell the agent what to build…
terminal-input-enter-prompt-for-agent = Enter a prompt for { $agent }…
terminal-input-cloud-agent-hint = Kick off an agent
terminal-input-a11y-label = Command Input.
terminal-input-a11y-helper = Input your shell command, press enter to execute. Press cmd-up to navigate to output of previously executed commands. Press cmd-l to re-focus command input.
terminal-input-ai-command-search-hint = Type '#' for AI command suggestions
terminal-input-run-commands-hint = Run commands
terminal-input-agent-hint-deploy-react-vercel = Ask anything e.g. Deploy my React app to Vercel and set up environment variables
terminal-input-agent-hint-debug-python-ci = Ask anything e.g. Help me debug why my Python tests are failing in CI
terminal-input-agent-hint-setup-microservice = Ask anything e.g. Set up a new microservice with Docker and create the deployment pipeline
terminal-input-agent-hint-fix-node-memory-leak = Ask anything e.g. Find and fix the memory leak in my Node.js application
terminal-input-agent-hint-backup-postgres = Ask anything e.g. Create a backup script for my PostgreSQL database and schedule it
terminal-input-agent-hint-migrate-mysql-postgres = Ask anything e.g. Help me migrate my data from MySQL to PostgreSQL
terminal-input-agent-hint-monitor-aws = Ask anything e.g. Set up monitoring and alerts for my AWS infrastructure
terminal-input-agent-hint-build-fastapi = Ask anything e.g. Build a REST API for my mobile app using FastAPI
terminal-input-agent-hint-optimize-sql = Ask anything e.g. Help me optimize my SQL queries that are running slowly
terminal-input-agent-hint-github-actions = Ask anything e.g. Create a GitHub Actions workflow to automatically deploy on merge
terminal-input-agent-hint-redis-cache = Ask anything e.g. Set up Redis caching for my web application
terminal-input-agent-hint-kubernetes-pods = Ask anything e.g. Help me troubleshoot why my Kubernetes pods keep crashing
terminal-input-agent-hint-bigquery-pipeline = Ask anything e.g. Build a data pipeline to process CSV files and load them into BigQuery
terminal-input-agent-hint-ssl-https = Ask anything e.g. Set up SSL certificates and configure HTTPS for my domain
terminal-input-agent-hint-refactor-legacy-code = Ask anything e.g. Help me refactor this legacy code to use modern design patterns
terminal-input-agent-hint-unit-tests = Ask anything e.g. Create unit tests for my authentication service
terminal-input-agent-hint-elk-logs = Ask anything e.g. Set up log aggregation with ELK stack for my distributed system
terminal-input-agent-hint-oauth-express = Ask anything e.g. Help me implement OAuth2 authentication in my Express.js app
terminal-input-agent-hint-optimize-docker = Ask anything e.g. Optimize my Docker images to reduce build times and size
terminal-input-agent-hint-ab-testing = Ask anything e.g. Set up A/B testing infrastructure for my web application
terminal-input-steer-agent-hint = Steer the running agent
terminal-input-steer-agent-backspace-hint = Steer the running agent, or backspace to exit
terminal-input-follow-up-hint = Ask a follow up
terminal-input-follow-up-backspace-hint = Ask a follow up, or backspace to exit
terminal-input-queue-follow-up-hint = Queue a follow up for the running agent
terminal-input-queue-follow-up-backspace-hint = Queue a follow up for the running agent, or backspace to exit
terminal-input-child-queue-follow-up = Queue a follow up for the { $agent } agent
terminal-input-child-steer = Steer the { $agent } agent
terminal-input-child-follow-up = Ask the { $agent } agent a follow up
terminal-input-handoff-cloud = Hand off to the cloud
terminal-banner-use-emacs-bindings = Yes, use Emacs-style bindings
terminal-banner-keep-ide-bindings = No, keep IDE bindings
terminal-agent-permission-run = InfiniShell Agent needs your permission to run `{ $command }`
terminal-agent-permission-read = InfiniShell Agent needs your permission to read files
terminal-agent-permission-edit = InfiniShell Agent needs your permission to edit a file
terminal-agent-permission-running-shell = InfiniShell Agent needs your permission to interact with a running shell command
terminal-agent-confirmation = InfiniShell Agent needs your confirmation to continue
terminal-block-started-at = Started at: { $date }
terminal-block-completed-at = Completed at: { $date }
terminal-recording-started = PTY recording started: { $path }
terminal-recording-stopped = PTY recording stopped: { $path }
terminal-input-search-queries = Search queries
terminal-input-search-queries-rewind = Search queries to rewind to
terminal-input-search-conversations = Search conversations
terminal-input-search-skills = Search skills
terminal-input-search-models = Search models
terminal-input-search-profiles = Search profiles
terminal-input-search-commands = Search commands
terminal-input-search-prompts = Search prompts
terminal-input-search-indexed-repos = Search indexed repos
terminal-input-search-plans = Search plans
terminal-input-choose-agent-model = Choose agent model
terminal-message-new-agent-conversation = {" "}new /agent conversation
terminal-message-agent-for-new-conversation = /agent for new conversation
terminal-message-selected-text-attached = selected text attached as context
terminal-message-to-remove = {" "}to remove
terminal-message-to-dismiss = {" "}to dismiss
terminal-message-plan-with-agent = {" "}plan with agent
terminal-message-continue-conversation = {" "}to continue conversation
terminal-message-to-execute = {" "}to execute
terminal-message-to-send = {" "}to send
terminal-message-open-conversation-title = {" "}to open '{ $title }'
terminal-message-autodetected = {" "}(autodetected){" "}
terminal-message-to-override = {" "}to override
terminal-message-to-navigate = {" "}to navigate
terminal-message-to-cycle-tabs = {" "}to cycle tabs
terminal-message-to-select = {" "}to select
terminal-message-select-save-profile = {" "}select and save to profile
terminal-message-open-plan = {" "}open plan
terminal-starting-shell = Starting shell…
terminal-input-no-skills-found = No skills found
terminal-model-specs-title = Model Specs
terminal-model-specs-description = InfiniShell's benchmarks for how well a model performs in our harness, the rate at which it consumes credits, and task speed.
terminal-model-specs-reasoning-level-title = Reasoning level
terminal-model-specs-reasoning-level-description = Increased reasoning levels consume more credits and have higher latency, but higher performance for complicated tasks.
terminal-model-auto-mode-title = Auto mode
terminal-model-auto-mode-description = Auto will select the best model for the task. Cost-efficiency optimizes for cost, Responsiveness optimizes for response speed.
terminal-model-banner-base-agent = You're using the base agent. Full terminal use models only apply to the full terminal use agent.
terminal-model-banner-full-terminal-agent = You're using the full terminal use agent. Base models only apply to the base agent.
terminal-filter-block-output-placeholder = Filter block output

# =============================================================================
# SECTION: object-surfaces
# Files: app/src/code_review/**, app/src/notebooks/**, app/src/workflows/**, app/src/drive/**
# =============================================================================

code-review-tooltip-show-file-navigation = Show file navigation
code-review-discard-changes = Discard changes
code-review-create-pr = Create PR
code-review-add-diff-set-context = Add diff set as context
code-review-show-saved-comment = Show saved comment
code-review-add-comment = Add comment
code-review-discard-all = Discard all
code-review-initialize-codebase = Initialize codebase
code-review-initialize-codebase-tooltip = Enables codebase indexing and WARP.md
code-review-open-repository = Open repository
code-review-open-repository-tooltip = Navigate to a repo and initialize it for coding
code-review-open-file = Open file
code-review-add-file-diff-context = Add file diff as context
code-review-copy-file-path = Copy file path
code-review-no-open-changes = No open changes
code-review-header-reviewing-changes = Reviewing code changes
code-review-search-diff-placeholder = Search diff sets or branches to compare…
code-review-one-comment = 1 Comment
code-review-copy-text = Copy text
code-review-file-level-comment-cannot-edit = File-level comments currently can't be edited.
code-review-outdated-comment-cannot-edit = Outdated comments can't be edited.
code-review-view-in-github = View in GitHub
notebook-menu-attach-active-session = Attach to active session
object-menu-open-on-desktop = Open on Desktop
notebook-tooltip-restore-from-trash = Restore notebook from trash
notebook-tooltip-copy-to-personal = Copy notebook contents into your personal workspace
notebook-copy-to-personal = Copy to Personal
notebook-tooltip-copy-to-clipboard = Copy notebook contents to your clipboard
notebook-copy-all = Copy All
object-toast-link-copied = Link copied to clipboard
drive-toast-finished-exporting = Finished exporting objects

# =============================================================================
# SECTION: remaining-settings-tabs-env
# Files: app/src/settings_view/**, app/src/tab_configs/**, app/src/env_vars/**
# =============================================================================

settings-environment-delete-button = Delete environment
settings-language-system-default = System default
settings-language-english = English
tab-config-open-tab = Open Tab
tab-config-make-default = Make default
tab-config-already-default = Already the default
tab-config-edit-config = Edit config
env-vars-restore-tooltip = Restore environment variables from trash
env-vars-variables-label = Variables

# =============================================================================
# SECTION: onboarding-callout
# Files: crates/onboarding/src/callout/view.rs
# =============================================================================

onboarding-callout-meet-input-title = Meet the InfiniShell input
onboarding-callout-meet-input-text-prefix = Your terminal input accepts both terminal commands and agent prompts and automatically detects which you're using. Use
onboarding-callout-meet-input-text-suffix = to lock the input to Agent mode (natural language) or Terminal mode (commands).
onboarding-callout-talk-agent-title = Talk to the agent
onboarding-callout-talk-agent-text = You can type in natural language to engage the agent. Submit the query below to start: What tests exist in this repo, how are they structured, and what do they cover?
onboarding-callout-skip = Skip
onboarding-callout-submit = Submit
onboarding-callout-finish = Finish
onboarding-callout-meet-terminal-title = Meet your terminal input
onboarding-callout-meet-updated-terminal-title = Meet your updated terminal input
onboarding-callout-meet-terminal-text-prefix = Run commands from the terminal, or use
onboarding-callout-meet-terminal-text-suffix = to start or send to the agent.
onboarding-callout-nl-overrides-title = Natural language overrides
onboarding-callout-nl-overrides-text-prefix = You can always override any auto-detection using
onboarding-callout-nl-support-title = Natural language support
onboarding-callout-nl-support-text-prefix = Natural language input is off by default. If enabled, you can type requests in plain English and InfiniShell will autodetect queries for the agent. You can always override them using
onboarding-callout-enable-nl-detection = Enable Natural Language Detection
onboarding-callout-new-agent-title = Introducing InfiniShell's new agent experience
onboarding-callout-new-agent-text = Agent conversations are now their own scoped view outside of your terminal. Simply hit ESC to return to the terminal at any point.
onboarding-callout-updated-agent-input-title = Updated agent input
onboarding-callout-updated-agent-input-project-text = Your agent input will detect natural language as well as commands by default. Use ! to lock the input in bash mode to write commands.\n\nSubmit the query below to have the agent initialize this project, or ⊗ to clear the input and start your own!
onboarding-callout-skip-initialization = Skip initialization
onboarding-callout-initialize = Initialize
onboarding-callout-updated-agent-input-text = Your agent input will detect natural language as well as commands by default. Use ! to lock the input in bash mode to write commands.
onboarding-callout-back-terminal = Back to terminal

# =============================================================================
# SECTION: language
# Files: app/src/settings_view/appearance_page.rs (Language widget + restart modal)
# =============================================================================

language-widget-label = Language
language-widget-secondary = Restart InfiniShell for the change to fully take effect.
language-restart-required-title = Language changed
language-restart-required-body = InfiniShell's UI language has been updated. Some text will switch immediately, but a full restart is required for the change to take effect everywhere.

# =============================================================================
# SECTION: settings
# Files: app/src/settings_view/**
# =============================================================================

# --- ANCHOR-SUB-MOD-NAV (agent-settings-mod) ---
# settings_view/mod.rs SettingsSection Display labels + context menu pane actions

# Sidebar / SettingsSection labels (Display impl)
settings-section-about = About
# InfiniShell: settings-section-account removed alongside the Account settings page.
settings-section-mcp-servers = MCP Servers
settings-section-billing-and-usage = Billing and usage
settings-section-appearance = Appearance
settings-section-features = Features
settings-section-keybindings = Keyboard shortcuts
settings-section-referrals = Referrals
settings-section-shared-blocks = Shared blocks
settings-section-warp-drive = InfiniShell Drive
settings-section-warpify = Warpify
settings-section-network = Network
settings-section-cloud-sync = Cloud Sync
settings-section-ai = AI
settings-section-warp-agent = InfiniShell Agent
settings-section-agent-profiles = Profiles
settings-section-agent-mcp-servers = MCP servers
settings-section-agent-providers = Providers
settings-section-knowledge = Knowledge
settings-section-third-party-cli-agents = Third-party CLI agents
settings-section-code = Code
settings-section-editor-and-code-review = Editor and Code Review
settings-section-cloud-environments = Environments
settings-section-oz-cloud-api-keys = Agent API Keys
settings-title = Settings

# Context menu items (split / close pane)
settings-pane-split-right = Split pane right
settings-pane-split-left = Split pane left
settings-pane-split-down = Split pane down
settings-pane-split-up = Split pane up
settings-pane-close = Close pane

# Debug toggle setting descriptions (command palette)
settings-debug-show-init-block = Show initialization block
settings-debug-hide-init-block = Hide initialization block
settings-debug-show-inband-blocks = Show in-band command blocks
settings-debug-hide-inband-blocks = Hide in-band command blocks

# --- ANCHOR-SUB-ABOUT (agent-settings-about) ---
# 此锚点下放 settings_view/about_page.rs + main_page.rs 字符串
# 命名前缀:settings-about-* / settings-main-*

# about_page.rs
settings-about-copyright = Copyright 2026 InfiniShell
settings-about-automatic-updates-label = Automatic updates
settings-about-automatic-updates-description = When enabled, InfiniShell checks for new versions in the background and downloads the installer to a local cache. The currently running InfiniShell is not touched until you click "Install now" to launch the installer yourself.
settings-about-update-checking = Checking for updates…
settings-about-update-up-to-date = InfiniShell is up to date.
settings-about-update-available = New version { $version } is available.
settings-about-update-downloading = Downloading { $version }… { $progress }
settings-about-update-downloading-init = Downloading { $version }…
settings-about-update-ready = { $version } is downloaded and ready to install.
settings-about-update-check-now = Check for updates
settings-about-update-open-release = Download from GitHub
settings-about-update-install-now = Install now
settings-about-update-install-hint-macos = The installer will open — drag InfiniShell into your Applications folder to finish.
settings-about-update-install-hint-windows = The setup wizard will launch — follow the prompts to finish the upgrade.
settings-about-update-install-hint-linux = The AppImage will be replaced in place and InfiniShell will restart.
settings-about-export-logs = Export logs…
settings-about-export-logs-description = Bundles recent app logs (and MCP / update logs when present) plus a diagnostic summary into a zip you choose where to save, so you can share it for troubleshooting.
settings-about-export-logs-success = Logs exported to { $path }
settings-about-export-logs-failure = Failed to export logs: { $error }

# InfiniShell: main_page.rs (Account / version / autoupdate) strings removed alongside
# the Account settings page. The About page now owns version / update CTAs.


# --- ANCHOR-SUB-MCP (agent-settings-mcp) ---
# 此锚点下放 settings_view/mcp_servers_page.rs 字符串
# 命名前缀:settings-mcp-*
settings-mcp-page-title = MCP Servers
settings-mcp-logout-success-named = Successfully logged out of {$name} MCP server
settings-mcp-logout-success = Successfully logged out of MCP server
settings-mcp-install-modal-busy = Finish the current MCP install before opening another install link.
settings-mcp-unknown-server = Unknown MCP server '{$name}'
settings-mcp-install-from-link-failed = MCP server '{$name}' cannot be installed from this link.

# ---- destructive_mcp_confirmation_dialog.rs ----
settings-mcp-confirm-delete-local-title = Delete MCP server?
settings-mcp-confirm-delete-local-description = This will uninstall and remove this MCP server from this device.
settings-mcp-confirm-delete-shared-title = Delete MCP server?
settings-mcp-confirm-delete-shared-description = This removes the saved MCP server from this device.
settings-mcp-confirm-unshare-title = Remove saved MCP server?
settings-mcp-confirm-unshare-description = This removes the saved MCP server from this device.
settings-mcp-confirm-delete-button = Delete MCP
settings-mcp-confirm-remove-from-team-button = Remove saved copy
settings-mcp-confirm-cancel-button = Cancel

# ---- edit_page.rs ----
settings-mcp-edit-save = Save
settings-mcp-edit-edit-variables = Edit Variables
settings-mcp-edit-delete = Delete MCP
settings-mcp-edit-remove-from-team = Remove saved copy
settings-mcp-edit-editing-disabled-banner = This MCP server cannot be edited from this view.
settings-mcp-edit-add-new-title = Add New MCP Server
settings-mcp-edit-edit-named-title = Edit { $name } MCP Server
settings-mcp-edit-edit-title = Edit MCP Server
settings-mcp-edit-logout-tooltip = Log out
settings-mcp-edit-secrets-error = This MCP server contains secrets. Visit Settings > Privacy to modify your secret redaction settings.
settings-mcp-edit-no-server-error = No MCP Server specified.
settings-mcp-edit-multiple-servers-error = Cannot add multiple MCP servers while editing a single server.

# ---- installation_modal.rs ----
settings-mcp-install-modal-title = Install { $name }
settings-mcp-install-modal-source-shared = Saved preset
settings-mcp-install-modal-source-other-device = From another device
settings-mcp-install-modal-cancel = Cancel
settings-mcp-install-modal-install = Install
settings-mcp-install-modal-no-server = No MCP server selected

# ---- list_page.rs ----
settings-mcp-list-description = Add MCP servers to extend the InfiniShell Agent's capabilities. MCP servers expose data sources or tools to agents through a standardized interface, essentially acting like plugins. Add a custom server, or use the presets to get started with popular servers.
settings-mcp-list-learn-more = Learn more.
settings-mcp-list-empty-state = Once you add a MCP server, it will be shown here.
settings-mcp-list-no-search-results = No search results found
settings-mcp-list-search-placeholder = Search MCP Servers
settings-mcp-list-add-button = Add
settings-mcp-list-file-based-toggle-label = Auto-spawn servers from third-party agents
settings-mcp-list-file-based-description = Automatically detect and spawn MCP servers from globally-scoped third-party AI agent configuration files (e.g. in your home directory). Servers detected inside a repository are never spawned automatically and must be enabled individually in the "Detected from" sections below.
settings-mcp-list-file-based-supported-providers = See supported providers.
settings-mcp-list-template-available-to-install = Available to install
settings-mcp-list-file-based-detected = Detected from config file
settings-mcp-list-toast-server-updated = MCP server updated
settings-mcp-list-section-my-mcps = My MCPs
settings-mcp-list-section-shared-by-warp-and-team = Available from InfiniShell and { $name }
settings-mcp-list-section-shared-by-warp-and-other-devices = Shared by InfiniShell and from other devices
settings-mcp-list-section-shared-from-warp = Shared from InfiniShell
settings-mcp-list-section-detected-from = Detected from { $provider }
settings-mcp-list-chip-global = global
settings-mcp-list-chip-shared-by-creator = Shared by: { $creator }
settings-mcp-list-chip-shared-by-team-member = Saved preset
settings-mcp-list-chip-from-another-device = From another device

# ---- server_card.rs ----
settings-mcp-card-tooltip-show-logs = Show logs
settings-mcp-card-tooltip-log-out = Log out
settings-mcp-card-tooltip-share-server = Share server
settings-mcp-card-tooltip-edit = Edit
settings-mcp-card-tooltip-update-available = Server update available
settings-mcp-card-button-view-logs = View logs
settings-mcp-card-button-edit-config = Edit config
settings-mcp-card-button-set-up = Set up
settings-mcp-card-tools-none = No tools available
settings-mcp-card-tools-available = { $count } tools available
settings-mcp-card-status-offline = Offline
settings-mcp-card-status-starting = Starting server…
settings-mcp-card-status-authenticating = Authenticating…
settings-mcp-card-status-shutting-down = Shutting down…

# ---- update_modal.rs ----
settings-mcp-update-modal-default-name = Server
settings-mcp-update-modal-title = Update { $name }
settings-mcp-update-modal-description = This server has { $count } updates available, which would you like to proceed with?
settings-mcp-update-modal-publisher-another-device = another device
settings-mcp-update-modal-publisher-team-member = a local source
settings-mcp-update-modal-update-from = Update from { $publisher }
settings-mcp-update-modal-version = Version { $version }
settings-mcp-update-modal-cancel = Cancel
settings-mcp-update-modal-update = Update
settings-mcp-update-modal-no-updates = No updates available

# --- ANCHOR-SUB-PLATFORM (agent-settings-platform) ---
# 此锚点下放 settings_view/platform_page.rs 字符串
# 命名前缀:settings-platform-*
settings-platform-section-title = Agent API Keys
settings-platform-description = Create and manage API keys to allow local agents to access your InfiniShell account.
    For more information, visit the
settings-platform-documentation-link = Documentation.
settings-platform-create-button = + Create API Key
settings-platform-modal-title-new = New API key
settings-platform-modal-title-save = Save your key
settings-platform-toast-deleted = API key deleted
settings-platform-column-name = Name
settings-platform-column-key = Key
settings-platform-column-scope = Scope
settings-platform-column-created = Created
settings-platform-column-last-used = Last used
settings-platform-column-expires-at = Expires at
settings-platform-value-never = Never
settings-platform-scope-personal = Personal
settings-platform-scope-team = Team
settings-platform-zero-state-title = No API Keys
settings-platform-zero-state-description = Create a key to manage external access to InfiniShell
settings-platform-create-api-key-description-personal = This API key is tied to your user and can make requests against your InfiniShell account.
settings-platform-create-api-key-description-team = This API key is tied to your team and can make requests on behalf of your team.
settings-platform-create-api-key-name-placeholder = InfiniShell API Key
settings-platform-create-api-key-expiration-one-day = 1 day
settings-platform-create-api-key-expiration-thirty-days = 30 days
settings-platform-create-api-key-expiration-ninety-days = 90 days
settings-platform-create-api-key-label-type = Type
settings-platform-create-api-key-label-expiration = Expiration
settings-platform-create-api-key-error-no-current-team = Unable to create a team API key because there is no current team.
settings-platform-create-api-key-error-create-failed = Failed to create API key. Please try again.
settings-platform-create-api-key-secret-once = This secret key is shown only once. Copy and store it securely.
settings-platform-create-api-key-copied = Copied
settings-platform-create-api-key-done = Done
settings-platform-create-api-key-creating = Creating…
settings-platform-create-api-key-create = Create key
settings-platform-create-api-key-toast-secret-copied = Secret key copied.

# --- ANCHOR-SUB-KEYBINDINGS (agent-settings-keybindings) ---
settings-keybindings-search-placeholder = Search by name or by keys (ex. "cmd d")
settings-keybindings-conflict-warning = This shortcut conflicts with other keybinds
settings-keybindings-button-default = Default
settings-keybindings-button-cancel = Cancel
settings-keybindings-button-clear = Clear
settings-keybindings-button-save = Save
settings-keybindings-press-new-shortcut = Press new keyboard shortcut
settings-keybindings-description = Add your own custom keybindings to existing actions below.
settings-keybindings-use-prefix = Use
settings-keybindings-use-suffix = to reference these keybindings in a side pane at anytime.
settings-keybindings-not-synced-tooltip = Keyboard shortcuts are stored locally on this machine
settings-keybindings-subheader = Configure keyboard shortcuts
settings-keybindings-command-column = Command

# --- ANCHOR-SUB-REFERRALS (agent-settings-referrals) ---
settings-referrals-page-title = Invite a friend to InfiniShell
settings-referrals-anonymous-header = Referral program is unavailable in local InfiniShell builds
settings-referrals-sign-up = Unavailable locally
settings-referrals-link-label = Link
settings-referrals-email-label = Email
settings-referrals-link-error = Failed to load referral code.
settings-referrals-loading = Loading…
settings-referrals-copy-link-button = Copy link
settings-referrals-email-send-button = Send
settings-referrals-email-sending-button = Sending…
settings-referrals-link-copied-toast = Link copied.
settings-referrals-email-success-toast = Successfully sent emails.
settings-referrals-email-failure-toast = Failed to send emails. Please try again.
settings-referrals-email-empty-error = Please enter an email.
settings-referrals-email-invalid-error = Please ensure the following email is valid: { $email }
settings-referrals-reward-intro = Get exclusive InfiniShell goodies when you refer someone*
settings-referrals-claimed-count-singular = Current referral
settings-referrals-claimed-count-plural = Current referrals
settings-referrals-terms-link = Certain restrictions apply.
settings-referrals-terms-contact = { " " }If you have any questions about the referral program, please contact referrals@warp.dev.
settings-referrals-reward-theme = Exclusive theme
settings-referrals-reward-keycaps = Keycaps + stickers
settings-referrals-reward-tshirt = T-shirt
settings-referrals-reward-notebook = Notebook
settings-referrals-reward-cap = Baseball cap
settings-referrals-reward-hoodie = Hoodie
settings-referrals-reward-hydroflask = Premium Hydro Flask
settings-referrals-reward-backpack = Backpack

# --- ANCHOR-SUB-WARPIFY (agent-settings-warpify) ---
settings-warpify-page-title = Warpify
settings-warpify-description-prefix = Configure whether InfiniShell attempts to "Warpify" (add support for blocks, input modes, etc) certain shells.
settings-warpify-learn-more = Learn more
settings-warpify-section-subshells = Subshells
settings-warpify-section-subshells-subtitle = Subshells supported: bash, zsh, and fish.
settings-warpify-section-ssh = SSH
settings-warpify-section-ssh-subtitle = Warpify your interactive SSH sessions.
settings-warpify-added-commands = Added commands
settings-warpify-denylisted-commands = Denylisted commands
settings-warpify-denylisted-hosts = Denylisted hosts
settings-warpify-command-placeholder = command (supports regex)
settings-warpify-host-placeholder = host (supports regex)
settings-warpify-enable-ssh = Warpify SSH Sessions
settings-warpify-install-ssh-extension = Install SSH extension
settings-warpify-install-ssh-extension-description = Controls the installation behavior for InfiniShell's SSH extension when a remote host doesn't have it installed.
settings-warpify-use-tmux = Use tmux Warpify
settings-warpify-tmux-description = The tmux SSH wrapper works in many situations where the default wrapper does not, but you may need to click a button to run Warpify. This setting takes effect in new tabs.
settings-warpify-ssh-tmux-toggle-binding-label = SSH session detection for Warpify

# --- ANCHOR-SUB-NETWORK (network-settings) ---
# Global HTTP proxy settings page (see Issue #72).
settings-network-page-title = Network
settings-network-header = HTTP proxy
settings-network-description = Configure a global proxy for all outbound HTTP / WebSocket requests. Press Enter after editing a field to save.\nNew requests (BYOP model list, test connection, conversation loading, etc.) take effect immediately; long-lived clients constructed at startup (autoupdate, changelog) require an app restart.
settings-network-mode-label = Proxy mode
settings-network-mode-description = System follows OS / env vars (default); Custom uses the URL below; Off disables all proxying.
settings-network-mode-system = System
settings-network-mode-custom = Custom
settings-network-mode-off = Off
settings-network-url-label = Proxy URL
settings-network-url-placeholder = http://proxy.example.com:8080
settings-network-url-description = e.g. http://proxy.corp:8080
settings-network-username-label = Username
settings-network-username-placeholder = Username (optional)
settings-network-username-description = If the proxy requires Basic Auth, fill in the username here.
settings-network-password-label = Password
settings-network-password-placeholder = Password (saved to the OS keyring on submit)
settings-network-password-description = Submitted password is stored in the OS keyring (not in settings.toml).
settings-network-no-proxy-label = No-proxy list
settings-network-no-proxy-placeholder = localhost,127.0.0.1,.internal
settings-network-no-proxy-description = Comma-separated hosts.
settings-network-save = Save
settings-network-clear = Clear
settings-network-test-button = Test connection
settings-network-test-idle-tcp = Probes the proxy host:port via TCP. Tests reachability of the proxy itself, not internet egress — suitable for intranet-only proxies.
settings-network-test-idle-http = Sends a GET to {$url} through the current configuration. Tests internet egress.
settings-network-test-running = Testing…
settings-network-test-success-tcp = ✅ Proxy reachable ({$latency} ms)
settings-network-test-success-http = ✅ Internet reachable ({$latency} ms)
settings-network-test-failed-tcp = ❌ Cannot reach proxy: {$error}
settings-network-test-failed-http = ❌ Connection failed: {$error}

# --- ANCHOR-SUB-CLOUD-SYNC (agent-settings-cloud-sync) ---
# Cloud Sync settings page
settings-cloud-sync-description = Configure cloud synchronization via GitHub Gist or Gitee Gist. Your settings will be encrypted and stored as a secret Gist.
settings-cloud-sync-scope-note = Currently syncing SSH managed server configurations only.
settings-cloud-sync-platform-label = Sync Platform
settings-cloud-sync-platform-description = Select the cloud service for synchronization
settings-cloud-sync-token-label = Access Token
settings-cloud-sync-token-description = Personal access token with gist scope
settings-cloud-sync-token-placeholder = Enter access token…
settings-cloud-sync-operations-header = Sync Operations
settings-cloud-sync-upload-label = Upload
settings-cloud-sync-download-label = Download
settings-cloud-sync-status-header = Sync Status
settings-cloud-sync-local-version-label = Local version
settings-cloud-sync-last-time-label = Last sync time
settings-cloud-sync-last-platform-label = Last sync platform
settings-cloud-sync-local-version = Local version: {$version}
settings-cloud-sync-last-time = Last sync time: {$time}
settings-cloud-sync-last-platform = Last sync platform: {$platform}
settings-cloud-sync-na = N/A
settings-cloud-sync-never = Never
settings-cloud-sync-syncing-upload = Uploading to {$platform}…
settings-cloud-sync-syncing-download = Downloading from {$platform}…
settings-cloud-sync-success-upload = Upload to {$platform} successful (version v{$version})
settings-cloud-sync-success-download = Download from {$platform} successful (version v{$version})
settings-cloud-sync-already-up-to-date = Already up to date (v{$version}), no sync needed
settings-cloud-sync-failed = Failed: {$error}
settings-cloud-sync-conflict-status = Conflict: local v{$local} vs remote v{$remote}
settings-cloud-sync-conflict-status-equal = Versions are equal: local v{$local} = remote v{$remote}
settings-cloud-sync-token-not-configured = {$platform} Token not configured
settings-cloud-sync-conflict-title = Version Conflict
settings-cloud-sync-conflict-description = Remote version (v{$remote}) is newer than local (v{$local}). Forcing upload will overwrite the remote data.
settings-cloud-sync-conflict-description-equal = Remote and local versions are identical. Forcing upload will overwrite the remote data.
settings-cloud-sync-force-upload = Force Upload
settings-cloud-sync-download-confirm-title = Confirm Download
settings-cloud-sync-download-confirm-description = Downloading will replace all local SSH server configurations with the remote version. This action cannot be undone.
settings-cloud-sync-download-confirm-button = Confirm Download
settings-cloud-sync-upload-confirm-title = Confirm Upload
settings-cloud-sync-upload-confirm-description = Uploading will overwrite all remote SSH server configurations with the local version. Gists do not keep history, so this action cannot be undone.
settings-cloud-sync-upload-confirm-button = Confirm Upload
settings-cloud-sync-clear = Clear
settings-cloud-sync-validating = Validating token…
settings-cloud-sync-token-valid = Token valid ({$username})
settings-cloud-sync-token-invalid = Invalid token: {$error}
settings-cloud-sync-auto-sync-label = Auto Sync
settings-cloud-sync-auto-sync-description = Automatically upload on config change and download on app startup

# --- ANCHOR-SUB-AI-PAGE (agent-settings-ai-page) ---
# Section / sub-headers
settings-ai-warp-agent-header = InfiniShell Agent
settings-ai-active-ai-section = Active AI
settings-ai-input-section = Input
settings-ai-mcp-servers-section = MCP Servers
settings-ai-knowledge-section = Knowledge
settings-ai-voice-section = Voice
settings-ai-other-section = Other
settings-ai-third-party-cli-section = Third-party CLI agents
settings-ai-experimental-section = Experimental
settings-ai-aws-bedrock-section = AWS Bedrock
settings-ai-agents-header = Agents
settings-ai-profiles-header = Profiles
settings-ai-models-subheader = Models
settings-ai-permissions-subheader = Permissions
settings-ai-usage-header = Usage
settings-ai-usage-resets = Resets { $date }
settings-ai-grok-code-placeholder = Paste sign-in code
settings-ai-grok-start-failed = Couldn't start Grok login: { $error }
settings-ai-grok-opening-browser = Opening your browser to connect your SuperGrok subscription…
settings-ai-grok-copy-url = Copy URL
settings-ai-grok-connected = SuperGrok subscription connected
settings-ai-grok-connect-failed = Couldn't connect SuperGrok: { $error }
settings-ai-router-description-placeholder = Describe when to use this model…
settings-ai-credits-label = Credits

# Active AI toggle labels
settings-ai-next-command-label = Next Command
settings-ai-prompt-suggestions-label = Prompt Suggestions
settings-ai-suggested-code-banners-label = Suggested Code Banners
settings-ai-natural-language-autosuggestions-label = Natural Language Autosuggestions
settings-ai-git-operations-autogen-label = Commit & Pull Request Generation

# Permissions dropdown options
settings-ai-permission-agent-decides = Agent decides
settings-ai-permission-always-allow = Always allow
settings-ai-permission-always-ask = Always ask
settings-ai-permission-ask-on-first-write = Ask on first write
settings-ai-permission-read-only = Read only
settings-ai-permission-supervised = Supervised
settings-ai-permission-allow-specific-dirs = Allow in specific directories

# Permission row labels
settings-ai-apply-code-diffs = Apply code diffs
settings-ai-read-files = Read files
settings-ai-execute-commands = Execute commands
settings-ai-interact-running-commands = Interact with running commands
settings-ai-call-mcp-servers = Call MCP servers
settings-ai-command-denylist = Command denylist
settings-ai-command-denylist-description = Regular expressions to match commands that the InfiniShell Agent should always ask permission to execute.
settings-ai-command-allowlist = Command allowlist
settings-ai-command-allowlist-description = Regular expressions to match commands that can be automatically executed by the InfiniShell Agent.
settings-ai-directory-allowlist = Directory allowlist
settings-ai-directory-allowlist-description = Give the agent file access to certain directories.
settings-ai-mcp-allowlist = MCP allowlist
settings-ai-mcp-allowlist-description = Allow the InfiniShell Agent to call these MCP servers.
settings-ai-mcp-denylist = MCP denylist
settings-ai-mcp-denylist-description = The InfiniShell Agent will always ask for permission before calling any MCP servers on this list.
settings-ai-info-banner-managed-by-workspace = Some of your permissions are managed by your workspace.

# Models / Profiles
settings-ai-base-model = Base model
settings-ai-base-model-description = This model serves as the primary engine behind the InfiniShell Agent. It powers most interactions and invokes other models for tasks like planning or code generation when necessary. InfiniShell may automatically switch to alternate models based on model availability or for auxiliary tasks such as conversation summarization.
settings-ai-show-model-picker-in-prompt = Show model picker in prompt
settings-ai-codebase-context = Codebase Context
settings-ai-codebase-context-description = Allow the InfiniShell Agent to generate an outline of your codebase that can be used for context. No code is ever stored on our servers.
settings-ai-add-profile = Add Profile
settings-ai-agents-description = Set the boundaries for how your Agent operates. Choose what it can access, how much autonomy it has, and when it must ask for your approval. You can also fine-tune behavior around natural language input, codebase awareness, and more.
settings-ai-profiles-description = Profiles let you define how your Agent operates — from the actions it can take and when it needs approval, to the models it uses for tasks like coding and planning. You can also scope them to individual projects.

# Anonymous / org gates
settings-ai-sign-up = Enable local AI
settings-ai-anonymous-create-account = Local AI features do not require an account.
settings-ai-org-enforced-tooltip = This option is enforced by your organization's settings and cannot be customized.
settings-ai-restricted-billing = Restricted due to billing issue
settings-ai-unlimited = Unlimited

# AI Input section
settings-ai-show-input-hint-text = Show input hint text
settings-ai-show-agent-tips = Show agent tips
settings-ai-show-agent-zero-state-hints = Show Agent shortcut hints
settings-ai-include-agent-commands-in-history = Include agent-executed commands in history
settings-ai-autodetect-agent-prompts = Autodetect agent prompts in terminal input
settings-ai-autodetect-terminal-commands = Autodetect terminal commands in agent input
settings-ai-natural-language-detection = Natural language detection
settings-ai-natural-language-denylist = Natural language denylist
settings-ai-natural-language-denylist-description = Commands listed here will never trigger natural language detection.
settings-ai-let-us-know = Let us know

# MCP Servers
settings-ai-learn-more = Learn more
settings-ai-add-server = Add a server
settings-ai-manage-mcp-servers = Manage MCP servers
settings-ai-file-based-mcp-toggle = Auto-spawn servers from third-party agents
settings-ai-drive-context-label = InfiniShell Drive as agent context
settings-ai-drive-context-description = InfiniShell Agent can use your InfiniShell Drive content to tailor responses to your personal and team workflows and environments, including workflows, notebooks, and environment variables.
settings-ai-mcp-dropdown-header = Select MCP servers

# Knowledge / Rules
settings-ai-rules-label = Rules
settings-ai-suggested-rules-label = Suggested Rules
settings-ai-suggested-rules-description = Let AI suggest rules to save based on your interactions.
settings-ai-manage-rules = Manage rules
settings-ai-rules-description = Rules help the InfiniShell Agent follow your conventions, whether for codebases or specific workflows.

# Voice
settings-ai-voice-input-label = Voice Input
settings-ai-voice-key = Key for Activating Voice Input
settings-ai-voice-key-hint = Press and hold to activate.

# Other section
settings-ai-show-use-agent-footer = Show "Use Agent" footer
settings-ai-use-agent-footer-description = Shows hint to use the "Full Terminal Use"-enabled agent in long running commands.
settings-ai-show-conversation-history = Show conversation history in tools panel
settings-ai-thinking-display = Agent thinking display
settings-ai-thinking-display-description = Controls how reasoning/thinking traces are displayed.
settings-ai-conversation-layout-label = Preferred layout when opening existing agent conversations
settings-ai-conversation-layout-newtab = New Tab
settings-ai-conversation-layout-splitpane = Split Pane
settings-ai-toolbar-layout = Toolbar layout

# Third-party CLI agents
settings-ai-show-coding-agent-toolbar = Show coding agent toolbar
settings-ai-auto-show-rich-input = Auto show/hide Rich Input based on agent status
settings-ai-auto-show-rich-input-tooltip = Requires the Warp plugin for your coding agent
settings-ai-auto-open-rich-input = Auto open Rich Input when a coding agent session starts
settings-ai-auto-dismiss-rich-input = Auto dismiss Rich Input after prompt submission
settings-ai-toolbar-commands-label = Commands that enable the toolbar
settings-ai-toolbar-commands-description = Add regex patterns to show the coding agent toolbar for matching commands.
settings-ai-per-agent-section = Installed agents
settings-ai-per-agent-scanning = Looking for installed agents…
settings-ai-per-agent-empty = No installed CLI agents found.
settings-ai-per-agent-agent-col = Agent
settings-ai-per-agent-toolbar-col = Toolbar
settings-ai-per-agent-tab-menu-col = Tab menu
settings-ai-per-agent-titlebar-col = Title bar
settings-ai-coding-agent-other = Other
settings-ai-coding-agent-select-header = Select coding agent

# Experimental / Agent
settings-ai-cloud-agent-computer-use = Computer use in agents
settings-ai-cloud-agent-computer-use-description = Enable computer use in agent conversations started from the InfiniShell app.

# AWS Bedrock
settings-ai-aws-bedrock-toggle = Use AWS Bedrock credentials
settings-ai-aws-bedrock-description = InfiniShell loads and sends local AWS CLI credentials for Bedrock-supported models.
settings-ai-aws-bedrock-description-managed = InfiniShell loads and sends local AWS CLI credentials for Bedrock-supported models. This setting is managed by your organization.
settings-ai-aws-login-command = Login Command
settings-ai-aws-profile = AWS Profile
settings-ai-aws-auto-login = Automatically run login command
settings-ai-aws-auto-login-description = When enabled, the login command will run automatically when AWS Bedrock credentials expire.
settings-ai-refresh = Refresh

# --- ANCHOR-SUB-FEATURES (agent-settings-features) ---
# settings_view/features_page.rs P0 + P1(category + toggle labels)
# 命名前缀:settings-features-*
settings-features-category-general = General
settings-features-category-session = Session
settings-features-category-keys = Keys
settings-features-category-text-editing = Text Editing
settings-features-category-terminal-input = Terminal Input
settings-features-category-terminal = Terminal
settings-features-category-notifications = Notifications
settings-features-category-workflows = Workflows
settings-features-category-system = System
settings-features-open-links-in-desktop = Open links in desktop app
settings-features-open-links-in-desktop-tooltip = Automatically open links in desktop app whenever possible.
settings-features-restore-session = Restore windows, tabs, and panes on startup
settings-features-persist-conversations = Save agent conversations to local history
settings-features-show-sticky-command-header = Show sticky command header
settings-features-show-link-tooltip = Show tooltip on click on links
settings-features-show-quit-warning = Show warning before quitting/logging out
settings-features-quit-on-last-window-closed = Quit when all windows are closed
settings-features-show-changelog-after-update = Show changelog toast after updates
settings-features-mouse-scroll-multiplier = Lines scrolled by mouse wheel interval
settings-features-auto-open-code-review = Automatically open code review panel
settings-features-max-rows-per-block = Maximum rows in a block
settings-features-ssh-wrapper = InfiniShell SSH Wrapper
settings-features-ssh-auto-discovery = Auto-discover SSH hosts
settings-features-receive-desktop-notifications = Receive desktop notifications from InfiniShell
settings-features-show-in-app-agent-notifications = Show in-app agent notifications
settings-features-confirm-close-shared-session = Confirm before closing read-only session
settings-features-global-hotkey-label = Global hotkey:
settings-features-global-hotkey-not-supported-on-wayland = Not supported on Wayland.
settings-features-autocomplete-symbols = Autocomplete quotes, parentheses, and brackets
settings-features-error-underlining = Error underlining for commands
settings-features-syntax-highlighting = Syntax highlighting for commands
settings-features-completions-while-typing = Open completions menu as you type
settings-features-command-corrections = Suggest corrected commands
settings-features-expand-aliases = Expand aliases as you type
settings-features-middle-click-paste = Middle-click to paste
settings-features-vim-mode = Edit code and commands with Vim keybindings
settings-features-at-context-menu = Enable '@' context menu in terminal mode
settings-features-slash-commands-in-terminal = Enable slash commands in terminal mode
settings-features-outline-codebase-symbols = Outline codebase symbols for '@' context menu
settings-features-show-input-message-bar = Show terminal input message line
settings-features-show-autosuggestion-hint = Show autosuggestion keybinding hint
settings-features-show-autosuggestion-ignore = Show autosuggestion ignore button
settings-features-enable-mouse-reporting = Enable Mouse Reporting
settings-features-enable-scroll-reporting = Enable Scroll Reporting
settings-features-enable-focus-reporting = Enable Focus Reporting
settings-features-use-audible-bell = Use Audible Bell
settings-features-double-click-smart-selection = Double-click smart selection
settings-features-show-help-block-in-new-sessions = Show help block in new sessions
settings-features-copy-on-select = Copy on select
settings-features-show-global-workflows-in-command-search = Show Global Workflows in Command Search (ctrl-r)
settings-features-linux-selection-clipboard = Honor linux selection clipboard
settings-features-prefer-low-power-gpu = Prefer rendering new windows with integrated GPU (low power)
settings-features-use-wayland = Use Wayland for window management
settings-features-use-wayland-tooltip = Enables the use of Wayland
settings-features-ctrl-tab-behavior-label = Ctrl+Tab behavior:
settings-features-extra-meta-key-left-mac = Left Option key is Meta
settings-features-extra-meta-key-right-mac = Right Option key is Meta
settings-features-extra-meta-key-left-other = Left Alt key is Meta
settings-features-extra-meta-key-right-other = Right Alt key is Meta
settings-features-default-shell-header = Default shell for new sessions
settings-features-working-directory-header = Working directory for new sessions
settings-features-notify-agent-task-completed = Notify when an agent completes a task
settings-features-notify-needs-attention = Notify when a command or agent needs your attention to continue
settings-features-play-notification-sounds = Play notification sounds
settings-features-default-session-mode = Default mode for new sessions
settings-features-block-rows-description = Setting the limit above 100k lines may impact performance. Maximum rows supported is { $max_rows }.
settings-features-toast-duration-label = Toast notifications stay visible for
settings-features-tab-key-behavior = Tab key behavior
settings-features-graphics-backend-label = Preferred graphics backend
settings-features-graphics-backend-current = Current backend: { $backend }
settings-features-working-dir-home = Home directory
settings-features-working-dir-previous = Previous session's directory
settings-features-working-dir-custom = Custom directory
settings-features-working-dir-advanced = Advanced
settings-features-working-dir-new-window = New window
settings-features-working-dir-new-tab = New tab
settings-features-working-dir-split-pane = Split pane
settings-features-undo-close-enable = Enable reopening of closed sessions
settings-features-undo-close-grace-period = Grace period (seconds)
settings-features-configure-global-hotkey = Configure Global Hotkey
settings-features-make-default-terminal = Make InfiniShell the default terminal
settings-features-pin-top = Pin to top
settings-features-pin-bottom = Pin to bottom
settings-features-pin-left = Pin to left
settings-features-pin-right = Pin to right
settings-features-default-option = Default
settings-features-tab-behavior-completions = Open completions menu
settings-features-tab-behavior-autosuggestions = Accept autosuggestion
settings-features-tab-behavior-user-defined = User defined
settings-features-new-tab-placement-all = After all tabs
settings-features-new-tab-placement-current = After current tab
settings-features-width-percent = Width %
settings-features-height-percent = Height %
settings-features-autohide-on-focus-loss = Autohides on loss of keyboard focus
settings-features-long-running-prefix = When a command takes longer than
settings-features-long-running-suffix = seconds to complete
settings-features-keybinding-label = Keybinding
settings-features-click-set-global-hotkey = Click to set global hotkey
settings-features-cancel = Cancel
settings-features-save = Save
settings-features-press-new-shortcut = Press new keyboard shortcut
settings-features-change-keybinding = Change keybinding
settings-features-active-screen = Active Screen
settings-features-wayland-window-restore-warning = Window positions won't be restored on Wayland.
settings-features-see-docs = See docs.
settings-features-allowed-values-1-20 = Allowed Values: 1-20
settings-features-supports-floating-1-20 = Supports floating point values between 1 and 20.
settings-features-auto-open-code-review-description = Open the code review panel when the first diff in a conversation is accepted.
settings-features-default-terminal-current = InfiniShell is the default terminal
settings-features-takes-effect-new-sessions = This change will take effect in new sessions
settings-features-seconds = seconds
settings-features-vim-system-clipboard = Set unnamed register as system clipboard
settings-features-vim-status-bar = Show Vim status bar
settings-features-tab-behavior-right-arrow-accepts = → accepts autosuggestions.
settings-features-tab-behavior-key-accepts = { $keybinding } accepts autosuggestions.
settings-features-completions-open-while-typing-sentence = Completions open as you type.
settings-features-completions-open-while-typing-or-key = Completions open as you type (or { $keybinding }).
settings-features-open-completions-unbound = Opening the completion menu is unbound.
settings-features-tab-behavior-key-opens-completions = { $keybinding } opens completion menu.
settings-features-word-characters-label = Characters considered part of a word
settings-features-new-tab-placement = New tab placement
settings-features-linux-selection-clipboard-tooltip = Whether the Linux primary clipboard should be supported.
settings-features-changes-apply-new-windows = Changes will apply to new windows.
settings-features-wayland-description = Enabling this setting disables global hotkey support. When disabled, text may be blurry if your Wayland compositor is using fraction scaling (ex: 125%).
settings-features-restart-warp-to-apply = Restart InfiniShell for changes to take effect.

# --- ANCHOR-SUB-SETTINGS-PAGE-NAV (agent-settings-page-nav) ---
# 此锚点下放 settings_view/{settings_page,nav,delete_environment_confirmation_dialog,directory_color_add_picker,pane_manager}.rs 字符串
# 命名前缀:settings-page-* / settings-nav-* / settings-confirm-* / settings-color-picker-*

# ---- settings_page.rs ----
settings-page-info-icon-tooltip = Click to learn more in docs
settings-page-local-only-icon-tooltip = This setting is not synced to your other devices
settings-page-reset-to-default = Reset to default

# ---- delete_environment_confirmation_dialog.rs ----
settings-confirm-cancel = Cancel
settings-confirm-delete-environment-button = Delete environment
settings-confirm-delete-environment-title = Delete environment?
settings-confirm-delete-environment-description = Are you sure you want to remove the { $name } environment?

# ---- directory_color_add_picker.rs ----
settings-color-picker-add-directory-footer = + Add directory…
settings-color-picker-add-directory-color = Add directory color

# ---- settings_file_footer.rs ----
settings-footer-open-file = Open settings file
settings-footer-alert-open-file = Open file
settings-footer-alert-fix-with-oz = Fix with InfiniShell Agent

# --- ANCHOR-SUB-CODE (agent-settings-code) ---
settings-code-auto-open-review-panel = Automatically open code review panel
settings-code-auto-open-review-panel-desc = Open the code review panel when the first diff in a conversation is accepted.
settings-code-show-code-review-button = Show code review button
settings-code-show-code-review-button-desc = Show a button in the top right of the window to toggle the code review panel.
settings-code-show-diff-stats = Show diff stats on code review button
settings-code-show-diff-stats-desc = Show lines added and removed counts on the code review button.
settings-code-project-explorer = Project explorer
settings-code-project-explorer-desc = Adds an IDE-style project explorer / file tree to the left side tools panel.
settings-code-global-search = Global file search
settings-code-global-search-desc = Adds global file search to the left side tools panel.

# --- ANCHOR-SUB-EXEC-MODAL-BLOCKS (agent-settings-misc) ---
# ---- execution_profile_view ----
settings-exec-profile-edit-button = Edit
settings-exec-profile-auto = Auto
settings-exec-profile-section-models = MODELS
settings-exec-profile-section-permissions = PERMISSIONS
settings-exec-profile-base-model = Base model:
settings-exec-profile-full-terminal-use = Full terminal use:
settings-exec-profile-title-model = Title generation:
settings-exec-profile-active-ai-model = Active AI:
settings-exec-profile-next-command-model = Next Command:
settings-exec-profile-computer-use = Computer use:
settings-exec-profile-apply-code-diffs = Apply code diffs:
settings-exec-profile-read-files = Read files:
settings-exec-profile-execute-commands = Execute commands:
settings-exec-profile-interact-running-commands = Interact with running commands:
settings-exec-profile-ask-questions = Ask questions:
settings-exec-profile-call-mcp-servers = Call MCP servers:
settings-exec-profile-call-web-tools = Call web tools:
settings-exec-profile-chips-none = None
settings-exec-profile-perm-agent-decides = Agent decides
settings-exec-profile-perm-always-allow = Always allow
settings-exec-profile-perm-always-ask = Always ask
settings-exec-profile-perm-unknown = Unknown
settings-exec-profile-perm-ask-on-first-write = Ask on first write
settings-exec-profile-perm-never = Never
settings-exec-profile-perm-never-ask = Never ask
settings-exec-profile-perm-ask-unless-auto-approve = Ask unless auto-approve
settings-exec-profile-perm-on = On
settings-exec-profile-perm-off = Off
settings-exec-profile-directory-allowlist = Directory allowlist:
settings-exec-profile-command-allowlist = Command allowlist:
settings-exec-profile-command-denylist = Command denylist:
settings-exec-profile-mcp-allowlist = MCP allowlist:
settings-exec-profile-mcp-denylist = MCP denylist:

# ---- execution_profile_editor (Profile Editor pane) ----
settings-exec-profile-editor-header = Profile Editor
settings-exec-profile-editor-title = Edit Profile
settings-exec-profile-editor-name-label = Name
settings-exec-profile-editor-default-name-info = Default profile name cannot be changed.
settings-exec-profile-editor-workspace-override-tooltip = This option is enforced by your organization's settings and cannot be customized.
settings-exec-profile-editor-section-models = MODELS
settings-exec-profile-editor-section-permissions = PERMISSIONS
settings-exec-profile-editor-base-model = Base model
settings-exec-profile-editor-base-model-desc = This model serves as the primary engine behind the agent. It powers most interactions and invokes other models for tasks like planning or code generation when necessary. InfiniShell may automatically switch to alternate models based on model availability or for auxiliary tasks such as conversation summarization.
settings-exec-profile-editor-full-terminal-use-model = Full terminal use model
settings-exec-profile-editor-full-terminal-use-model-desc = The model used when the agent operates inside interactive terminal applications like database shells, debuggers, REPLs, or dev servers—reading live output and writing commands to the PTY.
settings-exec-profile-editor-title-model = Title generation model
settings-exec-profile-editor-title-model-desc = The model used to generate concise conversation titles. Defaults to the base model — pick a cheaper/faster model here to save tokens on title summarization without affecting the agent's main reasoning.
settings-exec-profile-editor-active-ai-model = Active AI model
settings-exec-profile-editor-active-ai-model-desc = The model used by proactive AI features: prompt suggestions after a command finishes, natural-language autocomplete in the agent input, and codebase relevance ranking. Defaults to the base model — pick a small/fast model here to keep these features snappy without affecting the agent's main reasoning.
settings-exec-profile-editor-next-command-model = Next Command model
settings-exec-profile-editor-next-command-model-desc = The model used to predict your next shell command (gray inline autosuggestion + zero-state suggestion after a block finishes). Latency-sensitive — pick the smallest/fastest BYOP model you have. Defaults to the base model.
settings-exec-profile-editor-computer-use-model = Computer use model
settings-exec-profile-editor-computer-use-model-desc = The model used when the agent takes control of your computer to interact with graphical applications through mouse movements, clicks, and keyboard input.
settings-exec-profile-editor-apply-code-diffs = Apply code diffs
settings-exec-profile-editor-read-files = Read files
settings-exec-profile-editor-execute-commands = Execute commands
settings-exec-profile-editor-interact-running-commands = Interact with running commands
settings-exec-profile-editor-computer-use = Computer use
settings-exec-profile-editor-ask-questions = Ask questions
settings-exec-profile-editor-call-mcp-servers = Call MCP servers
settings-exec-profile-editor-call-web-tools = Call web tools
settings-exec-profile-editor-call-web-tools-desc = The agent may use web search when helpful for completing tasks.
settings-exec-profile-editor-directory-allowlist = Directory allowlist
settings-exec-profile-editor-directory-allowlist-desc = Give the agent file access to certain directories.
settings-exec-profile-editor-command-allowlist = Command allowlist
settings-exec-profile-editor-command-allowlist-desc = Regular expressions to match commands that can be automatically executed by InfiniShell Agent.
settings-exec-profile-editor-command-denylist = Command denylist
settings-exec-profile-editor-command-denylist-desc = Regular expressions to match commands that InfiniShell Agent should always ask permission to execute.
settings-exec-profile-editor-mcp-allowlist = MCP allowlist
settings-exec-profile-editor-mcp-allowlist-desc = MCP servers that InfiniShell Agent is allowed to call.
settings-exec-profile-editor-mcp-denylist = MCP denylist
settings-exec-profile-editor-mcp-denylist-desc = MCP servers that InfiniShell Agent is not allowed to call.

# ---- agent_assisted_environment_modal ----
settings-env-modal-add-repo = Add repo
settings-env-modal-cancel = Cancel
settings-env-modal-create-environment = Create environment
settings-env-modal-cloud-unavailable = Creating a cloud environment is not available in this build.
settings-env-modal-selected-repos = Selected repos
settings-env-modal-no-repos-selected = No repos selected yet
settings-env-modal-available-repos = Available indexed repos
settings-env-modal-loading = Loading locally indexed repos…
settings-env-modal-empty-no-indexed = No locally indexed repos found yet. Index a repo, then try again.
settings-env-modal-unavailable-build = Local repo selection is unavailable in this build.
settings-env-modal-all-selected = All locally indexed repos are already selected.
settings-env-modal-unknown-repo-name = (unknown)
settings-env-modal-not-git-repo = Selected folder is not a Git repository: { $path }
settings-env-modal-no-directory-selected = No directory selected
settings-env-modal-dialog-title = Select repos for your environment
settings-env-modal-dialog-description-indexed = Select locally indexed repos to provide context for the environment creation agent.
settings-env-modal-dialog-description-default = Select repos to provide context for the environment creation agent.

# ---- show_blocks_view ----
settings-show-blocks-page-title = Shared blocks
settings-show-blocks-unshare-menu-item = Unshare
settings-show-blocks-copy-link = Copy link
settings-show-blocks-deleting = Deleting…
settings-show-blocks-executed-on = Executed on: { $time }
settings-show-blocks-empty = You don't have any shared blocks yet.
settings-show-blocks-loading = Loading blocks…
settings-show-blocks-load-failed = Failed to load blocks. Please try again.
settings-show-blocks-link-copied = Link copied.
settings-show-blocks-unshare-success = Block was successfully unshared.
settings-show-blocks-unshare-failed = Failed to unshare block. Please try again.
settings-show-blocks-confirm-dialog-title = Unshare block
settings-show-blocks-confirm-dialog-text = Are you sure you want to unshare this block?

    It will no longer be accessible by link and will be permanently deleted from InfiniShell servers.
settings-show-blocks-confirm-cancel = Cancel
settings-show-blocks-confirm-unshare = Unshare

# --- ANCHOR-SUB-APPEARANCE (agent-settings-appearance) ---
# 此锚点下放 settings_view/appearance_page.rs 剩余字符串(不含已完成的 Language widget)
# 命名前缀:settings-appearance-*

# Categories
settings-appearance-category-themes = Themes
settings-appearance-category-language = Language
settings-appearance-category-icon = Icon
settings-appearance-category-window = Window
settings-appearance-category-input = Input
settings-appearance-category-panes = Panes
settings-appearance-category-blocks = Blocks
settings-appearance-category-text = Text
settings-appearance-category-cursor = Cursor
settings-appearance-category-tabs = Tabs
settings-appearance-category-fullscreen-apps = Full-screen Apps

# Theme widget
settings-appearance-theme-create-custom = Create your own custom theme
settings-appearance-theme-mode-light = Light
settings-appearance-theme-mode-dark = Dark
settings-appearance-theme-mode-current = Current theme
settings-appearance-theme-sync-os-label = Sync with OS
settings-appearance-theme-sync-os-description = Automatically switch between light and dark themes when your system does.

# Custom App Icon widget
settings-appearance-custom-icon-label = Customize your app icon
settings-appearance-custom-icon-bundle-warning = Changing the app icon requires the app to be bundled.
settings-appearance-custom-icon-restart-warning = You may need to restart InfiniShell for MacOS to apply the preferred icon style.

# Window widgets
settings-appearance-window-custom-size-label = Open new windows with custom size
settings-appearance-window-columns-label = Columns
settings-appearance-window-rows-label = Rows
settings-appearance-window-opacity-label = Window Opacity:
settings-appearance-window-opacity-value = Window Opacity: { $value }
settings-appearance-window-opacity-not-supported = Transparency is not supported with your graphics drivers.
settings-appearance-window-opacity-graphics-warning = The selected graphics settings may not support rendering transparent windows.
settings-appearance-window-opacity-graphics-warning-hint = Try changing the settings for the graphics backend or integrated GPU in Features > System.
settings-appearance-window-blur-radius = Window Blur Radius: { $value }
settings-appearance-window-blur-texture-label = Use Window Blur (Acrylic texture)
settings-appearance-window-blur-texture-not-supported = The selected hardware may not support rendering transparent windows.
settings-appearance-tools-panel-consistent-label = Tools panel visibility is consistent across tabs

# Input
settings-appearance-input-type-label = Input type
settings-appearance-input-type-warp = InfiniShell
settings-appearance-input-type-shell = Shell (PS1)
settings-appearance-input-position-label = Input position
settings-appearance-input-mode-pinned-bottom = Pin to the bottom (InfiniShell mode)
settings-appearance-input-mode-pinned-top = Pin to the top (Reverse mode)
settings-appearance-input-mode-waterfall = Start at the top (Classic mode)
settings-appearance-command-input-waterfall = Start input at the top
settings-appearance-command-input-pinned-top = Pin input to the top
settings-appearance-command-input-pinned-bottom = Pin input to the bottom
settings-appearance-command-toggle-input-mode = Toggle input mode (InfiniShell/Classic)
settings-appearance-command-tab-bar-always = Always show tab bar
settings-appearance-command-tab-bar-windowed = Hide tab bar in full screen
settings-appearance-command-tab-bar-hover = Show tab bar only on hover
settings-appearance-tools-panel-project-explorer-description = Show the project explorer/file tree tab in the tools panel.
settings-appearance-tools-panel-agent-conversations-description = Show the agent conversation history tab in the tools panel.
settings-appearance-tools-panel-global-search-description = Show the global file search tab in the tools panel.
settings-appearance-tools-panel-drive-description = Show the InfiniShell Drive tab in the tools panel.

# Panes
settings-appearance-pane-dim-inactive-label = Dim inactive panes
settings-appearance-pane-focus-follows-mouse-label = Focus follows mouse

# Blocks
settings-appearance-block-compact-label = Compact mode
settings-appearance-block-jump-bottom-label = Show Jump to Bottom of Block button
settings-appearance-block-show-dividers-label = Show block dividers

# Text / Fonts
settings-appearance-font-agent-label = Agent font
settings-appearance-font-match-terminal = Match terminal
settings-appearance-font-ui-label = UI font
settings-appearance-font-terminal-label = Terminal font
settings-appearance-font-terminal-fallback-label = Fallback font
settings-appearance-font-fallback-system = System fallback
settings-appearance-font-view-all-system = View all available system fonts
settings-appearance-font-weight-label = Font weight
settings-appearance-font-size-label = Font size (px)
settings-appearance-font-line-height-label = Line height
settings-appearance-font-reset-default = Reset to default
settings-appearance-font-notebook-size-label = Notebook font size
settings-appearance-markdown-heading-scale-label = Markdown heading font scale
settings-appearance-markdown-heading-scale-description = Scale is relative to the monospace (terminal) font size. Actual size = monospace font size × scale
settings-appearance-markdown-heading-h1-label = H1 scale
settings-appearance-markdown-heading-h2-label = H2 scale
settings-appearance-markdown-heading-h3-label = H3 scale
settings-appearance-markdown-heading-h4-label = H4 scale
settings-appearance-markdown-heading-h5-label = H5 scale
settings-appearance-markdown-heading-h6-label = H6 scale
settings-appearance-font-thin-strokes-label = Use thin strokes
settings-appearance-font-thin-strokes-never = Never
settings-appearance-font-thin-strokes-low-dpi = On low-DPI displays
settings-appearance-font-thin-strokes-high-dpi = On high-DPI displays
settings-appearance-font-thin-strokes-always = Always
settings-appearance-font-min-contrast-label = Enforce minimum contrast
settings-appearance-font-min-contrast-always = Always
settings-appearance-font-min-contrast-named-only = Only for named colors
settings-appearance-font-min-contrast-never = Never
settings-appearance-font-ligatures-label = Show ligatures in terminal
settings-appearance-font-ligatures-perf-tooltip = Ligatures may reduce performance

# Cursor
settings-appearance-cursor-type-label = Cursor type
settings-appearance-cursor-disabled-vim = Cursor type is disabled in Vim mode
settings-appearance-cursor-blink-label = Blinking cursor

# Tabs
settings-appearance-tab-close-position-label = Tab close button position
settings-appearance-tab-close-position-right = Right
settings-appearance-tab-close-position-left = Left
settings-appearance-tab-show-indicators-label = Show tab indicators
settings-appearance-tab-show-code-review-label = Show code review button
settings-appearance-tab-preserve-active-color-label = Preserve active tab color for new tabs
settings-appearance-tab-vertical-layout-label = Use vertical tab layout
settings-appearance-tab-show-vertical-panel-in-restored-windows-label = Show vertical tabs panel in restored windows
settings-appearance-tab-show-vertical-panel-in-restored-windows-description = When enabled, reopening or restoring a window opens the vertical tabs panel even if it was closed when the window was last saved.
settings-appearance-tab-show-title-bar-search-bar-label = Show search bar in title bar
settings-appearance-tab-show-title-bar-search-bar-description = Show the "Search sessions, agents, files…" search bar in the middle of the title bar; click to open the command palette. Disable to leave the slot empty. Only applies to vertical tabs layout.
workspace-title-bar-search-placeholder = Search sessions, agents, files…
settings-appearance-tab-use-prompt-as-title-label = Use latest user prompt as conversation title in tab names
settings-appearance-tab-use-prompt-as-title-description = Show the latest user prompt instead of the generated conversation title for built-in AI and third-party agent sessions in vertical tabs.
settings-appearance-tab-toolbar-layout-label = Header toolbar layout
settings-appearance-tab-directory-colors-label = Directory tab colors
settings-appearance-tab-directory-colors-description = Automatically color tabs based on the directory or repo you're working in.
settings-appearance-tab-directory-color-default-tooltip = Default (no color)
settings-appearance-zen-mode-label = Show the tab bar
settings-appearance-zen-decoration-always = Always
settings-appearance-zen-decoration-windowed = When windowed
settings-appearance-zen-decoration-on-hover = Only on hover

# Full-screen apps
settings-appearance-alt-screen-padding-label = Use custom padding in alt-screen
settings-appearance-alt-screen-uniform-padding-label = Uniform padding (px)

# Zoom
settings-appearance-zoom-label = Zoom
settings-appearance-zoom-secondary = Adjusts the default zoom level across all windows

# --- ANCHOR-SUB-ENVIRONMENTS (agent-settings-environments) ---
settings-environments-page-title = Environments
settings-environments-page-description = Environments define where your ambient agents run. Set one up in minutes via GitHub (recommended), InfiniShell-assisted setup, or manual configuration.
settings-environments-search-placeholder = Search environments…
settings-environments-no-matches = No environments match your search.
settings-environments-section-personal = Personal
settings-environments-section-team-default = Provided by InfiniShell and this device
settings-environments-section-team-named = Shared by InfiniShell and { $team }
settings-environments-env-id-prefix = Env ID: { $id }
settings-environments-detail-image = Image: { $image }
settings-environments-detail-repos = Repos: { $repos }
settings-environments-detail-setup-commands = Setup commands: { $commands }
settings-environments-last-edited = Last edited: { $time }
settings-environments-last-used = Last used: { $time }
settings-environments-last-used-never = Last used: never
settings-environments-view-my-runs = View my runs
settings-environments-tooltip-share = Share
settings-environments-tooltip-edit = Edit
settings-environments-empty-header = You haven’t set up any environments yet.
settings-environments-empty-subheader = Choose how you’d like to set up your environment:
settings-environments-empty-quick-setup-title = Quick setup
settings-environments-empty-suggested-badge = Suggested
settings-environments-empty-quick-setup-subtitle = Select the GitHub repositories you’d like to work with and we’ll suggest a base image and config
settings-environments-empty-use-agent-title = Use the agent
settings-environments-empty-use-agent-subtitle = Choose a locally set up project and we’ll help you set up an environment based on it
settings-environments-button-loading = Loading…
settings-environments-button-retry = Retry
settings-environments-button-authorize = Authorize
settings-environments-button-get-started = Get started
settings-environments-button-launch-agent = Launch agent
settings-environments-toast-update-success = Successfully updated environment
settings-environments-toast-create-success = Successfully created environment
settings-environments-toast-delete-success = Environment deleted successfully
settings-environments-toast-share-success = Successfully shared environment
settings-environments-toast-share-failure = Failed to share environment with team
settings-environments-toast-create-not-logged-in = Unable to create environment: not logged in.
settings-environments-toast-save-not-found = Unable to save: environment no longer exists.
settings-environments-toast-share-no-team = Unable to share environment: you are not currently on a team.
settings-environments-toast-share-not-synced = Unable to share environment: environment is not yet synced.
settings-update-environment-name-placeholder = Environment name
settings-update-environment-docker-image-placeholder = e.g. python:3.11, node:20-alpine
settings-update-environment-repos-placeholder-authed = Enter repos (owner/repo format)
settings-update-environment-repos-placeholder-unauthenticated = Paste repo URL(s)
settings-update-environment-setup-command-placeholder = e.g. cd my-repo && pip install -r requirements.txt
settings-update-environment-description-placeholder = e.g., this environment is for all front end focused agents

# --- ANCHOR-SUB-AGENT-PROVIDERS (agent-settings-agent-providers) ---
# 此锚点下放 settings_view/agent_providers_widget.rs 字符串
# 命名前缀:settings-agent-providers-*
settings-agent-providers-title = Agent providers
settings-agent-providers-description = Configure custom agent providers across multiple protocols: OpenAI-compatible services (DeepSeek, Zhipu GLM, Moonshot, DashScope, SiliconFlow, OpenRouter, and others), Anthropic, Gemini, and local Ollama. You can add models manually by mapping display names to model IDs, or fetch them automatically from the API. Provider metadata is stored in the local settings.toml file; API keys are stored securely in the system keychain.
settings-agent-providers-empty = No providers configured yet. Click [+ Add provider] in the top-right to add one.
settings-agent-providers-placeholder-display-name = No custom provider configured — add one in Settings → AI
settings-agent-providers-placeholder-base-model-name = Not configured
settings-agent-providers-add-button = + Add provider
settings-agent-providers-search-placeholder = Search providers…
settings-agent-providers-quick-add-title = Quick add
settings-agent-providers-refresh-catalog = Refresh catalog
settings-agent-providers-loading-catalog = Loading models.dev catalog… (the first load may take a few seconds)
settings-agent-providers-catalog-empty = models.dev catalog is empty. Click [Refresh catalog] to retry.
settings-agent-providers-catalog-load-failed = Failed to load models.dev catalog. Click [Refresh catalog] to retry.
settings-agent-providers-no-match = No match for "{ $query }"
settings-agent-providers-collapse = Collapse ▲
settings-agent-providers-expand-remaining = Expand remaining { $count } ▼
settings-agent-providers-row-missing = (no editors bound for this provider yet: { $id })
settings-agent-providers-field-name = Name
settings-agent-providers-field-base-url = Base URL
settings-agent-providers-field-api-key = API Key
settings-agent-providers-field-api-type = API Type
settings-agent-providers-api-type-hint = (genai uses this to bind the adapter explicitly, avoiding misdetection by model name. If Base URL is empty, the default will be used: { $url })
settings-agent-providers-responses-state-label = Responses state
settings-agent-providers-responses-state-local = Local / ZDR
settings-agent-providers-responses-state-provider-chain = Provider chain
settings-agent-providers-responses-state-cloud-conversation = Cloud conversation
settings-agent-providers-responses-transport-label = Transport
settings-agent-providers-responses-compaction-label = Automatic compaction
settings-agent-providers-responses-reasoning-label = GPT-5.6 reasoning
settings-agent-providers-responses-reasoning-pro = Pro mode
settings-agent-providers-responses-reasoning-all-turns = All-turn reasoning context
settings-agent-providers-responses-capabilities-label = Agent capabilities
settings-agent-providers-responses-background = Background + resume
settings-agent-providers-responses-background-requires-cloud = Background (requires cloud state + HTTP)
settings-agent-providers-responses-ptc = Programmatic tool calling
settings-agent-providers-responses-multi-agent = Multi-agent Beta (3)
settings-agent-providers-responses-privacy-hint = Local / ZDR does not store the conversation at the provider. Cloud state, background tasks, and third-party tools change the data-retention boundary.
settings-agent-providers-name-placeholder = Custom provider name (e.g. DeepSeek, local Ollama)
settings-agent-providers-api-key-placeholder = sk-... (optional; leave empty for local providers such as Ollama)
settings-agent-providers-models-label = Models ({ $count })
settings-agent-providers-models-empty-hint = No models configured yet. Click [+ Add model] to add manually, or [Fetch from API] to fetch automatically.
settings-agent-providers-models-header-name = Display name
settings-agent-providers-models-header-id = Model ID
settings-agent-providers-models-header-context = Context (tok)
settings-agent-providers-models-header-output = Output (tok)
settings-agent-providers-model-name-placeholder = Display name (e.g. DS-V3 General)
settings-agent-providers-model-id-placeholder = Model ID (the `model` field sent to the API, e.g. deepseek-chat)
settings-agent-providers-model-context-placeholder = Context (tokens)
settings-agent-providers-model-output-placeholder = Output (tokens)
settings-agent-providers-add-model = + Add model
settings-agent-providers-model-modalities = Modalities
settings-agent-providers-model-modality-image = Image
settings-agent-providers-model-modality-pdf = PDF
settings-agent-providers-model-modality-audio = Audio
settings-agent-providers-model-capabilities = Capabilities
settings-agent-providers-model-capability-reasoning = Reasoning
settings-agent-providers-model-capability-tool-calling = Tool calling
settings-agent-providers-remove-model = Remove model
settings-agent-providers-extra-headers = Extra headers
settings-agent-providers-add-header = + Add header
settings-agent-providers-fetch-from-api = Fetch from API
settings-agent-providers-sync-models-dev = Sync from models.dev
settings-agent-providers-remove = Remove
settings-agent-providers-save = Save
settings-agent-providers-saved-toast = Saved
settings-agent-providers-add-custom-endpoint = + Add custom endpoint
settings-agent-providers-add-custom-endpoint-title = Add custom endpoint
settings-agent-providers-edit-custom-endpoint-title = Edit custom endpoint
settings-agent-providers-custom-endpoint-description = Enter the endpoint details below. You can add multiple models and give them aliases for the model picker.
settings-agent-providers-custom-endpoint-api-schema = API schema
settings-agent-providers-custom-endpoint-name = Endpoint name
settings-agent-providers-custom-endpoint-name-placeholder = e.g., My external models
settings-agent-providers-custom-endpoint-url = Endpoint URL
settings-agent-providers-custom-endpoint-url-placeholder = Include https://
settings-agent-providers-custom-endpoint-api-key = API key
settings-agent-providers-custom-endpoint-api-key-placeholder = e.g., sk-...
settings-agent-providers-custom-endpoint-model-name = Model name
settings-agent-providers-custom-endpoint-model-name-placeholder = e.g., GLM-5-FP8
settings-agent-providers-custom-endpoint-model-alias = Model alias (optional)
settings-agent-providers-custom-endpoint-model-alias-placeholder = e.g., GLM-5
settings-agent-providers-custom-endpoint-add-action = Add endpoint
settings-agent-providers-remove-endpoint-action = Remove endpoint
settings-agent-providers-remove-endpoint-title = Remove endpoint?
settings-agent-providers-remove-endpoint-description = Are you sure you want to remove this endpoint? Its models will no longer be available in agent sessions.
settings-agent-providers-change-default-title = Change your default model?
settings-agent-providers-change-default-not-now = Not now
settings-agent-providers-change-default-action = Change default model
settings-agent-providers-change-default-provider-description = You added your own { $provider } API key, but your default model is currently set to { $model }, which won't work without InfiniShell credits. Would you like to change your default model?
settings-agent-providers-change-default-endpoint-description = You added the “{ $endpoint }” custom endpoint, but your default model is currently set to { $model }, which won't work without InfiniShell credits. Would you like to change your default model?
settings-agent-providers-default-model-updated = Default model updated
settings-agent-providers-endpoint-added = Endpoint added
settings-agent-providers-endpoint-saved = Endpoint saved
settings-agent-providers-endpoint-removed = Endpoint removed
settings-ai-add-router = + Add router
settings-ai-context-window-label = Context window (tokens)

# ---- AI page (settings_view/ai_page.rs) ----
settings-ai-title = AI
settings-ai-active-ai = Active AI
settings-ai-input-autodetection = terminal command autodetection in agent input
settings-ai-input-autodetection-legacy = natural language detection
settings-ai-next-command-description = Let AI suggest the next command to run based on your command history, outputs, and common workflows.
settings-ai-prompt-suggestions-description = Let AI suggest natural language prompts, as inline banners in the input, based on recent commands and their outputs.
settings-ai-suggested-code-banners-description = Let AI suggest code diffs and queries as inline banners in the blocklist, based on recent commands and their outputs.
settings-ai-natural-language-autosuggestions = Let AI suggest natural language autosuggestions, based on recent commands and their outputs.
settings-ai-git-operations-autogen-description = Let AI generate commit messages and pull request titles and descriptions.

# =============================================================================
# SECTION: banner
# Files: app/src/banner/**
# =============================================================================

banner-dont-show-again = Don't show me again

# =============================================================================
# SECTION: quit-warning
# Files: app/src/quit_warning/mod.rs
# =============================================================================

# ---- Dialog titles ----
quit-warning-title-pane = Close pane?
quit-warning-title-tab-singular = Close tab?
quit-warning-title-tab-plural = Close tabs?
quit-warning-title-window = Close window?
quit-warning-title-app = Quit InfiniShell?
quit-warning-title-editor-tab = Save changes?

# ---- Buttons ----
quit-warning-button-confirm-close = Yes, close
quit-warning-button-confirm-quit = Yes, quit
quit-warning-button-save = Save
quit-warning-button-discard = Don't Save
quit-warning-button-show-processes = Show running processes
quit-warning-button-cancel = Cancel

# ---- Warning body lines ----
# Suffix appended to each warning line, indicating the scope.
quit-warning-suffix-tab = { " " }in this tab.
quit-warning-suffix-window = { " " }in this window.
quit-warning-suffix-pane = { " " }in this pane.
quit-warning-suffix-default = .

# Process info: "{count} process(es) running" with optional window/tab qualifier.
quit-warning-processes-running = You have { $count } { $count ->
        [one] process
       *[other] processes
    } running
quit-warning-processes-in-windows = { " " }in { $count } windows
quit-warning-processes-in-tabs = { " " }in { $count } tabs

# Shared sessions line.
quit-warning-shared-sessions = You are sharing { $count } { $count ->
        [one] session
       *[other] sessions
    }

# Unsaved code changes (generic scope).
quit-warning-unsaved-changes = You have unsaved file changes

# Unsaved code changes for a specific editor tab.
quit-warning-unsaved-editor-tab = Do you want to save the changes you made to { $file }? Your changes will be discarded if you don't save them.
quit-warning-unsaved-editor-tab-fallback-name = this file

# --- ANCHOR-SUB-RULES-PAGE (agent-rules-page) ---
# Manage Rules 页面(InfiniShell Drive 中的 AI Fact Collection)。
rules-collection-name = Rules

# --- ANCHOR-SUB-KEYBINDING-DESC (agent-keybinding-descriptions) ---
# Description 文案 for keyboard binding entries shown in the Settings >
# Keyboard Shortcuts page and the command palette. Each key corresponds to
# a binding registered via `EditableBinding::new(name, description, action)`
# or `BindingDescription::new("…")`. The binding `name` (e.g.
# `workspace:open_settings_file`) is **not** translated — it is a protocol
# field used to persist user-customised shortcuts.

# Tabs / sessions
keybinding-desc-workspace-cycle-next-session = Switch to next tab
keybinding-desc-workspace-cycle-prev-session = Switch to previous tab
keybinding-desc-workspace-add-window = Create New Window
keybinding-desc-workspace-new-file = New File
keybinding-desc-workspace-zoom-in = Zoom In
keybinding-desc-workspace-zoom-out = Zoom Out
keybinding-desc-workspace-reset-zoom = Reset Zoom
keybinding-desc-workspace-increase-font-size = Increase font size
keybinding-desc-workspace-decrease-font-size = Decrease font size
keybinding-desc-workspace-reset-font-size = Reset font size to default
keybinding-desc-workspace-increase-zoom = Increase zoom level
keybinding-desc-workspace-decrease-zoom = Decrease zoom level
keybinding-desc-workspace-reset-zoom-level = Reset zoom level to default
keybinding-desc-workspace-save-launch-config = Save new launch configuration

# Project Explorer / panels
keybinding-desc-workspace-toggle-project-explorer = Toggle project explorer
keybinding-desc-workspace-toggle-project-explorer-menu = Project Explorer
keybinding-desc-workspace-show-theme-chooser = Open theme picker
keybinding-desc-workspace-toggle-tab-configs-menu = Open tab configs menu

# Switch to N-th tab
keybinding-desc-workspace-activate-1st-tab = Switch to 1st tab
keybinding-desc-workspace-activate-2nd-tab = Switch to 2nd tab
keybinding-desc-workspace-activate-3rd-tab = Switch to 3rd tab
keybinding-desc-workspace-activate-4th-tab = Switch to 4th tab
keybinding-desc-workspace-activate-5th-tab = Switch to 5th tab
keybinding-desc-workspace-activate-6th-tab = Switch to 6th tab
keybinding-desc-workspace-activate-7th-tab = Switch to 7th tab
keybinding-desc-workspace-activate-8th-tab = Switch to 8th tab
keybinding-desc-workspace-activate-last-tab = Switch to last tab
keybinding-desc-workspace-activate-prev-tab = Activate previous tab
keybinding-desc-workspace-activate-next-tab = Activate next tab

# Pane navigation
keybinding-desc-pane-group-navigate-prev = Activate previous pane
keybinding-desc-pane-group-navigate-next = Activate next pane

# Mouse / Notebooks / Workflows / Folders
keybinding-desc-workspace-toggle-mouse-reporting = Toggle Mouse Reporting
keybinding-desc-workspace-create-personal-notebook = Create a new personal notebook
keybinding-desc-workspace-create-personal-notebook-menu = New Personal Notebook
keybinding-desc-workspace-create-personal-workflow = Create a new personal workflow
keybinding-desc-workspace-create-personal-workflow-menu = New Personal Workflow
keybinding-desc-workspace-create-personal-folder = Create a new personal folder
keybinding-desc-workspace-create-personal-folder-menu = New Personal Folder

# New tab variants
keybinding-desc-workspace-new-tab = Create new tab
keybinding-desc-workspace-new-terminal-tab = New Terminal Tab
keybinding-desc-workspace-new-agent-tab = New Agent Tab
keybinding-desc-workspace-new-cloud-agent-tab = New Agent Tab
new-session-create-new-tab = Create New Tab
new-session-create-new-window = Create New Window
new-session-split-pane-down = Split Pane Down
new-session-split-pane-right = Split Pane Right
new-session-split-pane-up = Split Pane Up
new-session-split-pane-left = Split Pane Left
new-session-create-new-tab-with-shell = Create New Tab: { $shell }
new-session-create-new-window-with-shell = Create New Window: { $shell }
new-session-split-pane-with-shell = Split Pane { $direction }: { $shell }
new-session-direction-down = Down
new-session-direction-right = Right
new-session-direction-up = Up
new-session-direction-left = Left

# Left / right panel toggles
keybinding-desc-workspace-toggle-left-panel = Open Left Panel
keybinding-desc-workspace-toggle-right-panel = Toggle code review
keybinding-desc-workspace-toggle-right-panel-menu = Toggle Code Review
keybinding-desc-workspace-toggle-vertical-tabs = Toggle vertical tabs panel
keybinding-desc-workspace-toggle-vertical-tabs-menu = Toggle Vertical Tabs Panel
keybinding-desc-workspace-left-panel-agent-conversations = Left Panel: Agent conversations
keybinding-desc-workspace-left-panel-project-explorer = Left Panel: Project explorer
keybinding-desc-workspace-left-panel-global-search = Left Panel: Global search
keybinding-desc-workspace-left-panel-warp-drive = Left Panel: InfiniShell Drive
keybinding-desc-workspace-left-panel-ssh-manager = Left Panel: SSH Manager
keybinding-desc-workspace-left-panel-skill-manager = Left Panel: Skill Manager
keybinding-desc-workspace-left-panel-projects = Left Panel: Projects
keybinding-desc-workspace-open-global-search = Open global search
keybinding-desc-workspace-open-global-search-menu = Global Search
keybinding-desc-workspace-toggle-warp-drive = Toggle InfiniShell Drive
keybinding-desc-workspace-toggle-warp-drive-menu = InfiniShell Drive
keybinding-desc-workspace-toggle-conversation-list-view = Toggle Agent conversation list view
keybinding-desc-workspace-toggle-conversation-list-view-menu = Agent conversation list view
keybinding-desc-workspace-close-panel = Close focused panel

# Command palette / navigation
keybinding-desc-workspace-toggle-command-palette = Toggle command palette
keybinding-desc-workspace-toggle-command-palette-menu = Command Palette
keybinding-desc-workspace-toggle-navigation-palette = Toggle navigation palette
keybinding-desc-workspace-toggle-navigation-palette-menu = Navigation Palette
keybinding-desc-workspace-toggle-launch-config-palette = Launch configuration palette
keybinding-desc-workspace-toggle-files-palette = Toggle Files Palette
keybinding-desc-workspace-connect-ssh-in-active-terminal = SSH: Connect in current terminal
keybinding-desc-workspace-search-drive = Search InfiniShell Drive
keybinding-desc-workspace-move-tab-left = Move tab left
keybinding-desc-workspace-move-tab-up = move tab up
keybinding-desc-workspace-move-tab-right = Move tab right
keybinding-desc-workspace-move-tab-down = move tab down

# Keybindings settings
keybinding-desc-workspace-toggle-keybindings-page = Toggle keyboard shortcuts
keybinding-desc-workspace-show-keybinding-settings = Open keybindings editor
keybinding-desc-workspace-toggle-block-snackbar = Toggle sticky command header

# Window / tab close
keybinding-desc-workspace-rename-active-tab = Rename the current tab
keybinding-desc-workspace-terminate-app = Quit InfiniShell
keybinding-desc-workspace-close-window = Close Window
keybinding-desc-workspace-close-active-tab = Close the current tab
keybinding-desc-workspace-close-other-tabs = Close other tabs
keybinding-desc-workspace-close-tabs-right = Close tabs to the right
keybinding-desc-workspace-close-tabs-below = close tabs below

# Notifications
keybinding-desc-workspace-toggle-notifications-on = Turn notifications on
keybinding-desc-workspace-toggle-notifications-off = Turn notifications off

# Updates / changelog
keybinding-desc-workspace-update-and-relaunch = Install update and relaunch
keybinding-desc-workspace-check-for-updates = Check for updates
keybinding-desc-workspace-view-changelog = View latest changelog

# Resource center / Drive export / CLI
keybinding-desc-workspace-toggle-resource-center = Toggle resource center
keybinding-desc-workspace-export-all-warp-drive-objects = Export all InfiniShell Drive objects
keybinding-desc-workspace-install-cli = Install InfiniShell Agent CLI command (`oz`)
keybinding-desc-workspace-uninstall-cli = Uninstall InfiniShell Agent CLI command (`oz`)

# AI assistant / agents
keybinding-desc-workspace-toggle-ai-assistant = Toggle InfiniShell AI

# Env vars / prompts
keybinding-desc-workspace-create-personal-env-vars = Create new personal environment variables
keybinding-desc-workspace-create-personal-env-vars-menu = New Personal Environment Variables
keybinding-desc-workspace-create-personal-ai-prompt = Create a new personal prompt
keybinding-desc-workspace-create-personal-ai-prompt-menu = New Personal Prompt

# Focus / import
keybinding-desc-workspace-shift-focus-left = Switch Focus to Left Panel
keybinding-desc-workspace-shift-focus-right = Switch Focus to Right Panel
keybinding-desc-workspace-import-to-personal-drive = Import To Personal Drive

# Drive / repository / AI rules / MCP
keybinding-desc-workspace-open-repository = Open repository
keybinding-desc-workspace-open-repository-menu = Open Repository
keybinding-desc-workspace-open-ai-fact-collection = Open AI Rules
keybinding-desc-workspace-open-mcp-servers = Open MCP Servers
keybinding-desc-workspace-jump-to-latest-toast = Jump to latest agent task
keybinding-desc-workspace-toggle-notification-mailbox = Toggle notification mailbox

# Settings pages
keybinding-desc-workspace-show-settings = Open Settings
keybinding-desc-workspace-show-settings-menu = Settings
# InfiniShell: keybinding-desc-workspace-show-settings-account removed alongside the
# Account settings page.
keybinding-desc-workspace-show-settings-appearance = Open Settings: Appearance
keybinding-desc-workspace-show-settings-appearance-menu = Appearance…
keybinding-desc-workspace-show-settings-features = Open Settings: Features
keybinding-desc-workspace-show-settings-shared-blocks = Open Settings: Shared Blocks
keybinding-desc-workspace-show-settings-shared-blocks-menu = View Shared Blocks…
keybinding-desc-workspace-show-settings-keyboard-shortcuts = Open Settings: Keyboard Shortcuts
keybinding-desc-workspace-show-settings-keyboard-shortcuts-menu = Configure Keyboard Shortcuts…
keybinding-desc-workspace-show-settings-about = Open Settings: About
keybinding-desc-workspace-show-settings-about-menu = About InfiniShell
keybinding-desc-workspace-show-settings-warpify = Open Settings: Warpify
keybinding-desc-workspace-show-settings-warpify-menu = Configure Warpify…
keybinding-desc-workspace-show-settings-ai = Open Settings: AI
keybinding-desc-workspace-show-settings-code = Open Settings: Code
keybinding-desc-workspace-show-settings-referrals = Open Settings: Referrals
keybinding-desc-workspace-show-settings-environments = Open Settings: Environments
keybinding-desc-workspace-show-settings-mcp-servers = Open Settings: MCP Servers
keybinding-desc-workspace-open-settings-file = Open settings file

# Overflow menu / external links
keybinding-desc-workspace-link-to-slack = Join our Slack community (opens external link)
keybinding-desc-workspace-link-to-user-docs = View user docs (opens external link)
keybinding-desc-workspace-send-feedback = Send feedback (opens external link)
keybinding-desc-workspace-send-feedback-oz = Send feedback with InfiniShell Agent
keybinding-desc-workspace-view-logs = View InfiniShell logs
keybinding-desc-workspace-cleanup-storage = Clean up local and remote storage
keybinding-desc-workspace-link-to-privacy-policy = View privacy policy (opens external link)

# Input / terminal / project bindings (registered outside workspace/mod.rs)
keybinding-desc-input-edit-prompt = Edit Prompt
keybinding-desc-terminal-attach-block-as-context = Attach Selected Block as Agent Context
keybinding-desc-terminal-attach-text-as-context = Attach Selected Text as Agent Context
keybinding-desc-terminal-attach-as-context-menu = Attach Selection as Agent Context
keybinding-desc-workspace-init-project = Initiate project for warp
keybinding-desc-workspace-add-current-folder = Add current folder as project

# Workspace debug / crash / heap profile bindings
keybinding-desc-workspace-crash-macos = Crash the app (for testing local crash reporting)
keybinding-desc-workspace-crash-other = Crash the app (for testing local crash reporting)
keybinding-desc-workspace-log-review-comment-send-status = [Debug] Log review comment send status for active tab
keybinding-desc-workspace-panic = Trigger a panic (for testing local panic logging)
keybinding-desc-workspace-open-view-tree-debugger = Open view tree debugger
keybinding-desc-workspace-view-first-time-user-experience = [Debug] View first-time user experience
keybinding-desc-workspace-undismiss-aws-login-banner = [Debug] Un-dismiss AWS login banner
keybinding-desc-workspace-open-oz-launch-modal = [Debug] Open InfiniShell Launch Modal
keybinding-desc-workspace-reset-oz-launch-modal-state = [Debug] Reset InfiniShell Launch Modal State
keybinding-desc-workspace-open-infinishell-launch-modal = [Debug] Open InfiniShell Launch Modal
keybinding-desc-workspace-reset-infinishell-launch-modal-state = [Debug] Reset InfiniShell Launch Modal State
keybinding-desc-workspace-install-opencode-warp-plugin = [Debug] Install OpenCode Warp plugin
keybinding-desc-workspace-use-local-opencode-warp-plugin = [Debug] Use local OpenCode Warp plugin (testing only)
keybinding-desc-workspace-open-session-config-modal = [Debug] Open Session Config Modal
keybinding-desc-workspace-start-hoa-onboarding-flow = [Debug] Start HOA Onboarding Flow
keybinding-desc-workspace-sample-process = Sample Process
keybinding-desc-workspace-dump-heap-profile = Dump heap profile (can only be done once)

# Terminal input bindings
keybinding-desc-input-show-network-log = Show InfiniShell network log
keybinding-desc-input-clear-screen = Clear screen
keybinding-desc-input-toggle-classic-completions = (Experimental) Toggle classic completions mode
keybinding-desc-input-command-search = Command Search
keybinding-desc-input-history-search = History Search
keybinding-desc-input-open-completions-menu = Open completions menu
keybinding-desc-input-workflows = Workflows
keybinding-desc-input-open-ai-command-suggestions = Open AI Command Suggestions
keybinding-desc-input-new-agent-conversation = New agent conversation
keybinding-desc-input-trigger-auto-detection = Trigger Auto Detection
keybinding-desc-input-clear-and-reset-ai-context-menu-query = Clear and reset AI context menu query

# Terminal view bindings
keybinding-desc-terminal-alternate-paste = Alternate terminal paste
keybinding-desc-terminal-toggle-cli-agent-rich-input = Toggle CLI Agent Rich Input
keybinding-desc-terminal-warpify-subshell = Warpify subshell
keybinding-desc-terminal-warpify-ssh-session = Warpify SSH session
keybinding-desc-terminal-accept-prompt-suggestion = Accept Prompt Suggestion
keybinding-desc-terminal-cancel-process-windows = Copy text or cancel active process
keybinding-desc-terminal-cancel-process = Cancel active process
keybinding-desc-terminal-focus-input = Focus terminal input
keybinding-desc-terminal-paste = Paste
keybinding-desc-terminal-copy = Copy
keybinding-desc-terminal-reinput-commands = Reinput selected commands
keybinding-desc-terminal-reinput-commands-sudo = Reinput selected commands as root
keybinding-desc-terminal-find = Find in Terminal
keybinding-desc-terminal-select-bookmark-up = Select the closest bookmark up
keybinding-desc-terminal-select-bookmark-down = Select the closest bookmark down
keybinding-desc-terminal-open-block-context-menu = Open block context menu
keybinding-desc-terminal-toggle-workflows-modal = Toggle workflows modal
keybinding-desc-terminal-copy-git-branch = Copy git branch
keybinding-desc-terminal-clear-blocks = Clear Blocks
keybinding-desc-terminal-cursor-word-left = Move cursor one word to the left within an executing command
keybinding-desc-terminal-cursor-word-right = Move cursor one word to the right within an executing command
keybinding-desc-terminal-cursor-home = Move cursor home within an executing command
keybinding-desc-terminal-cursor-end = Move cursor end within an executing command
keybinding-desc-terminal-delete-word-left = Delete word left within an executing command
keybinding-desc-terminal-delete-line-start = Delete to line start within an executing command
keybinding-desc-terminal-delete-line-end = Delete to line end within an executing command
keybinding-desc-terminal-backward-tabulation = Backward tabulation within an executing command
keybinding-desc-terminal-select-previous-block = Select previous block
keybinding-desc-terminal-select-next-block = Select next block
keybinding-desc-terminal-share-selected-block = Share selected block
keybinding-desc-terminal-bookmark-selected-block = Bookmark selected block
keybinding-desc-terminal-find-within-selected-block = Find within selected block
keybinding-desc-terminal-copy-command-and-output = Copy command and output
keybinding-desc-terminal-copy-command-output = Copy command output
keybinding-desc-terminal-copy-command = Copy command
keybinding-desc-terminal-scroll-up-one-line = Scroll terminal output up one line
keybinding-desc-terminal-scroll-down-one-line = Scroll terminal output down one line
keybinding-desc-terminal-scroll-up-one-page = Scroll terminal output up one page
keybinding-desc-terminal-scroll-down-one-page = Scroll terminal output down one page
keybinding-desc-terminal-scroll-to-top-of-block = Scroll to top of selected block
keybinding-desc-terminal-scroll-to-bottom-of-block = Scroll to bottom of selected block
keybinding-desc-terminal-select-all-blocks = Select all blocks
keybinding-desc-terminal-expand-blocks-above = Expand selected blocks above
keybinding-desc-terminal-expand-blocks-below = Expand selected blocks below
keybinding-desc-terminal-insert-command-correction = Insert Command Correction
keybinding-desc-terminal-setup-guide = Setup Guide
keybinding-desc-terminal-onboarding-warp-input-terminal = [Debug] Onboarding Callout: WarpInput - Terminal
keybinding-desc-terminal-onboarding-warp-input-project = [Debug] Onboarding Callout: WarpInput - Project
keybinding-desc-terminal-onboarding-warp-input-no-project = [Debug] Onboarding Callout: WarpInput - No Project
keybinding-desc-terminal-onboarding-modality-project = [Debug] Onboarding Callout: Modality - Project
keybinding-desc-terminal-onboarding-modality-no-project = [Debug] Onboarding Callout: Modality - No Project
keybinding-desc-terminal-onboarding-modality-terminal = [Debug] Onboarding Callout: Modality - Terminal
keybinding-desc-terminal-import-external-settings = Import External Settings
keybinding-desc-terminal-share-current-session = Share current session
keybinding-desc-terminal-stop-sharing-current-session = Stop sharing current session
keybinding-desc-terminal-toggle-block-filter = Toggle block filter on selected or last block
keybinding-desc-terminal-toggle-sticky-command-header = Toggle Sticky Command Header in Active Pane
keybinding-desc-terminal-toggle-autoexecute-mode = Toggle Auto-execute Mode
keybinding-desc-terminal-toggle-queue-next-prompt = Toggle Queue Next Prompt

# Pane group bindings
keybinding-desc-pane-group-close-current-session = Close Current Session
keybinding-desc-pane-group-split-left = Split pane left
keybinding-desc-pane-group-split-up = Split pane up
keybinding-desc-pane-group-split-down = Split pane down
keybinding-desc-pane-group-split-right = Split pane right
keybinding-desc-pane-group-switch-left = Switch panes left
keybinding-desc-pane-group-switch-right = Switch panes right
keybinding-desc-pane-group-switch-up = Switch panes up
keybinding-desc-pane-group-switch-down = Switch panes down
keybinding-desc-pane-group-resize-left = Resize pane > Move divider left
keybinding-desc-pane-group-resize-right = Resize pane > Move divider right
keybinding-desc-pane-group-resize-up = Resize pane > Move divider up
keybinding-desc-pane-group-resize-down = Resize pane > Move divider down
keybinding-desc-pane-group-toggle-maximize = Toggle Maximize Active Pane

# Root view bindings
keybinding-desc-root-view-toggle-fullscreen = Toggle fullscreen
keybinding-desc-root-view-enter-onboarding-state = [Debug] Enter Onboarding State

# Workflow view bindings
keybinding-desc-workflow-view-save = Save workflow
keybinding-desc-workflow-view-close = Close

# Editor view binding desc (shared by editor/view/mod.rs, code/editor/view/actions.rs, notebooks/editor/view.rs)
keybinding-desc-editor-copy = Copy
keybinding-desc-editor-cut = Cut
keybinding-desc-editor-paste = Paste
keybinding-desc-editor-undo = Undo
keybinding-desc-editor-redo = Redo
keybinding-desc-editor-select-left-by-word = Select one word to the left
keybinding-desc-editor-select-right-by-word = Select one word to the right
keybinding-desc-editor-select-left = Select one character to the left
keybinding-desc-editor-select-right = Select one character to the right
keybinding-desc-editor-select-up = Select up
keybinding-desc-editor-select-down = Select down
keybinding-desc-editor-select-all = Select all
keybinding-desc-editor-select-to-line-start = Select to start of line
keybinding-desc-editor-select-to-line-end = Select to end of line
keybinding-desc-editor-select-to-line-start-cap = Select To Line Start
keybinding-desc-editor-select-to-line-end-cap = Select To Line End
keybinding-desc-editor-clear-and-copy-lines = Copy and clear selected lines
keybinding-desc-editor-add-next-occurrence = Add selection for next occurrence
keybinding-desc-editor-up = Move cursor up
keybinding-desc-editor-down = Move cursor down
keybinding-desc-editor-left = Move cursor left
keybinding-desc-editor-right = Move cursor right
keybinding-desc-editor-move-to-line-start = Move to start of line
keybinding-desc-editor-move-to-line-end = Move to end of line
keybinding-desc-editor-move-to-line-start-short = Move to line start
keybinding-desc-editor-move-to-line-end-short = Move to line end
keybinding-desc-editor-home = Home
keybinding-desc-editor-end = End
keybinding-desc-editor-cmd-down = Move cursor to the bottom
keybinding-desc-editor-cmd-up = Move cursor to the top
keybinding-desc-editor-move-to-and-select-buffer-start = Select and move to the top
keybinding-desc-editor-move-to-and-select-buffer-end = Select and move to the bottom
keybinding-desc-editor-move-forward-one-word = Move forward one word
keybinding-desc-editor-move-backward-one-word = Move backward one word
keybinding-desc-editor-move-forward-one-word-cap = Move Forward One Word
keybinding-desc-editor-move-backward-one-word-cap = Move Backward One Word
keybinding-desc-editor-move-to-paragraph-start = Move to the start of the paragraph
keybinding-desc-editor-move-to-paragraph-end = Move to the end of the paragraph
keybinding-desc-editor-move-to-paragraph-start-short = Move to start of paragraph
keybinding-desc-editor-move-to-paragraph-end-short = Move to end of paragraph
keybinding-desc-editor-move-to-buffer-start = Move to the start of the buffer
keybinding-desc-editor-move-to-buffer-end = Move to the end of the buffer
keybinding-desc-editor-cursor-at-buffer-start = Cursor at buffer start
keybinding-desc-editor-cursor-at-buffer-end = Cursor at buffer end
keybinding-desc-editor-backspace = Remove the previous character
keybinding-desc-editor-cut-word-left = Cut word left
keybinding-desc-editor-cut-word-right = Cut word right
keybinding-desc-editor-delete-word-left = Delete word left
keybinding-desc-editor-delete-word-right = Delete word right
keybinding-desc-editor-cut-all-left = Cut all left
keybinding-desc-editor-cut-all-right = Cut all right
keybinding-desc-editor-delete-all-left = Delete all left
keybinding-desc-editor-delete-all-right = Delete all right
keybinding-desc-editor-delete = Delete
keybinding-desc-editor-clear-lines = Clear selected lines
keybinding-desc-editor-insert-newline = Insert newline
keybinding-desc-editor-fold = Fold
keybinding-desc-editor-unfold = Unfold
keybinding-desc-editor-fold-selected-ranges = Fold selected ranges
keybinding-desc-editor-insert-last-word-prev-cmd = Insert last word of previous command
keybinding-desc-editor-move-backward-one-subword = Move Backward One Subword
keybinding-desc-editor-move-forward-one-subword = Move Forward One Subword
keybinding-desc-editor-select-left-by-subword = Select one subword to the left
keybinding-desc-editor-select-right-by-subword = Select one subword to the right
keybinding-desc-editor-accept-autosuggestion = Accept autosuggestion
keybinding-desc-editor-inspect-command = Inspect Command
keybinding-desc-editor-clear-buffer = Clear command editor
keybinding-desc-editor-add-cursor-above = Add cursor above
keybinding-desc-editor-add-cursor-below = Add cursor below
keybinding-desc-editor-insert-nonexpanding-space = Insert non-expanding space
keybinding-desc-editor-vim-exit-insert-mode = Exit Vim insert mode
keybinding-desc-editor-toggle-comment = Toggle comment
keybinding-desc-editor-go-to-line = Go to line
keybinding-desc-editor-find-in-code-editor = Find in code editor

# Code editor (Code) binding desc
keybinding-desc-code-save-as = Save file as
keybinding-desc-code-close-all-tabs = Close all tabs
keybinding-desc-code-close-saved-tabs = Close saved tabs

# Welcome view binding desc
keybinding-desc-welcome-terminal-session = Terminal session
keybinding-desc-welcome-add-repository = Add repository

# AI assistant panel binding desc
keybinding-desc-ai-assistant-close = Close InfiniShell AI
keybinding-desc-ai-assistant-focus-terminal-input = Focus Terminal Input From InfiniShell AI
keybinding-desc-ai-assistant-restart = Restart InfiniShell AI

# Code review binding desc
keybinding-desc-code-review-save-all = Save all unsaved files in code review
keybinding-desc-code-review-show-find = Show find bar in code review

# Project buttons binding desc
keybinding-desc-project-buttons-open-repository = Open repository
keybinding-desc-project-buttons-create-new-project = Create new project

# Find view binding desc
keybinding-desc-find-next-occurrence = Find the next occurrence of your search query
keybinding-desc-find-prev-occurrence = Find the previous occurrence of your search query

# Notebook file / notebook binding desc
keybinding-desc-notebook-focus-terminal-input-from-file = Focus Terminal Input from File
keybinding-desc-notebook-reload-file = Reload file
keybinding-desc-notebook-increase-font-size = Increase notebook font size
keybinding-desc-notebook-decrease-font-size = Decrease notebook font size
keybinding-desc-notebook-reset-font-size = Reset notebook font size
keybinding-desc-notebook-focus-terminal-input = Focus Terminal Input from Notebook
keybinding-desc-notebook-fb-increase-font-size = Increase font size
keybinding-desc-notebook-fb-decrease-font-size = Decrease font size

# Notebook editor binding desc (extra to shared editor keys)
keybinding-desc-nbeditor-deselect-command = De-select shell commands
keybinding-desc-nbeditor-select-command = Select shell command at cursor
keybinding-desc-nbeditor-select-previous-command = Select previous command
keybinding-desc-nbeditor-select-next-command = Select next command
keybinding-desc-nbeditor-run-commands = Run selected commands
keybinding-desc-nbeditor-toggle-debug = Toggle rich-text debug mode
keybinding-desc-nbeditor-debug-copy-buffer = Copy rich-text buffer
keybinding-desc-nbeditor-debug-copy-selection = Copy rich-text selection
keybinding-desc-nbeditor-log-state = Log editor state
keybinding-desc-nbeditor-edit-link = Create or edit link
keybinding-desc-nbeditor-inline-code = Toggle inline code styling
keybinding-desc-nbeditor-strikethrough = Toggle strikethrough styling
keybinding-desc-nbeditor-underline = Toggle underline styling
keybinding-desc-nbeditor-find = Find in Notebook
keybinding-desc-nbeditor-next-find-match = Focus next match
keybinding-desc-nbeditor-previous-find-match = Focus previous match
keybinding-desc-nbeditor-toggle-regex-find = Toggle regular expression search
keybinding-desc-nbeditor-toggle-case-sensitive-find = Toggle case-sensitive search

# Pane group / undo close binding desc
keybinding-desc-get-started-terminal-session = Terminal session
keybinding-desc-undo-close-reopen-session = Reopen closed session
keybinding-desc-right-panel-toggle-maximize-code-review = Toggle Maximize Code Review Panel

# Workspace sync inputs binding desc
keybinding-desc-workspace-disable-sync-inputs = Stop Synchronizing Any Panes
keybinding-desc-workspace-toggle-sync-inputs-tab = Toggle Synchronizing All Panes in Current Tab
keybinding-desc-workspace-toggle-sync-inputs-all-tabs = Toggle Synchronizing All Panes in All Tabs

# Workspace a11y / debug binding desc
keybinding-desc-workspace-a11y-concise = [a11y] Set concise accessibility announcements
keybinding-desc-workspace-a11y-verbose = [a11y] Set verbose accessibility announcements
keybinding-desc-workspace-copy-access-token = Copy access token to clipboard

# Env var collection binding desc
keybinding-desc-env-var-collection-close = Close

# Auth / share modal binding desc
keybinding-desc-share-block-copy = Copy
keybinding-desc-auth-paste-token = Paste
keybinding-desc-conversation-details-copy = Copy

# Terminal extras binding desc
keybinding-desc-terminal-show-history = Show History
keybinding-desc-terminal-ask-ai-selection = Ask InfiniShell AI about Selection
keybinding-desc-terminal-ask-ai-last-block = Ask InfiniShell AI about last block
keybinding-desc-terminal-ask-ai = Ask InfiniShell AI
keybinding-desc-terminal-load-agent-conversation = Load agent mode conversation (from debug link in clipboard)
keybinding-desc-terminal-toggle-session-recording = Toggle PTY Recording for Session

# Notebook editor extra
keybinding-desc-nbeditor-select-to-paragraph-start = Select to start of paragraph
keybinding-desc-nbeditor-select-to-paragraph-end = Select to end of paragraph

# Misc binding desc(收尾批次:常量/LazyLock/动态描述去硬编码)
keybinding-desc-save-file = Save file
keybinding-desc-new-agent-pane = New Agent Pane
keybinding-desc-edit-code-diff = Edit Code Diff
keybinding-desc-edit-requested-command = Edit requested command
keybinding-desc-set-input-mode-agent = Set Input Mode to Agent Mode
keybinding-desc-set-input-mode-terminal = Set Input Mode to Terminal Mode
keybinding-desc-toggle-hide-cli-responses = Toggle Hide CLI Responses
keybinding-desc-slash-command = Slash command: { $name }
keybinding-desc-take-control-of-running-command = Take control of running command

# --- Terminal zero-state block (welcome chips) ---
terminal-zero-state-title = New terminal session
terminal-zero-state-start-agent = start a new agent conversation
terminal-zero-state-cycle-history = cycle past commands and conversations
terminal-zero-state-open-code-review = open code review
terminal-zero-state-autodetect-prompts = autodetect agent prompts in terminal sessions
terminal-zero-state-dismiss = Don't show again

# --- Rules page (ai/facts/view/rule.rs) ---
rules-description = Rules enhance the agent by providing structured guidelines that help maintain consistency, enforce best practices, and adapt to specific workflows, including codebases or broader tasks.
rules-search-placeholder = Search rules
rules-name-placeholder = e.g. Rust rules
rules-description-placeholder = e.g. Never use unwrap in Rust
rules-zero-state-global = Once you add a rule, it will be shown here.
rules-zero-state-project = Once you generate a WARP.md rules file for a project, it will appear here.
rules-disabled-banner-prefix = Your rules are disabled and won't be used as context in sessions. You can {" "}
rules-disabled-banner-link = turn it back on
rules-disabled-banner-suffix = {" "}anytime.
rules-tab-global = Global
rules-tab-project = Project based
rules-add-button = Add
rules-init-project-button = Initialize Project

# --- Agent view zero-state + message bar ---
agent-zero-state-title = New InfiniShell Agent conversation
agent-zero-state-description = Send a prompt below to start a new conversation
agent-zero-state-description-with-location = Send a prompt below to start a new conversation in `{ $location }`
agent-zero-state-recent-activity = RECENT ACTIVITY
agent-zero-state-hide-hints-tooltip = Hide shortcut hints (re-enable in Settings)
inline-agent-header-prompt-to-interact-command = Prompt agent to interact with `{ $command }`
inline-agent-header-prompt-to-interact-running-command = Prompt agent to interact with the running command
inline-agent-header-waiting-on-instructions = Agent is waiting for instructions
inline-agent-header-waiting-for-command = Agent is waiting for command to exit
inline-agent-header-agent-blocked = Agent needs your permission to continue
inline-agent-header-agent-in-control = Agent is in control
inline-agent-header-user-in-control = User is in control
agent-toolbar-edit-agent-toolbelt = Edit agent toolbelt
agent-toolbar-edit-cli-agent-toolbelt = Edit CLI agent toolbelt
agent-toolbar-available-chips = Available chips
agent-message-bar-get-figma-mcp = Get Figma MCP
agent-message-bar-enable-figma-mcp = Enable Figma MCP
agent-message-bar-enabling = Enabling…
child-agent-default-name = Agent
agent-zero-state-switch-model = switch model
agent-zero-state-go-back-to-terminal = go back to terminal
agent-message-bar-for-help = for help
agent-message-bar-for-commands = for commands
agent-message-bar-open-conversation = open conversation
agent-message-bar-for-code-review = for code review
agent-message-bar-resume-conversation = to resume conversation
agent-message-bar-hide-plan = to hide plan
agent-message-bar-view-plans = to view plans
agent-message-bar-view-plan = to view plan
agent-message-bar-fork-continue = to fork and continue
agent-message-bar-new-pane = {" "}new pane
agent-message-bar-new-tab = {" "}new tab
agent-message-bar-current-pane = {" "}current pane
agent-message-bar-hide-help = to hide help
agent-message-bar-autodetected-shell-command-prefix = autodetected shell command, {" "}
agent-message-bar-autodetected-shell-command = autodetected shell command
agent-message-bar-override = {" "}to override
agent-message-bar-exit-shell-mode = to exit shell mode
agent-message-bar-again-stop-exit = again to stop and exit
agent-message-bar-again-exit = again to exit
agent-message-bar-again-start-new-conversation = again to start new conversation
agent-shortcuts-input-shell-command = input shell command
agent-shortcuts-slash-commands = for slash commands
agent-shortcuts-file-paths-context = for file paths and attaching other context
agent-shortcuts-open-code-review = open code review
agent-shortcuts-toggle-conversation-list = toggle conversation list
agent-shortcuts-search-continue-conversations = search and continue conversations
agent-shortcuts-start-new-conversation = start a new conversation
agent-shortcuts-toggle-auto-accept = toggle auto-accept
agent-shortcuts-pause-agent = pause agent
agent-error-will-resume-when-network-restored = The conversation will resume when network connectivity is restored…
agent-error-attempting-resume-conversation = Attempting to resume the conversation…

# --- ANCHOR-SUB-TOGGLE-PAIR (settings-toggle-pair) ---
toggle-setting-enable = Enable { $suffix }
toggle-setting-disable = Disable { $suffix }

toggle-suffix-active-ai = Active AI
toggle-suffix-ai-input-autodetect-agent = terminal command autodetection in agent input
toggle-suffix-ai-input-autodetect-nld = natural language detection
toggle-suffix-nld-in-terminal = agent prompt autodetection in terminal input
toggle-suffix-next-command = Next Command
toggle-suffix-prompt-suggestions = prompt suggestions
toggle-suffix-code-suggestions = code suggestions
toggle-suffix-nl-autosuggestions = natural language autosuggestions
toggle-suffix-voice-input = voice input
toggle-suffix-codebase-index = codebase index
toggle-suffix-auto-indexing = auto-indexing
toggle-suffix-compact-mode = compact mode
toggle-suffix-themes-sync-os = themes: sync with OS
toggle-suffix-cursor-blink = cursor blink
toggle-suffix-jump-bottom-block = jump to bottom of block button
toggle-suffix-block-dividers = block dividers
toggle-suffix-dim-inactive-panes = dim inactive panes
toggle-suffix-tab-indicators = tab indicators
toggle-suffix-focus-follows-mouse = focus follows mouse
toggle-suffix-zen-mode = zen mode
toggle-suffix-vertical-tabs = vertical tab layout
toggle-suffix-ligature-rendering = ligature rendering
toggle-suffix-copy-on-select = copy on select within the terminal
toggle-suffix-linux-selection-clipboard = linux selection clipboard
toggle-suffix-autocomplete-symbols = autocomplete quotes, parentheses, and brackets
toggle-suffix-restore-session = restore windows, tabs, and panes on startup
toggle-suffix-left-option-meta = Left Option key is Meta
toggle-suffix-left-alt-meta = Left Alt key is Meta
toggle-suffix-right-option-meta = Right Option key is Meta
toggle-suffix-right-alt-meta = Right Alt key is Meta
toggle-suffix-scroll-reporting = scroll reporting
toggle-suffix-completions-while-typing = completions while typing
toggle-suffix-command-corrections = command corrections
toggle-suffix-error-underlining = error underlining
toggle-suffix-syntax-highlighting = syntax highlighting
toggle-suffix-audible-bell = audible terminal bell
toggle-suffix-autosuggestions = autosuggestions
toggle-suffix-autosuggestion-keybinding-hint = autosuggestion keybinding hint
toggle-suffix-ssh-wrapper = InfiniShell SSH wrapper
toggle-suffix-ssh-auto-discovery = auto-discover SSH hosts
toggle-suffix-link-tooltip = show tooltip on click on links
toggle-suffix-quit-warning = quit warning modal
toggle-suffix-alias-expansion = alias expansion
toggle-suffix-middle-click-paste = middle-click paste
toggle-suffix-code-as-default-editor = code as default editor
toggle-suffix-input-hint-text = input hint text
toggle-suffix-vim-keybindings = editing commands with Vim keybindings
toggle-suffix-vim-clipboard = Vim unnamed register as system clipboard
toggle-suffix-vim-status-bar = Vim status bar
toggle-suffix-focus-reporting = focus reporting
toggle-suffix-smart-select = smart select
toggle-suffix-input-message-line = terminal input message line
toggle-suffix-slash-commands-terminal = slash commands in terminal mode
toggle-suffix-integrated-gpu = integrated GPU rendering (low power)
toggle-suffix-wayland = Wayland for window management
toggle-suffix-app-analytics = local diagnostics
toggle-suffix-crash-reporting = crash reporting
toggle-suffix-secret-redaction = secret redaction
toggle-suffix-recording-mode = recording mode
toggle-suffix-inband-generators = in-band generators for new sessions
toggle-suffix-debug-network = debug network status
toggle-suffix-memory-stats = memory statistics

# Set agent thinking display
agent-thinking-display-show-collapse = Set agent thinking display: show & collapse
agent-thinking-display-always-show = Set agent thinking display: always show
agent-thinking-display-never-show = Set agent thinking display: never show

# --- ANCHOR-SUB-EXTERNAL-EDITOR (settings-external-editor) ---
settings-external-editor-choose-default = Choose an editor to open file links
settings-external-editor-choose-code-panels = Choose an editor to open files from the code review panel, project explorer, and global search
settings-external-editor-choose-layout = Choose a layout to open files in InfiniShell
settings-external-editor-tabbed-header = Group files into single editor pane
settings-external-editor-tabbed-desc = When this setting is on, any files opened in the same tab will be automatically grouped into a single editor pane.
settings-external-editor-prefer-markdown = Open Markdown files in InfiniShell's Markdown Viewer by default
settings-external-editor-layout-split-pane = Split Pane
settings-external-editor-layout-new-tab = New Tab
settings-external-editor-default-app = Default App

# =============================================================================
# SECTION: context-menu
# 鼠标右键弹出菜单。surface 前缀:menu-{block,input,ai-block,tab,pane,filetree,codeeditor}-*
# =============================================================================

# --- block 右键菜单(terminal/view.rs) ---
menu-block-copy = Copy
menu-block-copy-url = Copy URL
menu-block-copy-path = Copy path
menu-block-show-in-finder = Show in Finder
menu-block-show-containing-folder = Show containing folder
menu-block-open-in-warp = Open in InfiniShell
menu-block-open-in-editor = Open in editor
menu-block-insert-into-input = Insert into input
menu-block-copy-command = Copy command
menu-block-copy-commands = Copy commands
menu-block-find-within-block = Find within block
menu-block-find-within-blocks = Find within blocks
menu-block-scroll-to-top-of-block = Scroll to top of block
menu-block-scroll-to-top-of-blocks = Scroll to top of blocks
menu-block-scroll-to-bottom-of-block = Scroll to bottom of block
menu-block-scroll-to-bottom-of-blocks = Scroll to bottom of blocks
menu-block-save-as-workflow = Save as workflow
menu-block-ask-warp-ai = Ask InfiniShell AI
menu-block-copy-output = Copy output
menu-block-copy-filtered-output = Copy filtered output
menu-block-toggle-block-filter = Toggle block filter
menu-block-toggle-bookmark = Toggle bookmark
menu-block-copy-prompt = Copy prompt
menu-block-copy-right-prompt = Copy right prompt
menu-block-copy-working-directory = Copy working directory
menu-block-copy-git-branch = Copy git branch
menu-block-edit-prompt = Edit prompt
menu-block-edit-cli-agent-toolbelt = Edit CLI agent toolbelt
menu-block-edit-agent-toolbelt = Edit agent toolbelt
menu-block-split-pane-right = Split pane right
menu-block-split-pane-left = Split pane left
menu-block-split-pane-down = Split pane down
menu-block-split-pane-up = Split pane up
menu-block-close-pane = Close pane
menu-block-clear-blocks = Clear Blocks

# --- input 右键菜单(terminal/view.rs) ---
menu-input-cut = Cut
menu-input-copy = Copy
menu-input-paste = Paste
menu-input-select-all = Select all
menu-input-command-search = Command search
menu-input-ai-command-search = AI command search
menu-input-ask-warp-ai = Ask InfiniShell AI
menu-input-save-as-workflow = Save as workflow
menu-input-hide-hint-text = Hide input hint text
menu-input-show-hint-text = Show input hint text

# --- AI block overflow 菜单(terminal/view.rs) ---
menu-ai-block-copy = Copy
menu-ai-block-copy-prompt = Copy prompt
menu-ai-block-copy-output-as-markdown = Copy output as Markdown
menu-ai-block-copy-url = Copy URL
menu-ai-block-copy-path = Copy path
menu-ai-block-copy-command = Copy command
menu-ai-block-copy-git-branch = Copy git branch
menu-ai-block-save-as-prompt = Save as prompt
menu-ai-block-copy-conversation-text = Copy conversation text
menu-ai-block-fork = Fork
menu-ai-block-fork-from-here = Fork from here
menu-ai-block-rewind-to-before-here = Rewind to before here
menu-ai-block-fork-from-last-query = Fork from last query
menu-ai-block-fork-from-query = Fork from "{ $query }"

# --- tab 右键菜单(tab.rs) ---
menu-tab-stop-sharing = Stop sharing
menu-tab-stop-sharing-all = Stop sharing all
menu-tab-copy-link = Copy link
menu-tab-rename = Rename tab
menu-tab-reset-name = Reset tab name
menu-tab-move-down = Move Tab Down
menu-tab-move-right = Move Tab Right
menu-tab-move-up = Move Tab Up
menu-tab-move-left = Move Tab Left
menu-tab-close = Close tab
menu-tab-close-other = Close other tabs
menu-tab-close-below = Close Tabs Below
menu-tab-close-right = Close Tabs to the Right
menu-tab-save-as-new-config = Save as new config
menu-tab-default-no-color = Default (no color)
menu-tab-copy-title = Copy tab title
menu-tab-copy-pane-title = Copy pane title
menu-tab-copy-branch = Copy branch
menu-tab-copy-working-directory = Copy working directory
menu-tab-copy-pull-request-link = Copy pull request link
menu-tab-new-group-with-tab = New group with tab
menu-tab-move-to-group = Move to group
menu-tab-remove-from-group = Remove from group
menu-tab-group-create-from-tabs = Create group from tabs
menu-tab-group-move-up = Move group up
menu-tab-group-move-left = Move group left
menu-tab-group-move-down = Move group down
menu-tab-group-move-right = Move group right
menu-tab-group-close-all = Close all tabs in group
menu-tab-group-close-above = Close tabs above
menu-tab-group-close-left = Close tabs to the left
menu-tab-group-close-below = Close tabs below
menu-tab-group-close-right = Close tabs to the right
menu-tab-group-pin = Pin group
menu-tab-group-unpin = Unpin group
menu-tab-group-ungroup = Ungroup tabs
menu-tab-group-new-tab = New tab in group
menu-tab-pin = Pin tab
menu-tab-unpin = Unpin tab
menu-tab-untitled-group = Untitled group

# --- pane header 溢出菜单(terminal/view/pane_impl.rs) ---
menu-pane-copy-link = Copy link
menu-pane-stop-sharing-session = Stop session broadcast
menu-pane-open-on-desktop = Open on Desktop

# --- 文件树右键菜单(code/file_tree/view.rs) ---
menu-filetree-open-in-new-pane = Open in new pane
menu-filetree-open-in-new-tab = Open in new tab
menu-filetree-open-file = Open file
menu-filetree-new-file = New file
menu-filetree-cd-to-directory = cd to directory
menu-filetree-reveal-finder = Reveal in Finder
menu-filetree-reveal-explorer = Reveal in Explorer
menu-filetree-reveal-file-manager = Reveal in file manager
menu-filetree-rename = Rename
menu-filetree-delete = Delete
menu-filetree-attach-as-context = Attach as context
menu-filetree-copy-path = Copy path
menu-filetree-copy-relative-path = Copy relative path

# --- 代码编辑器右键菜单(code/local_code_editor.rs) ---
menu-codeeditor-go-to-definition = Go to definition
menu-codeeditor-find-references = Find references

# --- 共享标签:附加为 agent 上下文(blocklist/view_util.rs) ---
menu-attach-as-agent-context = Attach as agent context

# --- ANCHOR-SUB-SLASH-COMMANDS (agent-slash-commands) ---
# Slash command palette descriptions and argument hints
# (app/src/search/slash_command_menu/static_commands/commands.rs)
slash-cmd-agent-desc = Start a new conversation
slash-cmd-cloud-agent-desc = Start a new cloud agent conversation
slash-cmd-add-mcp-desc = Add new MCP server
slash-cmd-reset-statusline-desc = Restore the default status line items and order
slash-cmd-statusline-desc = Configure the status line
slash-cmd-auto-approve-desc = Toggle automatic approval
slash-cmd-mcp-desc = View and manage MCP servers
slash-cmd-view-logs-desc = Bundle your logs into a ZIP archive
slash-cmd-voice-desc = Start voice input (Ctrl-S)
slash-cmd-natural-language-detection-desc = Toggle natural-language detection
slash-cmd-api-keys-desc = View and manage API keys
slash-cmd-connect-grok-desc = Connect your Grok (X Premium / SuperGrok) account
slash-cmd-upgrade-desc = Open the InfiniShell upgrade page in your browser
slash-cmd-theme-desc = Set the color theme
slash-cmd-theme-hint = <auto|light|dark>
slash-cmd-exit-desc = Exit InfiniShell
slash-cmd-status-desc = Show local session status
slash-cmd-pr-comments-desc = Pull GitHub PR review comments
slash-cmd-create-environment-desc = Create an InfiniShell Agent environment (Docker image + repos) via guided setup
slash-cmd-create-environment-hint = <optional repo paths or GitHub URLs>
slash-cmd-docker-sandbox-desc = Create a new Docker sandbox terminal session
slash-cmd-create-new-project-desc = Have InfiniShell Agent walk you through creating a new coding project
slash-cmd-create-new-project-hint = <describe what you want to build>
slash-cmd-open-skill-desc = Open a skill's Markdown file in InfiniShell's built-in editor
slash-cmd-skills-desc = Invoke a skill
slash-cmd-add-prompt-desc = Add a new agent prompt
slash-cmd-add-rule-desc = Add a new global rule for the agent
slash-cmd-open-file-desc = Open a file in InfiniShell's code editor
slash-cmd-open-file-hint = <path/to/file[:line[:col]]> or "@" to search
slash-cmd-rename-tab-desc = Rename the current tab
slash-cmd-rename-tab-hint = <tab name>
slash-cmd-rename-conversation-desc = Rename the current conversation
slash-cmd-rename-conversation-hint = <new title>
slash-cmd-set-tab-color-desc = Set the color of the current tab
slash-cmd-fork-desc = Fork the current conversation in a new pane or a new tab
slash-cmd-fork-hint = <optional prompt to send in forked conversation>
slash-cmd-handoff-desc = Hand off this conversation to a cloud agent
slash-cmd-handoff-hint = <optional follow-up prompt>
slash-cmd-open-code-review-desc = Open code review
slash-cmd-index-desc = Index this codebase
slash-cmd-init-desc = Generate or update an AGENTS.md file
slash-cmd-open-project-rules-desc = Open the project rules file (AGENTS.md)
slash-cmd-open-mcp-servers-desc = Open MCP servers
slash-cmd-open-settings-file-desc = Open settings file (TOML)
slash-cmd-changelog-desc = Open the latest changelog
slash-cmd-feedback-desc = Send feedback
slash-cmd-open-repo-desc = Switch to another indexed repository
slash-cmd-open-rules-desc = View all of your global and project rules
slash-cmd-new-desc = Start a new conversation (alias for /agent)
slash-cmd-clear-desc = Clear the transcript and start a new conversation (alias for /agent)
slash-cmd-model-desc = Switch the base agent model
slash-cmd-host-desc = Switch the cloud agent execution host
slash-cmd-harness-desc = Switch the cloud agent harness
slash-cmd-environment-desc = Switch the cloud agent environment
slash-cmd-profile-desc = Switch the active execution profile
slash-cmd-plan-desc = Prompt the agent to do some research and create a plan for a task
slash-cmd-plan-hint = <describe your task>
slash-cmd-orchestrate-desc = Split a task into subtasks and run them in parallel with multiple agents
slash-cmd-compact-desc = Free up context by summarizing conversation history
slash-cmd-compact-hint = <optional custom summarization instructions>
slash-cmd-compact-and-desc = Compact conversation and then send a follow-up prompt
slash-cmd-compact-and-hint = <prompt to send after compaction>
slash-cmd-queue-desc = Queue a prompt to send after the agent finishes responding
slash-cmd-queue-hint = <prompt to send when agent is done>
slash-cmd-fork-and-compact-desc = Fork current conversation and compact it in the forked copy
slash-cmd-fork-and-compact-hint = <optional prompt to send after compaction>
slash-cmd-fork-from-desc = Fork conversation from a specific query
slash-cmd-continue-locally-desc = Continue this cloud conversation locally
slash-cmd-continue-locally-hint = <optional prompt to send in the local conversation>
slash-cmd-usage-desc = Open billing and usage settings
slash-cmd-remote-control-desc = Start remote control for this session
slash-cmd-cost-desc = Toggle credit usage details
slash-cmd-conversations-desc = Open conversation history
slash-cmd-prompts-desc = Search saved prompts
slash-cmd-rewind-desc = Rewind to a previous point in the conversation
slash-cmd-export-to-clipboard-desc = Export the current conversation to the clipboard in Markdown format
slash-cmd-export-to-file-desc = Export the current conversation to a Markdown file
slash-cmd-export-to-file-hint = <optional filename>
slash-cmd-vim-mode-desc = Toggle Vim mode
slash-cmd-copy-debugging-id-desc = Copy debugging information for this conversation

# --- ANCHOR-SUB-PROMPT-TIPS ---
# Prompt editor modal (app/src/prompt/editor_modal.rs)
prompt-editor-title = Edit prompt
prompt-editor-warp-prompt-section = InfiniShell terminal prompt
prompt-editor-shell-prompt-section = Shell prompt (PS1)
prompt-editor-restore-default = Restore default
prompt-editor-same-line-prompt = Same line prompt
prompt-editor-separator = Separator
prompt-editor-cancel = Cancel
prompt-editor-save-changes = Save changes

# Welcome tips (app/src/tips/tip_view.rs)
welcome-tips-command-palette-title = Command Palette
welcome-tips-command-palette-description = Easily discover everything you can do in InfiniShell without your hands leaving the keyboard.
welcome-tips-split-pane-title = Split Pane
welcome-tips-split-pane-description = Split tabs into multiple panes to make your ideal layout.
welcome-tips-history-search-title = History Search
welcome-tips-history-search-description = Find, edit and re-run previously executed commands.
welcome-tips-ai-command-search-title = AI Command Search
welcome-tips-ai-command-search-description = Generate shell commands with natural language.
welcome-tips-theme-picker-title = Theme Picker
welcome-tips-theme-picker-description = Make InfiniShell your own by choosing a built-in theme. Or create your own.
welcome-tips-shortcut-label = Shortcut
welcome-tips-skip = Skip Welcome Tips
welcome-tips-complete-title = Complete!
welcome-tips-complete-description = Nice work on finishing the welcome tips!
welcome-tips-close = Close Welcome Tips

# --- ANCHOR-SUB-SMALL-DIALOGS ---
# Rewind confirmation dialog (app/src/workspace/rewind_confirmation_dialog.rs)
rewind-dialog-title = Rewind
rewind-dialog-body = Are you sure you want to rewind? This will restore your code and conversation to before this point, and cancel any commands the agent is currently running. A copy of the original conversation will be saved in your conversation history.
rewind-dialog-info = Rewinding does not affect files edited manually or via shell commands.
rewind-dialog-cancel = Cancel
rewind-dialog-confirm = Rewind

# --- ANCHOR-SUB-SEARCH-PALETTES ---
# Search palettes (app/src/search/command_palette/view.rs, app/src/search/welcome_palette/view.rs)
command-palette-search-placeholder = Search for a command
command-palette-no-results = No results found
command-palette-toast-cannot-switch-conversations = Cannot switch conversations while agent is monitoring a command.
command-palette-toast-cannot-start-new-conversation = Cannot start a new conversation while agent is monitoring a command.
command-palette-zero-state-recent = Recent
command-palette-zero-state-suggested = Suggested
welcome-palette-search-placeholder = Code, build, or search for anything…
welcome-palette-no-results = No results found
search-filter-placeholder-history = Search history
search-filter-placeholder-workflows = Search workflows
search-filter-placeholder-agent-mode-workflows = Search prompts
search-filter-placeholder-notebooks = Search notebooks
search-filter-placeholder-plans = Search plans
search-filter-placeholder-natural-language = e.g. replace string in file
search-filter-placeholder-actions = Search actions
search-filter-placeholder-sessions = Search sessions
search-filter-placeholder-conversations = Search conversations
search-filter-placeholder-historical-conversations = Search historical conversations
search-filter-placeholder-launch-configurations = Search launch configurations
search-filter-placeholder-drive = Search objects in drive
search-filter-placeholder-environment-variables = Search environment variables
search-filter-placeholder-prompt-history = Search prompt history
search-filter-placeholder-files = Search files
search-filter-placeholder-commands = Search commands
search-filter-placeholder-blocks = Search blocks
search-filter-placeholder-code = Search code symbols
search-filter-placeholder-rules = Search AI rules
search-filter-placeholder-repos = Search code repos
search-filter-placeholder-diff-sets = Search diff sets
search-filter-placeholder-static-slash-commands = Search static slash commands
search-filter-placeholder-skills = Search skills
search-filter-placeholder-base-models = Search base models
search-filter-placeholder-full-terminal-use-models = Search full terminal use models
search-filter-placeholder-current-directory-conversations = Search conversations in current directory
search-filter-display-history = history
search-filter-display-workflows = workflows
search-filter-display-agent-mode-workflows = prompts
search-filter-display-notebooks = notebooks
search-filter-display-plans = plans
search-filter-display-natural-language = AI command suggestions
search-filter-display-actions = actions
search-filter-display-sessions = sessions
search-filter-display-conversations = conversations
search-filter-display-launch-configurations = launch configurations
search-filter-display-drive = InfiniShell Drive
search-filter-display-environment-variables = environment variables
search-filter-display-prompt-history = prompt history
search-filter-display-files = files
search-filter-display-commands = commands
search-filter-display-blocks = blocks
search-filter-display-code = code
search-filter-display-rules = rules
search-filter-display-repos = repos
search-filter-display-diff-sets = diff sets
search-filter-display-static-slash-commands = slash commands
search-filter-display-historical-conversations = historical conversations
search-filter-display-skills = skills
search-filter-display-base-models = base models
search-filter-display-full-terminal-use-models = full terminal use models
search-filter-display-current-directory-conversations = current directory conversations
search-results-menu-no-results = No results found
search-results-menu-prompts-title = Prompts
ai-context-diffset-uncommitted-changes = Uncommitted changes
ai-context-diffset-changes-vs-main-branch = Changes vs. main branch
ai-context-diffset-changes-vs-branch = Changes vs. { $branch }
ai-context-diffset-uncommitted-changes-description = All uncommitted changes in the working directory
ai-context-diffset-changes-vs-main-branch-description = All changes compared to the main branch
ai-context-diffset-changes-vs-branch-description = All changes compared to { $branch }
ai-context-code-search-failed = Code search failed
ai-context-files-directory-accessibility-label = Directory: { $path }
ai-context-files-file-accessibility-label = File: { $path }
ai-context-blocks-just-now = Just now
ai-context-blocks-minutes-ago = { $count ->
        [one] 1 minute ago
       *[other] { $count } minutes ago
    }
ai-context-blocks-hours-ago = { $count ->
        [one] 1 hour ago
       *[other] { $count } hours ago
    }
ai-context-blocks-days-ago = { $count ->
        [one] 1 day ago
       *[other] { $count } days ago
    }
ai-context-blocks-no-output = No output
ai-context-blocks-accessibility-label = Block: { $command }

# --- ANCHOR-SUB-DRIVE-NAMING-IMPORT ---
# Drive naming dialog (app/src/drive/cloud_object_naming_dialog.rs)
drive-naming-notebook-name = Notebook name
drive-naming-folder-name = Folder name
drive-naming-collection-name = Collection name
drive-naming-create = Create
drive-naming-cancel = Cancel
drive-naming-rename = Rename

# Drive import modal (app/src/drive/import/modal.rs, app/src/drive/import/modal_body.rs)
drive-import-title = Import
drive-import-close = Close
drive-import-cancel = Cancel
drive-import-preparing = Preparing…
drive-import-choose-files = Choose files…
drive-import-learn-file-support = Learn about file support and formatting
drive-import-file-upload-error = Failed to upload file to server
drive-import-folder-upload-error = Failed to upload folder to server

# Drive main panel and workflow editor (app/src/drive/index.rs, app/src/drive/workflows/*)
drive-title = InfiniShell Drive
drive-environment-variables = Environment variables
drive-folder = Folder
drive-notebook = Notebook
drive-workflow = Workflow
drive-prompt = Prompt
drive-import = Import
drive-remove = Remove
drive-new-folder = New folder
drive-new-notebook = New notebook
drive-new-workflow = New workflow
drive-new-prompt = New prompt
drive-new-environment-variables = New environment variables
drive-offline-banner = You are offline. Some files will be read only.
drive-sort-by = Sort by
drive-retry-sync = Retry sync
drive-empty-trash = Empty trash
drive-trash-section-title = TRASH
drive-trash-title = Trash
drive-trash-deletion-warning = Items in the trash will be deleted forever after 30 days.
drive-team-space-zero-state = Team spaces are unavailable in local builds. Manage workflows and notebooks in Personal.
drive-sign-up-storage-limit = Local storage limits are enforced on this device.
drive-local-storage-limit-description = Local storage limits are enforced on this device. Remove unused items to create space for new InfiniShell Drive objects.
drive-sign-up = Manage locally
drive-copy-link = Copy link
drive-collapse-all = Collapse all
drive-revert-to-server = Revert to server
drive-attach-to-active-session = Attach to active session
drive-copy-prompt = Copy prompt
drive-copy-workflow-text = Copy workflow text
drive-copy-id = Copy id
drive-copy-variables = Copy variables
drive-load-in-subshell = Load in subshell
drive-delete-forever = Delete forever
drive-rename = Rename
drive-retry = Retry
drive-move-to-space = Move to { $space }
drive-open-on-desktop = Open on Desktop
drive-duplicate = Duplicate
drive-export = Export
drive-trash-menu = Trash
drive-open = Open
drive-edit = Edit
drive-restore = Restore
drive-compare-plans = Compare plans
drive-manage-billing = Manage billing
drive-object-type-notebook-plural = notebook
drive-object-type-workflow-plural = workflow
drive-object-type-folder-plural = folder
drive-object-type-env-var-collection-plural = environment variable collection
drive-object-type-object-plural = object
drive-object-type-notebooks = Notebooks
drive-object-type-workflows = Workflows
drive-object-type-environment-variables = Environment Variables
drive-object-type-folders = Folders
drive-object-type-agent-workflows = Agent Workflows
drive-object-type-ai-fact = AI Fact
drive-object-type-rules = Rules
drive-object-type-mcp-server = MCP Server
drive-object-type-mcp-servers = MCP Servers
drive-shared-object-limit-hit-banner-prefix = You've reached the local { $object_type } limit.
drive-shared-object-limit-hit-banner = You've reached the local { $object_type } limit.
drive-payment-issue-banner-prefix = Shared objects have been restricted due to a subscription payment issue.
drive-payment-issue-banner-admin = Shared objects have been restricted due to a subscription payment issue. Please update your payment information to restore access.
drive-payment-issue-banner-admin-enterprise = Shared objects have been restricted due to a subscription payment issue. Please contact support@warp.dev to restore access.
drive-payment-issue-banner-nonadmin = Shared objects have been restricted due to a subscription payment issue. Please contact a team admin to restore access.
drive-empty-trash-title = Are you sure you want to empty the trash?
drive-empty-trash-body = This action cannot be undone.
drive-empty-trash-confirm = Yes, empty trash
drive-empty-trash-cancel = Cancel
workflow-title-placeholder = Untitled workflow
workflow-description-placeholder = Add a description
workflow-title-input-placeholder = Add a title
workflow-description-input-placeholder = Add a description
workflow-new-argument = New argument
workflow-arguments-label = Arguments
workflow-argument-description-placeholder = Description
workflow-argument-value-placeholder = Value (optional)
workflow-default-value-placeholder = Default value (optional)
workflow-agent-mode-query-placeholder = Enter your prompt here… (for example, “Create a function to sort an array of objects by date” or “Help me debug this React component”).
workflow-save = Save workflow
workflow-unsaved-changes = You have unsaved changes.
workflow-keep-editing = Keep editing
workflow-discard-changes = Discard changes
workflow-ai-assist-autofill = Autofill
workflow-ai-assist-loading = Loading
workflow-ai-assist-tooltip = Generate a title, descriptions, or parameters with InfiniShell AI
workflow-tooltip-restore-from-trash = Restore workflow from trash
workflow-ai-assist-error-byop-required = Autofill requires a BYOP model. Configure a provider and model in Settings → AI.
workflow-ai-assist-error-bad-command = Failed to generate metadata. Please try again with a different command.
workflow-ai-assist-error-generic = Something went wrong. Please try again.
workflow-ai-assist-error-rate-limited = Looks like you're out of AI credits. Please try again later.
workflow-enum-new = New
workflow-alias-name-placeholder = alias name
workflow-add-argument-tooltip = Add a workflow argument

# Workspace panels (app/src/workspace/view/*)
workspace-conversation-list-search = Search
workspace-conversation-list-active = ACTIVE
workspace-conversation-list-past = PAST
workspace-conversation-list-view-all = View all
workspace-conversation-list-show-less = Show less
workspace-conversation-list-empty-title = No conversations yet
workspace-conversation-list-empty-description = Your active and past conversations with local and ambient agents will appear here.
workspace-conversation-list-new-conversation = New conversation
conversation-untitled = Untitled conversation
conversation-deleted = Deleted conversation
workspace-conversation-list-no-matching = No matching conversations
workspace-conversation-list-delete = Delete
workspace-conversation-list-delete-in-progress-error = Conversations cannot be deleted while in progress.
workspace-conversation-list-delete-ambient-tooltip = Ambient agent conversations cannot be deleted
workspace-conversation-list-fork-new-pane = Fork in new pane
workspace-conversation-list-fork-new-tab = Fork in new tab
workspace-conversation-list-fallback-title = Conversation
command-palette-conversations-active-pane = Active pane conversations
command-palette-conversations-other-active = Other active conversations
command-palette-conversations-past = Past conversations
command-palette-conversations-fork-current = Fork current conversation
command-palette-conversations-fork-current-with-title = Fork current conversation ({ $title })
command-palette-conversations-a11y-navigate = Press enter to navigate to conversation
command-palette-conversations-a11y-fork = Press enter to fork the current conversation into a new conversation.
command-palette-conversations-a11y-new = Press enter to create a new conversation.
workspace-left-panel-project-explorer = Project explorer
project-explorer-unavailable-title = Project explorer unavailable
project-explorer-unavailable-disabled-description = The Project Explorer requires access to your local workspace. Open a new session or navigate to an active session to view.
project-explorer-unavailable-remote-description = The Project Explorer requires access to your local workspace, which isn’t supported in remote sessions.
project-explorer-unavailable-wsl-description = The Project Explorer doesn't currently work in WSL.
workspace-left-panel-global-search = Global search
workspace-left-panel-warp-drive = InfiniShell Drive
workspace-left-panel-agent-conversations = Agent conversations
workspace-left-panel-ssh-manager = SSH Manager
workspace-left-panel-server-file-browser = Server files
workspace-left-panel-skill-manager = Skill Manager
skill-manager-search-placeholder = Search skills
skill-manager-filter-all = All
skill-manager-filter-provider = Source
skill-manager-meta-default = Default
skill-manager-meta-duplicate = Duplicate
skill-manager-empty = No skills match the current filters.
skill-manager-preview-empty = Select a skill to preview SKILL.md.
workspace-left-panel-projects = Projects
project-manager-new-project = New project
project-manager-default-name = New project
project-manager-empty = No projects yet
project-manager-no-hosts = No linked hosts
project-manager-start-conversation = Start agent conversation
project-manager-menu-edit = Edit project
project-view-name-label = Name
project-view-git-url-label = Git URL
project-view-git-repositories-label = Git repositories
project-view-add-repository = Add repository
project-view-remove-repository = Remove
project-view-no-repositories = No Git repositories configured
project-view-repository-servers = Mapped servers
project-view-repository-no-linked-servers = Link at least one project server below first
project-view-root-path-label = Local directory
project-view-root-path-placeholder = /home/user/projects/app
project-view-choose-folder = Choose folder
project-view-git-url-placeholder = https://github.com/example/repo.git
project-view-rules-label = Project rules
project-view-notes-label = Notes
project-view-rules-placeholder = Project-level rules and habits, injected into the agent context
project-view-notes-placeholder = Free-form notes
project-view-linked-servers = Linked servers
project-view-no-servers = No SSH servers yet — add them in the SSH manager first
project-view-save = Save
project-view-delete = Delete project
project-view-delete-confirm-title = Delete this project?
project-view-delete-confirm-description = The project will be removed from the project list. Linked SSH servers are not affected.
project-view-delete-confirm-button = Delete
project-view-status-saved = Saved
project-view-error-name-required = Project name is required
project-view-error-duplicate-git-url = Git URLs must be unique
project-view-missing = This project no longer exists.
workspace-left-panel-ssh-manager-placeholder = SSH Manager — coming soon
workspace-left-panel-ssh-manager-detail-empty = Select a server to see its details.
workspace-left-panel-ssh-manager-detail-host = Host
workspace-left-panel-ssh-manager-detail-port = Port
workspace-left-panel-ssh-manager-detail-user = User
workspace-left-panel-ssh-manager-detail-auth = Auth
workspace-left-panel-ssh-manager-detail-key-path = Key path
workspace-left-panel-ssh-manager-auth-password = Password
workspace-left-panel-ssh-manager-auth-key = Private key
workspace-left-panel-ssh-manager-auth-onekey = OneKey
workspace-left-panel-ssh-manager-onekey-credential = Credential
workspace-left-panel-ssh-manager-onekey-new = New credential
workspace-left-panel-ssh-manager-onekey-label = Credential name
workspace-left-panel-ssh-manager-onekey-user = Credential user
workspace-left-panel-ssh-manager-onekey-password = Credential password
workspace-left-panel-ssh-manager-onekey-password-required = New OneKey credentials require a password.
workspace-left-panel-ssh-manager-onekey-save-before-connect = Save the OneKey credential before connecting.
workspace-left-panel-ssh-manager-onekey-select = Select credential
workspace-left-panel-ssh-manager-onekey-select-required = Select a OneKey credential.
workspace-left-panel-ssh-manager-onekey-manage = Manage OneKey
workspace-left-panel-ssh-manager-onekey-manager-title = OneKey Manager
workspace-left-panel-ssh-manager-onekey-add = Add
workspace-left-panel-ssh-manager-onekey-delete = Delete
workspace-left-panel-ssh-manager-onekey-type = Type
workspace-left-panel-ssh-manager-onekey-type-password = Password
workspace-left-panel-ssh-manager-onekey-type-key = Private key
workspace-left-panel-ssh-manager-onekey-key-path = Key path
workspace-left-panel-ssh-manager-onekey-key-path-required = Key path is required for private key credentials.
workspace-left-panel-ssh-manager-onekey-secret = Password
workspace-left-panel-ssh-manager-onekey-save = Save
workspace-left-panel-ssh-manager-onekey-label-required = Credential name cannot be empty.
workspace-left-panel-ssh-manager-menu-new-folder = New folder
workspace-left-panel-ssh-manager-menu-new-server = New SSH server
workspace-left-panel-ssh-manager-menu-edit = Edit
workspace-left-panel-ssh-manager-menu-connect = Connect
workspace-left-panel-ssh-manager-menu-connect-split-pane = Connect in split pane
workspace-left-panel-ssh-manager-menu-sftp = File Manager
workspace-left-panel-ssh-manager-menu-clone = Clone
workspace-left-panel-ssh-manager-menu-delete = Delete
workspace-left-panel-ssh-manager-pane-hint = Editing fields and "Connect" will arrive in the next iteration. For now this pane shows the saved configuration; tweak it via the SQLite store or the upcoming editor.
workspace-left-panel-ssh-manager-pane-folder-body = Folder. Select a server inside this folder to view its details, or right-click the folder for create / delete actions.
workspace-left-panel-ssh-manager-server-missing = Server not found. It may have been deleted from another window.
workspace-left-panel-ssh-manager-field-name = Name
workspace-left-panel-ssh-manager-field-group = Group
workspace-left-panel-ssh-manager-group-root = Root
workspace-left-panel-ssh-manager-passphrase = Passphrase
workspace-left-panel-ssh-manager-save = Save
workspace-left-panel-ssh-manager-status-saved = Saved.
workspace-left-panel-ssh-manager-error-name-required = Name cannot be empty.
workspace-left-panel-ssh-manager-error-port-invalid = Port must be a number between 1 and 65535.
workspace-left-panel-ssh-manager-error-host-required = Host cannot be empty.
workspace-left-panel-ssh-manager-connect = Connect
workspace-left-panel-ssh-manager-test = Test
workspace-left-panel-ssh-manager-testing = Testing…
workspace-left-panel-ssh-manager-status-online = Online
workspace-left-panel-ssh-manager-status-offline = Offline
workspace-left-panel-ssh-manager-status-unknown = Unknown
search-filter-placeholder-ssh-servers = Search SSH servers…
search-filter-display-ssh-servers = SSH Servers
workspace-left-panel-ssh-manager-menu-rename = Rename
workspace-left-panel-ssh-manager-tree-empty = No SSH servers yet. Click 📁 to add a folder, + to add a server.
workspace-left-panel-ssh-manager-root-password = Root Password
workspace-left-panel-ssh-manager-root-password-placeholder = Password for switching to root
workspace-left-panel-ssh-manager-startup-command = Startup Command
workspace-left-panel-ssh-manager-startup-command-placeholder = Command to run after connection
workspace-left-panel-ssh-manager-notes = Notes
workspace-left-panel-ssh-manager-notes-placeholder = Notes
workspace-left-panel-ssh-manager-memory = Memory
workspace-left-panel-ssh-manager-memory-updated = Updated { $updated_at }
workspace-left-panel-ssh-manager-memory-empty = No memory recorded for this machine yet.
workspace-left-panel-ssh-manager-memory-clear = Clear memory
workspace-left-panel-ssh-manager-memory-clear-confirm-title = Clear machine memory?
workspace-left-panel-ssh-manager-memory-clear-confirm-description = This permanently deletes the AI memory stored for this machine.
workspace-left-panel-ssh-manager-memory-clear-confirm-button = Clear memory
workspace-left-panel-ssh-manager-candidates-header = From { $path }
workspace-left-panel-ssh-manager-candidates-empty = No importable hosts in { $path }
workspace-left-panel-ssh-manager-candidates-not-found = No SSH config found at { $path }
workspace-left-panel-ssh-manager-candidates-error = Could not read SSH config at { $path }: { $error }
workspace-left-panel-ssh-manager-candidates-add = Add to SSH Manager
workspace-left-panel-ssh-manager-candidates-added = Added
workspace-left-panel-ssh-manager-candidates-refresh = Refresh from ~/.ssh/config
workspace-left-panel-ssh-manager-routes-header = Saved paths
terminal-su-root-password-confirm = Auto-fill Root Password
terminal-su-root-password-confirm-subtitle = Click to confirm and inject the saved Root password
terminal-su-root-password-cancel = Cancel
server-file-browser-path-placeholder = Remote path
server-file-browser-empty = Connect to an SSH session to browse server files.
server-file-browser-no-session = No connected remote server session.
server-file-browser-connection-lost = Connection to the remote server was lost. Reconnect your SSH session, then refresh or reopen this panel.
server-file-browser-loading = Loading…
server-file-browser-empty-directory = This directory is empty.
server-file-browser-empty-response = Remote server returned an empty response.
server-file-browser-unsupported-path = This remote path type is not supported yet.
server-file-browser-copied-path = Path copied.
server-file-browser-transfer-complete = Transfer complete.
server-file-browser-modified = modified
server-file-browser-menu-refresh = Refresh
server-file-browser-menu-upload = Upload
server-file-browser-menu-new = New
server-file-browser-menu-download = Download
server-file-browser-menu-upload-file = Upload file
server-file-browser-menu-upload-folder = Upload folder
server-file-browser-menu-new-file = New file
server-file-browser-menu-new-folder = New folder
server-file-browser-menu-copy-path = Copy path
server-file-browser-menu-terminal = Terminal
server-file-browser-menu-cd-to-terminal = Run cd in terminal
server-file-browser-menu-other = More
server-file-browser-menu-copy-filename = Copy filename
server-file-browser-menu-rename = Rename
server-file-browser-menu-delete = Delete
server-file-browser-copied-name = Copied filename
server-file-browser-delete-title = Delete “{ $name }”?
server-file-browser-delete-info-file = This file will be removed from the remote host.
server-file-browser-delete-info-directory = This folder and its contents will be removed from the remote host.
server-file-browser-renamed = Renamed successfully.
server-file-browser-deleted = Deleted successfully.
server-file-browser-created-file = File created.
server-file-browser-created-folder = Folder created.
server-file-browser-default-file-name = untitled
server-file-browser-default-folder-name = untitled folder
server-file-browser-rename-empty = Name cannot be empty.
server-file-browser-rename-invalid-name = Name cannot contain “/”.
server-file-browser-rename-unchanged = Name is unchanged.
server-file-browser-operation-failed = Operation failed: { $error }
server-file-browser-rename-requires-session = Renaming requires a connected remote server.
server-file-browser-create-requires-session = Creating files requires a connected remote server.
server-file-browser-delete-requires-session = Deleting folders requires an active SSH session.
server-file-browser-upload-progress-title = Upload progress
server-file-browser-transfer-progress-title = Transfer progress
server-file-browser-transfer-progress-empty = No transfers yet.
server-file-browser-transfer-overall = { $done } / { $total } files
server-file-browser-upload-progress-empty = No uploads yet.
server-file-browser-upload-status-pending = Waiting
server-file-browser-upload-status-uploading = Uploading { $percent }%
server-file-browser-upload-status-completed = Completed
server-file-browser-upload-status-failed = Failed: { $error }
server-file-browser-download-status-pending = Waiting
server-file-browser-download-status-downloading = Downloading { $percent }%
server-file-browser-download-status-completed = Completed
server-file-browser-download-status-failed = Failed: { $error }
server-file-browser-upload-clear-completed = Clear completed
server-file-browser-upload-overall = { $done } / { $total } files
server-file-browser-upload-phase-uploading = Uploading
server-file-browser-upload-phase-verifying = Verifying
server-file-browser-upload-phase-promoting = Applying
server-file-browser-upload-status-verifying = Verifying
server-file-browser-upload-status-promoting = Applying
server-file-browser-upload-status-skipped = Skipped
server-file-browser-upload-all-skipped = All files already exist; nothing was uploaded.
server-file-browser-upload-queued = Added to the upload queue; will start after the current task finishes.
server-file-browser-upload-promote-not-replacing = Destination already exists and overwrite was not selected; upload could not be applied: { $path }
server-file-browser-upload-promote-not-replacing-generic = Destination already exists and overwrite was not selected; upload could not be applied.
server-file-browser-upload-conflict-title = Conflicting paths found
server-file-browser-upload-conflict-info = The following paths already exist at the destination. Choose how to proceed:
server-file-browser-upload-conflict-overwrite = Overwrite all
server-file-browser-upload-conflict-skip = Skip existing
server-file-browser-upload-conflict-kind-file = file
server-file-browser-upload-conflict-kind-directory = folder
server-file-browser-upload-conflict-kind-symlink = symlink
server-file-browser-upload-conflict-kind-other = other
server-file-browser-upload-conflict-more = …and { $count } more
server-file-browser-upload-verify-missing = Verification failed: missing { $path } on remote
server-file-browser-upload-verify-size = Verification failed: size mismatch for { $path }
workspace-left-panel-close-panel = Close panel
workspace-tabs-panel-tooltip = Tabs panel
workspace-tools-panel-tooltip = Tools panel
workspace-agent-management-panel-tooltip = Agent management panel
workspace-code-review-panel-tooltip = Code review panel
workspace-notifications-tooltip = Notifications
workspace-new-tab-tooltip = New Tab
workspace-tab-configs-tooltip = Tab configs
workspace-offline-tooltip = Some features may be unavailable offline
workspace-right-panel-open-repository = Open repository
workspace-right-panel-open-repository-tooltip = Navigate to a repo and initialize it for coding
workspace-right-panel-close-panel = Close panel
workspace-right-panel-code-review = Code review
workspace-right-panel-minimize = Minimize
workspace-right-panel-maximize = Maximize
terminal-pane-new-agent-conversation-title = New agent conversation
vertical-tabs-no-tabs-open = No tabs open
vertical-tabs-untitled-tab = Untitled tab
vertical-tabs-view-options-tooltip = View options
vertical-tabs-new-session = New session
vertical-tabs-terminal-kind-oz = InfiniShell Agent
vertical-tabs-pane-kind-terminal = Terminal
vertical-tabs-pane-kind-code = Code
vertical-tabs-pane-kind-code-diff = Code Diff
vertical-tabs-pane-kind-file = File
vertical-tabs-pane-kind-notebook = Notebook
vertical-tabs-pane-kind-workflow = Workflow
vertical-tabs-pane-kind-environment-variables = Environment Variables
vertical-tabs-pane-kind-environments = Environments
vertical-tabs-pane-kind-rules = Rules
vertical-tabs-pane-kind-plan = Plan
vertical-tabs-pane-kind-execution-profile = Execution Profile
vertical-tabs-pane-kind-other = Other
vertical-tabs-setting-view-as = View as
vertical-tabs-setting-panes = Panes
vertical-tabs-setting-tabs = Tabs
vertical-tabs-setting-tab-item = Tab item
vertical-tabs-setting-focused-session = Focused session
vertical-tabs-setting-summary = Summary
vertical-tabs-setting-density = Density
vertical-tabs-setting-pane-title-as = Pane title as
vertical-tabs-setting-command-conversation = Command / Conversation
vertical-tabs-setting-working-directory = Working Directory
vertical-tabs-setting-branch = Branch
vertical-tabs-setting-additional-metadata = Additional metadata
vertical-tabs-setting-show = Show
vertical-tabs-setting-pr-link-requires-gh = Requires the GitHub CLI to be installed and authenticated
vertical-tabs-setting-pr-link = PR link
vertical-tabs-setting-diff-stats = Diff stats
vertical-tabs-setting-show-details-on-hover = Show details on hover
workspace-right-panel-unknown = Unknown
global-search-placeholder = Search in files
global-search-toggle-case-sensitivity = Toggle Case Sensitivity
global-search-toggle-regex = Toggle Regex
global-search-label = Search
global-search-no-results-gitignore = No results found. Review your gitignore files.
global-search-result-count-one = 1 result in { $files } { $files ->
        [one] file
       *[other] files
    }
global-search-result-count-many = { $n } results in { $files } { $files ->
        [one] file
       *[other] files
    }
global-search-subset-warning = The result set only contains a subset of all matches. Be more specific in your search to narrow down results.
global-search-title = Global search
global-search-description = Search in files across your current directories.
global-search-unavailable-title = Global search unavailable
global-search-unavailable-description = Global search requires access to your local workspace. Open a new session or navigate to an active session to view.
global-search-remote-description = Global search requires access to your local workspace, which isn't supported in remote sessions
global-search-unsupported-session-description = Global search doesn't currently work in Git Bash or WSL.
global-search-failed = Global search failed.

# Wasm NUX dialog (app/src/wasm_nux_dialog.rs)
wasm-nux-open-desktop-title = Open in InfiniShell Desktop?
wasm-nux-open-desktop-detail = Future links will automatically open on desktop.
wasm-nux-open-desktop-confirm = Open in InfiniShell
wasm-nux-download-title = Download InfiniShell Desktop?
wasm-nux-download-description = InfiniShell is the intelligent terminal with AI and your dev team's knowledge built-in.
wasm-nux-learn-more = Learn more
wasm-nux-download-confirm = Download
wasm-nux-object-kind-drive-objects = InfiniShell Drive objects
wasm-nux-object-kind-warp-links = InfiniShell links
wasm-nux-always-open-on-web-title = Always open { $object_kind } on the web?
wasm-nux-always-open-on-web-detail = You can change this at any time in settings.
wasm-nux-yes = Yes

# Auth override warning (app/src/auth/auth_override_warning_body.rs)
auth-override-warning-title = New login detected
auth-override-warning-confirm-title = Delete personal InfiniShell Drive objects and preferences?
auth-override-warning-description = It looks like you logged into an InfiniShell account through a web browser. If you continue, any personal InfiniShell Drive objects and preferences from this anonymous session will be permanently deleted.
auth-override-warning-cannot-undo = This cannot be undone.
auth-override-warning-export = Export your data
auth-override-warning-export-description =  to import later.
auth-override-warning-cancel = Cancel
auth-override-warning-continue = Continue
auth-override-warning-accessibility-help = InfiniShell has detected a new login from a web browser. Press escape to cancel and continue using InfiniShell without login.

# Auth SSO link/login failures/paste token/logout/offline/privacy
auth-needs-sso-link-button = Link SSO
auth-needs-sso-link-title = Your organization has enabled SSO for your account
auth-needs-sso-link-detail = Click the button below to link your InfiniShell account to your SSO provider.
auth-login-failure-troubleshooting-prefix =  Not the first time? See our
auth-login-failure-troubleshooting-link =  troubleshooting docs
auth-login-failure-troubleshooting-suffix = .
auth-login-failure-invalid-token = An invalid auth token was entered into the modal.
auth-login-failure-copy-token-manually = Failed to log in. Try manually copying the auth token from the authentication web page and pasting into the modal.
auth-login-failure-login-request = Request to log in failed.
auth-login-failure-signup-request = Request to sign up failed.
auth-login-failure-wrong-redirect-url = The redirect URL pasted did not originate from this app. Please click the button below to try again.
auth-paste-token-placeholder = Enter auth token
auth-paste-token-title = Paste your auth token below
auth-paste-token-detail = Paste your auth token from the browser to get complete login.
auth-paste-token-cancel = Cancel
auth-paste-token-continue = Continue
auth-offline-first-use-description = You are currently offline. An internet connection is required to use InfiniShell for the first time.
auth-offline-first-use-learn-more = Learn more
auth-offline-overlay-title = Using InfiniShell Offline
auth-offline-overlay-paragraph-1 = InfiniShell can be used offline for local terminal and agent workflows.
auth-offline-overlay-paragraph-2 = Some setup flows may still need an internet connection when they depend on external providers.
auth-offline-overlay-paragraph-3 = Logged-out usage keeps local workflows on this machine.
auth-offline-overlay-dismiss = Dismiss
auth-privacy-settings-title = Privacy Settings
auth-privacy-settings-done = Done
auth-privacy-settings-help-improve = Help improve InfiniShell
auth-privacy-settings-help-improve-description = High-level feature usage data helps InfiniShell's product team prioritize the roadmap.
auth-privacy-settings-learn-more = Learn more
auth-privacy-settings-send-crash-reports = Send crash reports
auth-privacy-settings-crash-reports-description = Crash reporting helps InfiniShell's engineering team understand stability and improve performance.
auth-logout-confirm = Yes, log out
auth-logout-show-running-processes = Show running processes
auth-logout-cancel = Cancel
auth-logout-title = Log out?
auth-logout-running-processes-warning = You have { $count } { $count ->
        [one] process
       *[other] processes
    } running.
auth-logout-shared-sessions-warning = You have { $count } remote { $count ->
        [one] session
       *[other] sessions
    }.
auth-logout-unsynced-drive-objects-warning = You have { $count } unsynced InfiniShell Drive { $count ->
        [one] object
       *[other] objects
    }. Logging out will cause you to lose the { $count ->
        [one] object
       *[other] objects
    }.
auth-logout-unsaved-files-warning = You have { $count } unsaved { $count ->
        [one] file
       *[other] files
    }. Logging out will cause you to lose the { $count ->
        [one] file
       *[other] files
    }.

# CLI agent plugin instructions
cli-agent-plugin-run-on-remote = Be sure to run these commands on your remote machine.
cli-agent-plugin-codex-install-title = Enable InfiniShell Notifications for Codex
cli-agent-plugin-codex-install-subtitle = Update Codex to the latest version, then enable in-focus notifications so InfiniShell can display them while you work.
cli-agent-plugin-codex-update-step = Update Codex to the latest version.
cli-agent-plugin-codex-notification-step = Set the notification condition to "always" in your Codex config. Open or create ~/.codex/config.toml and add:
cli-agent-plugin-codex-restart-note = Restart Codex to apply the changes.
cli-agent-plugin-deepseek-install-title = Enable InfiniShell Notifications for DeepSeek
cli-agent-plugin-deepseek-install-subtitle = Add the following to your DeepSeek config file (~/.deepseek/config.toml) to enable turn-completion notifications.
cli-agent-plugin-deepseek-notification-step = Set the notification condition to "always" in ~/.deepseek/config.toml:
cli-agent-plugin-deepseek-restart-note = Restart DeepSeek to apply the changes.
cli-agent-plugin-claude-install-title = Install InfiniShell Plugin for Claude Code
cli-agent-plugin-claude-install-subtitle = Ensure that jq is installed on your machine. Then, run these commands.
cli-agent-plugin-claude-add-marketplace-step = Add the Warp plugin marketplace repository
cli-agent-plugin-install-warp-plugin-step = Install the Warp plugin
cli-agent-plugin-claude-restart-note = Restart Claude Code to activate the plugin.
cli-agent-plugin-claude-known-issues-note = There are some known issues with Claude Code's plugin system. If the plugin is not found after step 1, you can try manually adding an "extraKnownMarketplaces" entry to ~/.claude/settings.json.
cli-agent-plugin-claude-update-title = Update InfiniShell Plugin for Claude Code
cli-agent-plugin-run-following-commands = Run the following commands.
cli-agent-plugin-remove-existing-marketplace-step = Remove the existing marketplace (if present)
cli-agent-plugin-readd-marketplace-step = Re-add the marketplace
cli-agent-plugin-install-latest-version-step = Install the latest plugin version
cli-agent-plugin-claude-restart-update-note = Restart Claude Code to activate the update.
cli-agent-plugin-gemini-install-title = Install InfiniShell Plugin for Gemini CLI
cli-agent-plugin-gemini-run-command-restart = Run the following command, then restart Gemini CLI.
cli-agent-plugin-install-warp-extension-step = Install the Warp extension
cli-agent-plugin-gemini-restart-note = Restart Gemini CLI to activate the plugin.
cli-agent-plugin-gemini-update-title = Update InfiniShell Plugin for Gemini CLI
cli-agent-plugin-update-warp-extension-step = Update the Warp extension
cli-agent-plugin-gemini-restart-update-note = Restart Gemini CLI to activate the update.
cli-agent-plugin-opencode-install-title = Install InfiniShell Plugin for OpenCode
cli-agent-plugin-opencode-install-subtitle = Add the Warp plugin to your OpenCode configuration, then restart OpenCode.
cli-agent-plugin-opencode-open-config-step = Open or create your opencode.json. This can be in your project root, or the global config path:
cli-agent-plugin-opencode-add-plugin-step = Add "@warp-dot-dev/opencode-warp" to the "plugin" array in the top-level JSON object:
cli-agent-plugin-opencode-restart-note = Restart OpenCode to activate the plugin.
cli-agent-plugin-opencode-update-title = Update InfiniShell Plugin for OpenCode
cli-agent-plugin-opencode-update-subtitle = Pin the plugin to the latest version in your opencode.json. OpenCode caches plugins per version spec, so changing the pin forces it to re-fetch on restart.
cli-agent-plugin-opencode-replace-plugin-step = Replace the existing "@warp-dot-dev/opencode-warp" entry in the "plugin" array with the explicit version:
cli-agent-plugin-opencode-restart-update-note = Restart OpenCode to load the updated plugin.

# Remaining visible UI strings
ai-ask-user-questions-unavailable = Questions unavailable
ai-ask-user-questions-skipped-auto-approve = Questions skipped due to auto-approve
terminal-bootstrapping-checking = Checking…
terminal-bootstrapping-installing-progress = Installing… ({ $p }%)
terminal-bootstrapping-installing = Installing…
terminal-bootstrapping-updating = Updating…
terminal-bootstrapping-initializing = Initializing…
terminal-bootstrapping-installing-warp-ssh-extension-progress = Installing InfiniShell SSH extension… ({ $p }%)
terminal-bootstrapping-installing-warp-ssh-extension = Installing InfiniShell SSH extension…
terminal-bootstrapping-updating-warp-ssh-extension = Updating InfiniShell SSH extension…
terminal-bootstrapping-starting-shell-name = Starting { $shell }…
agent-tip-prefix = Tip:
agent-tip-slash-menu = `/` to open the slash-command menu and access quick agent actions.
agent-tip-toggle-input-mode = <keybinding> to toggle natural language detection and switch between agent and terminal input.
agent-tip-plan = `/plan` <prompt> to create a plan for the agent before executing.
agent-tip-command-palette = <keybinding> to open the Command Palette and access InfiniShell actions and shortcuts.
agent-tip-warp-drive = Store reusable workflows, notebooks, and prompts in your
agent-tip-redirect-running-agent = Enter a new prompt to redirect the agent while it's running.
agent-tip-add-context = `@` to add context from files, blocks, or InfiniShell Drive objects to your prompt.
agent-tip-attach-prior-output = <keybinding> to attach the prior command output as agent context.
agent-tip-init-index = `/init` to index the repo so the agent can understand your codebase.
agent-tip-agent-profiles = Add agent profiles to customize permissions and models per session.
agent-tip-fork-block = Right-click a block to fork the conversation from that point.
agent-tip-copy-output = Right-click a block to copy a conversation's output.
agent-tip-drag-image = Drag an image into the pane to attach it as agent context.
agent-tip-interactive-tools = Prompt the agent to control interactive tools like node, python, postgres, gdb, or vim.
agent-tip-code-review-panel = <keybinding> to open the code review panel and review the agent's changes.
agent-tip-add-mcp = `/add-mcp` to add an MCP server to your workspace.
agent-tip-open-mcp-servers = `/open-mcp-servers` to view and manage local MCP servers.
agent-tip-add-prompt = `/add-prompt` to create a reusable prompt for repeatable workflows.
agent-tip-add-rule = `/add-rule` to create a global agent rule.
agent-tip-fork = `/fork` to create a fresh copy of the current conversation, optionally with a new prompt.
agent-tip-open-code-review = `/open-code-review` to open the code review panel and inspect agent-generated diffs.
agent-tip-new-conversation = `/new` to start a new agent conversation with clean context.
agent-tip-compact = `/compact` to summarize the current conversation and free up space in the context window.
agent-tip-usage = `/usage` to show your current AI credits usage.
agent-tip-oz-headless = Use the `oz` command to run InfiniShell Agent in headless mode, useful for remote machines.
agent-tip-selected-text-context = Right-click selected text to attach it as agent context.
agent-tip-project-rules = Use `AGENTS.md` or `CLAUDE.md` to apply project-scoped rules.
agent-tip-url-context = Paste a URL to attach that webpage as context for the agent.
agent-tip-warpify-ssh = Warpify a remote SSH session to enable InfiniShell Agent inside that environment.
agent-tip-switch-profiles = Switch agent profiles to quickly change models and agent permissions.
agent-tip-init-rules = `/init` to generate a `WARP.md` file and define project rules for the agent.
agent-tip-auto-approve = <keybinding> to auto-approve the agent's commands and diffs for the rest of the session.
agent-tip-desktop-notifications = Enable desktop notifications to get an alert when an agent needs your attention.
agent-tip-cancel-task = <keybinding> to cancel the current agent task.
agent-tip-action-open-palette = Open palette
agent-tip-action-warp-drive = InfiniShell Drive.
agent-tip-action-show-diff-view = Show diff view
agent-tip-voice-input = Hold <keybinding> to speak your prompt directly to the agent.
hoa-welcome-banner-title = Introducing universal agent support: level up any coding agent with InfiniShell
hoa-feature-vertical-tabs-title = Vertical tabs
hoa-feature-vertical-tabs-description = Rich tab titles and metadata like git branch, worktree, and PR. Fully customizable.
hoa-feature-tab-configs-title = Tab configs
hoa-feature-tab-configs-description = Tab-level schema to set your directory, startup commands, theme, and worktree with one click
hoa-feature-agent-inbox-title = Agent inbox
hoa-feature-agent-inbox-description = Notifications when any agent needs your attention, also accessible in a central inbox
hoa-feature-native-code-review-title = Native code review
hoa-feature-native-code-review-description = Send inline comments from InfiniShell's code review directly to Claude Code, Codex, or OpenCode
resource-center-whats-new-section = What's New?
resource-center-getting-started-section = Getting Started
resource-center-maximize-warp-section = Get the most from InfiniShell
resource-center-advanced-setup-section = Advanced Setup
resource-center-create-first-block-title = Create your first block
resource-center-create-first-block-description = Run a command to see your command and output grouped.
resource-center-navigate-blocks-title = Navigate blocks
resource-center-navigate-blocks-description = Click to select a block and navigate with arrow keys.
resource-center-block-action-title = Take action on a block
resource-center-block-action-description = Right-click a block to copy, paste, share, or access more actions.
resource-center-command-palette-title = Open command palette
resource-center-command-palette-description = Access all of InfiniShell via the keyboard.
resource-center-set-theme-title = Set your theme
resource-center-set-theme-description = Make InfiniShell your own by choosing a theme.
resource-center-custom-prompt-title = Use your custom prompt
resource-center-custom-prompt-description = Set up InfiniShell to honor your PS1 setting.
resource-center-view-documentation = View documentation
resource-center-integrate-ide-title = Integrate InfiniShell with your IDE
resource-center-integrate-ide-description = Configure InfiniShell to launch from your most-used development tools.
resource-center-how-warp-uses-warp-title = How we use InfiniShell
resource-center-how-warp-uses-warp-description = Learn how the InfiniShell engineering team uses its favorite features.
resource-center-read-article = Read article
resource-center-command-search-title = Command search
resource-center-command-search-description = Find and run previously executed commands, workflows, and more.
resource-center-ai-command-search-title = AI command search
resource-center-ai-command-search-description = Generate shell commands with natural language.
resource-center-split-panes-title = Split panes
resource-center-split-panes-description = Split tabs into multiple panes to make your ideal layout.
resource-center-launch-configuration-title = Launch configuration
resource-center-launch-configuration-description = Save your current configuration of windows, tabs, and panes.
notebook-link-new-session = New session
notebook-link-new-session-tooltip = Open a new terminal session in this directory
notebook-link-open-terminal-session = Open in terminal session
notebook-link-open-in-editor = Open in editor
notebook-link-edit-markdown-file = Edit Markdown file
auth-token-placeholder = Auth Token
sharing-inherited-from-prefix = Inherited from {" "}
sharing-inherited-permission-label = Inherited permission
sharing-inherited-permissions-edit-parent-tooltip = Edit inherited permissions on the parent folder
sharing-inherited-permissions-cannot-edit-tooltip = Cannot edit inherited permissions
command-palette-navigation-running = Running…
command-palette-navigation-completed-over-hour = Completed over 1 hour ago
command-palette-navigation-completed-minute-ago = Completed { $mins } minute ago
command-palette-navigation-completed-minutes-ago = Completed { $mins } minutes ago
command-palette-navigation-no-timestamp = No timestamp found
command-palette-navigation-completed = Completed
command-palette-navigation-empty-session = Empty Session
terminal-history-tab-commands = Commands
terminal-history-tab-prompts = Prompts
common-current = Current
auth-browser-token-placeholder = Browser auth token
requested-script-expand-to-show = Expand to show script
common-hide = Hide
terminal-message-new-conversation = {" "}new conversation
agent-message-bar-again-send-to-agent = again to send to agent

# =============================================================================
# SECTION: additional-ui-surfaces
# Files: onboarding slides, auth modal, voice, launch configs, notebook file state,
#        resource center, theme picker, terminal banners, AI footer/tool output
# =============================================================================

onboarding-intention-title = Welcome to InfiniShell
onboarding-intention-subtitle = How do you want to work?
onboarding-intention-agent-title = Build faster with AI agents
onboarding-intention-agent-description = An agent-first experience with best in class terminal support. Get terminal and agent driven development AI features like:
onboarding-intention-terminal-title = Just use the terminal
onboarding-intention-terminal-badge = No AI features
onboarding-intention-terminal-description = A modern terminal optimized for speed, context, and control without AI.
onboarding-ai-feature-warp-agents = InfiniShell agents
onboarding-ai-feature-oz-cloud-agents-platform = InfiniShell local agents platform
onboarding-ai-feature-next-command-predictions = Next command predictions
onboarding-ai-feature-prompt-suggestions = Prompt suggestions
onboarding-ai-feature-remote-control-agents = Remote control with Claude Code, Codex, and other agents
onboarding-ai-feature-agents-over-ssh = Agents over SSH
onboarding-agent-title = Customize your InfiniShell Agent
onboarding-agent-subtitle = Select your in-app agent's defaults.
onboarding-agent-default-model = Default model
onboarding-agent-autonomy = Autonomy
onboarding-agent-set-by-team-workspace = Managed by local workspace policy
onboarding-agent-team-workspace-autonomy-description = Autonomy settings are configured by the local workspace policy.
onboarding-agent-autonomy-full-title = Full
onboarding-agent-autonomy-full-subtitle = Runs commands, writes code, and reads files without asking.
onboarding-agent-autonomy-partial-title = Partial
onboarding-agent-autonomy-partial-subtitle = Can plan, read files, and execute low-risk commands. Asks before making any changes or executing sensitive commands.
onboarding-agent-autonomy-none-title = None
onboarding-agent-autonomy-none-subtitle = Takes no actions without your approval.
onboarding-agent-disable-warp-agent = Disable InfiniShell Agent
onboarding-project-title = Open a project
onboarding-project-subtitle = Set up a project to optimize it for coding in InfiniShell.
onboarding-project-open-local-folder = Open local folder
onboarding-project-initialize-automatically = Initialize project automatically
onboarding-project-initialize-description = Prepares the project environment, builds an index of your code, and generates project rules—giving the agent deeper understanding and better performance.
onboarding-intro-already-have-account = Already have an account?{" "}
onboarding-intro-subtitle = A modern terminal with state of the art agents built in.
onboarding-get-started = Get started
onboarding-theme-title = Choose a theme
onboarding-theme-subtitle = Click or use arrow keys to select, Enter to confirm.
onboarding-theme-sync-with-os = Sync light/dark theme with OS
onboarding-third-party-title = Customize third party agents
onboarding-third-party-subtitle = Select defaults for using agents like Claude Code, Codex, and Gemini.
onboarding-third-party-cli-toolbar = CLI agent toolbar
onboarding-third-party-notifications = Notifications
onboarding-customize-title = Customize your InfiniShell
onboarding-customize-subtitle = Tailor your features and UI to your working style.
onboarding-customize-tab-styling = Tab styling
onboarding-customize-vertical = Vertical
onboarding-customize-horizontal = Horizontal
onboarding-customize-conversation-history = Conversation history
onboarding-customize-file-explorer = File explorer
onboarding-customize-global-file-search = Global file search
onboarding-customize-warp-drive = InfiniShell Drive
onboarding-customize-tools-panel = Tools panel
onboarding-customize-code-review = Code review

auth-opt-out-line-1 = InfiniShell stores onboarding choices locally.
auth-opt-out-line-2-prefix = You can adjust your{" "}
auth-privacy-settings-prefix = You can adjust your{" "}
auth-privacy-settings-ai-prefix = You can adjust your local AI preferences in{" "}
auth-privacy-settings = Privacy Settings
auth-local-privacy-note = InfiniShell stores onboarding choices locally on this device.
auth-terms-prefix = Continuing keeps this setup on your device.{" "}
auth-terms-of-service = Local setup
auth-log-in = Log in
auth-paste-token-from-browser = Click here to paste your token from the browser
auth-login-slide-title-warp-drive = Get started with InfiniShell Drive
auth-login-slide-title-ai = Get started with AI
auth-login-slide-subtitle-warp-drive = Connect your account to save and share notebooks, workflows, and more across devices.
auth-login-slide-subtitle-ai = Connect your account to enable AI-powered planning, coding, and automation.
auth-disable-warp-drive = Disable InfiniShell Drive
auth-disable-ai-features = Disable AI features
auth-enable-warp-drive = Enable InfiniShell Drive
auth-enable-ai-features = Enable AI features
auth-browser-sign-in-one-line-title = Sign in on your browser to continue
auth-open-page-manually-line-prefix = {" "}and open
auth-open-page-manually-line-suffix = the page manually.
auth-disable-warp-drive-confirm-title = Are you sure you want to disable InfiniShell Drive?
auth-disable-ai-features-confirm-title = Are you sure you want to disable AI features?
auth-disable-warp-drive-confirm-body = InfiniShell Drive lets you save workflows and knowledge across devices and share them with your team. By continuing, you won't have access to the following features:
auth-disable-ai-features-confirm-body = InfiniShell is better with AI. By continuing, you won't have access to any of the following features:
auth-feature-session-sharing = Session Sharing
auth-sign-up = Continue locally
auth-sign-in = Sign in
auth-already-have-account = Already have an account?{" "}
auth-dont-want-sign-in-now = Don't want to sign in right now?{" "}
auth-skip-for-now = Skip for now
auth-skip-login-confirm-title = Are you sure you want to skip login?
auth-skip-login-confirm-line-1 = You can sign up later, but some features, such as AI,
auth-skip-login-confirm-line-2-prefix = are only available to logged-in users.{" "}
auth-yes-skip-login = Yes, skip login
auth-require-login-ai-collaboration = Local AI features do not require an InfiniShell account.
auth-require-login-drive-limit = InfiniShell Drive objects are stored locally in InfiniShell.
auth-require-login-share = Sharing is unavailable in local InfiniShell builds.
auth-welcome-title = Welcome to InfiniShell!
auth-sign-up-for-warp = Continue in InfiniShell
auth-browser-sign-in-title = Sign in on your browser\nto continue
auth-open-page-manually-suffix = and open the page manually.

voice-try-input = Try Voice Input
voice-input-enabled-toast = Voice input is enabled. You can also press and hold the `{ $key }` key to activate voice input (configure in Settings > AI > Voice)
voice-input-microphone-access-error = Failed to start voice input (you may need to enable Microphone access)
voice-transcription-disabled-microphone = Voice transcription is disabled because Microphone access was not granted.
voice-transcription = Voice transcription
voice-transcription-hold-key = Voice transcription (hold `{ $key }` key)

get-started-welcome-title = Welcome to InfiniShell
get-started-subtitle = The Agentic Development Environment
theme-creator-theme-name = Theme name
theme-creator-background-color = Background color
theme-creator-image-subheader = Automatically generate a theme based on extracted colors from an image (.png, .jpg).
theme-creator-select-image = Select an image
theme-creator-selecting-image = Selecting image…
theme-creator-select-new-image = Select a new image
theme-creator-create-theme = Create theme
theme-creator-process-image-failed = Failed to process selected image. Please try again with a different image.
theme-chooser-current-description = Change your current theme.
theme-chooser-light-description = Pick a theme for when your system is in light mode.
theme-chooser-dark-description = Pick a theme for when your system is in dark mode.
theme-chooser-no-matching-themes = No matching themes!
resource-center-keyboard-shortcuts = Keyboard Shortcuts
resource-center-keybindings-essentials = Essentials
resource-center-keybindings-blocks = Blocks
resource-center-keybindings-input-editor = Input Editor
resource-center-keybindings-terminal = Terminal
resource-center-keybindings-fundamentals = Fundamentals

launch-config-save-success-prefix = Saved successfully to{" "}
launch-config-save-failure-already-exists = Failed to save. A launch configuration with the same name already exists.
launch-config-save-failure-other = An issue was encountered while saving.
launch-config-save-configuration = Save Configuration
launch-config-open-yaml-file = Open YAML File
launch-config-save-current-configuration = Save Current Configuration
launch-config-link-to-documentation = Link to Documentation
launch-config-save-modal-a11y-title = Save Config Modal
launch-config-save-modal-a11y-description = Type the name of the file to which you want to save your current configuration of windows, tabs, and panes. Use enter to save the launch configuration, esc to quit the save configuration modal.
launch-config-save-description-no-keybinding = This will save your current configuration of windows, tabs and panes to a file so you can easily open it again.
launch-config-save-description-with-keybinding = This will save your current configuration of windows, tabs and panes to a file so you can easily open it again with { $keybinding }.
launch-config-yaml-saved-to-prefix = \nThe YAML file is saved to{" "}
notebook-file-could-not-read = Could not read { $name }
notebook-file-loading = Loading { $name }…
notebook-file-missing-source = Missing source file

terminal-shared-session-reconnecting = Offline; attempting to reconnect…
terminal-banner-p10k-supported = Powerlevel10k now supports InfiniShell!{"  "}
terminal-banner-p10k-older-version-prefix = You seem to be running an older (unsupported) version, please follow{" "}
terminal-banner-these-instructions = these instructions
terminal-banner-update-latest-suffix = {" "}to update to the latest version.
terminal-banner-pure-unsupported = Pure is not yet supported in InfiniShell. You might consider one of the supported prompts as an alternative.{"  "}
terminal-loading-session = Loading session…

ai-footer-hide-rich-input = Hide Rich Input
ai-footer-choose-environment = Choose an environment
ai-footer-agent-environment = Agent environment
ai-footer-enable-terminal-command-autodetection = Enable terminal command autodetection
ai-footer-disable-terminal-command-autodetection = Disable terminal command autodetection
ai-footer-turn-off-auto-approve-agent-actions = Turn off auto-approve all agent actions
ai-footer-auto-approve-agent-actions-for-conversation = Auto-approve all agent actions for this conversation
ai-footer-approval-mode-tooltip = Approval mode for this conversation
ai-footer-approval-mode-ask-label = Ask
ai-footer-approval-mode-auto-label = Auto
ai-footer-approval-mode-full-access-label = Full Access
ai-footer-approval-mode-ask-title = Ask Before Actions
ai-footer-approval-mode-ask-description = Use your configured approval settings.
ai-footer-approval-mode-auto-description = Automatically approve actions allowed by safety rules.
ai-footer-approval-mode-full-access-description = Skip local approvals; organization and sandbox policies still apply.
ai-footer-start-remote-control = Start remote control
ai-footer-login-required-remote-control = Log in to use /remote-control
ai-footer-see-logs-for-details = See logs for details
ai-footer-plugin-installed-restart-session = Warp plugin installed. Please restart the session to activate.
ai-footer-installing-warp-plugin = Installing Warp plugin…
ai-footer-failed-install-warp-plugin = Failed to install Warp plugin
ai-footer-plugin-updated-restart-session = Warp plugin updated. Please restart the session to activate.
ai-footer-updating-warp-plugin = Updating Warp plugin…
ai-footer-failed-update-warp-plugin = Failed to update Warp plugin
voice-input-limit-reached = Voice input limit reached
voice-input-transcription-failed = Failed to transcribe voice input
ai-toolbar-context-chip = Context Chip
ai-toolbar-model-selector = Model Selector
ai-toolbar-autodetection = Autodetection
ai-toolbar-voice-input = Voice Input
ai-toolbar-attach-file = Attach File
ai-toolbar-context-usage = Context Usage
ai-toolbar-file-explorer = File Explorer
ai-toolbar-rich-input = Rich Input
ai-toolbar-approval-mode = Approval Mode
ai-tool-output-grep-for = Grep for{" "}
ai-tool-output-grepping-for = Grepping for{" "}
ai-tool-output-in-path-cancelled = {" "}in { $path } cancelled
ai-tool-output-in-path = {" "}in { $path }
ai-tool-output-grep-patterns-cancelled = Cancelled grep for the following patterns in { $path }
ai-tool-output-grep-patterns-queued = Grep for the following patterns in { $path }
ai-tool-output-grep-patterns-running = Grepping for the following patterns in { $path }
ai-tool-output-search-files-match = Search for files that match{" "}
ai-tool-output-finding-files-match = Finding files that match{" "}
ai-tool-output-file-patterns-cancelled = Cancelled search for files that match the following patterns in { $path }
ai-tool-output-file-patterns-queued = Find files that match the following patterns in { $path }
ai-tool-output-file-patterns-running = Finding files that match the following patterns in { $path }
ai-tool-output-listing-messages = Listing messages
ai-tool-output-grepping-patterns = Grepping for patterns
ai-tool-output-grepping-patterns-with-query = Grepping for patterns: { $query }
ai-tool-output-reading-messages = Reading { $count } messages

code-review-discard-uncommitted-changes-title = Discard uncommitted changes?
code-review-discard-file-uncommitted-changes-title = Discard all uncommitted changes to file?
code-review-discard-all-changes-title = Discard all changes?
code-review-discard-file-changes-title = Discard all changes to file?
code-review-discard-uncommitted-changes-description = You're about to discard all local changes that haven't been committed.
code-review-discard-file-uncommitted-changes-description = This will restore this file to the last committed version and discard local edits.
code-review-discard-all-changes-description = You're about to discard all committed and uncommitted changes.
code-review-discard-file-main-branch-description = This will restore this file to the main branch version and discard all committed and uncommitted edits.
code-review-discard-file-branch-description = This will reset this file to the { $branch } branch version and discard all committed and uncommitted edits.
code-review-stash-changes = Stash changes
code-review-no-changes-to-commit = No changes to commit
code-review-no-git-actions-available = No git actions available
command-search-out-of-credits-contact-admin = Looks like you're out of credits. Contact a team admin to upgrade for more credits.
command-search-out-of-credits-prefix = Looks like you're out of credits.{" "}
command-search-for-more-credits-suffix = {" "}for more credits.
search-not-visible-to-other-users = Not visible to other users
sharing-invite = Invite
sharing-who-has-access = Who has access
terminal-shared-session-cancel-request = Cancel request
terminal-shared-session-continue-sharing = Continue sharing
settings-import-reset-to-warp-defaults = Reset to InfiniShell defaults
settings-import-type-theme = Theme
settings-import-type-theme-with-comma = Theme,
settings-import-type-option-as-meta = Option as Meta
settings-import-type-mouse-scroll-reporting = Mouse/Scroll Reporting
settings-import-type-font = Font
settings-import-type-default-shell = Default Shell
settings-import-type-working-directory = Working Directory
settings-import-type-global-hotkey = Global hotkey
settings-import-type-window-dimensions = Window Dimensions
settings-import-type-copy-on-select = Copy On Select
settings-import-type-window-opacity = Window Opacity
settings-import-type-cursor-blinking = Cursor Blinking
settings-import-one-other-setting = 1 other setting
settings-import-other-settings = { $count } other settings
workflow-argument-editor-helper = Fill out the arguments in this workflow and copy it to run in your terminal session
workflow-add-environment-variables = Add environment variables
workflow-environment-variables = Environment variables
workflow-new-environment-variables = New environment variables
ai-history-completed-successfully = Completed successfully
ai-history-pending = Pending
ai-history-cancelled-by-user = Cancelled by user
ai-block-always-allow = Always allow
ai-cancel-summarization = Cancel summarization
ai-continue-summarization = Continue summarization
ai-dont-show-suggested-code-banners-again = Don't show me suggested code banners again
ai-inline-code-diff-no-file-name = No file name
ai-tool-call-cancelled = Tool call was cancelled
ai-batch-command-title = Run command on multiple hosts
ai-batch-command-canary-badge = Canary: abort on first host failure
ai-batch-command-run = Run
ai-batch-command-hosts-label = Hosts ({ $count })
ai-batch-command-hosts-more = +{ $count } more hosts
ai-batch-command-unknown-host = (unknown host)
ai-batch-command-timeout = Per-host timeout: { $seconds }s
ai-batch-command-args-pending = Waiting for command arguments…
ai-batch-command-streaming = Preparing batch command…
ai-batch-command-running = Running command on { $count } host(s)…
ai-batch-command-finished = Batch command finished
ai-batch-command-finished-counts = Batch command finished ({ $ok }/{ $total } hosts succeeded)
ai-batch-command-failed = Batch command failed
ai-batch-command-failed-counts = Batch command failed ({ $ok }/{ $total } hosts succeeded)
ai-batch-command-error = Batch command failed: { $error }
ai-agent-view-open-in-different-pane = Open in different pane
passive-suggestion-feature-or-bug-label = Code a feature or fix a bug in {1}
passive-suggestion-help-feature-or-bug-label = Help me code a feature or fix a bug in {1}
passive-suggestion-implement-feature-or-bug-query = Implement a feature or fix a bug in {1}. Ask me for all the details you need.
passive-suggestion-create-pull-request-query = Help me create a pull request.
passive-suggestion-start-new-project-label = Help me start a new project
passive-suggestion-start-new-project-query = Help me start a new project. Ask me for all the details you need.
passive-suggestion-node-project-label = Help me start a Node.js project
passive-suggestion-node-project-query = Help me start a Node.js project. Ask me for all the details you need.
passive-suggestion-react-app-label = Help me create a new React app
passive-suggestion-react-app-query = Help me create a new React app called {1}. Ask me for all the details you need.
passive-suggestion-next-app-label = Help me create a new Next.js app
passive-suggestion-next-app-query = Help me create a new Next.js app called {1}. Ask me for all the details you need.
passive-suggestion-rust-project-label = Help me start a Rust project for {1}
passive-suggestion-rust-project-query = Help me start a Rust project for {1}. Ask me for all the details you need.
passive-suggestion-poetry-project-label = Help me start a Poetry project for {1}
passive-suggestion-poetry-project-query = Help me start a Poetry project for {1}. Ask me for all the details you need.
passive-suggestion-django-project-label = Help me start a Django project for {1}
passive-suggestion-django-project-query = Help me start a Django project for {1}. Ask me for all the details you need.
passive-suggestion-rails-app-label = Help me start a Rails app for {1}
passive-suggestion-rails-app-query = Help me start a Rails app for {1}. Ask me for all the details you need.
passive-suggestion-gradle-maven-project-label = Help me start a Gradle/Maven project
passive-suggestion-gradle-maven-project-query = Help me start a Gradle/Maven project. Ask me for all the details you need.
passive-suggestion-go-project-label = Help me start a Go project for {1}
passive-suggestion-go-project-query = Help me start a Go project for {1}. Ask me for all the details you need.
passive-suggestion-swift-project-label = Help me start a Swift project
passive-suggestion-swift-project-query = Help me start a Swift project. Ask me for all the details you need.
passive-suggestion-terraform-config-label = Help me start a Terraform configuration
passive-suggestion-terraform-config-query = Help me start a Terraform configuration. Ask me for all the details you need.
passive-suggestion-prisma-setup-label = Help me set up Prisma in this project
passive-suggestion-prisma-setup-query = Help me set up Prisma in this project.
passive-suggestion-install-dependencies-query = Help me install dependencies for {1}.
passive-suggestion-ruby-project-label = Help me set up a new Ruby project
passive-suggestion-ruby-project-query = Help me set up a new Ruby project. Ask me for all the details you need.
passive-suggestion-modelfile-query = Help me set up a Modelfile for {1}.
passive-suggestion-kubernetes-utilization-query = Help me understand resource utilization in my cluster.
passive-suggestion-kubernetes-inspect-query = Help me inspect Kubernetes resources.
passive-suggestion-docker-containers-query = Help me manage running containers.
passive-suggestion-docker-images-query = Help me manage Docker images.
passive-suggestion-docker-compose-label = Help me manage or troubleshoot {1} with Docker Compose
passive-suggestion-docker-compose-query = Help me manage or troubleshoot {1} with Docker Compose.
passive-suggestion-docker-network-query = Help me configure containers to use {1}.
passive-suggestion-vagrant-box-query = Help me set up or customize a Vagrant box {1}.
passive-suggestion-vagrant-up-query = Help me provision my environment or troubleshoot Vagrant startup.
passive-suggestion-grep-search-query = Help me search code across files for {1}.
passive-suggestion-find-search-query = Help me search code across files with {1}.
passive-suggestion-ssh-keygen-query = Walk me through generating an SSH key.

# =============================================================================
# SECTION: late-added-ui-surfaces
# Files: app/src/workspace, app/src/terminal, app/src/code, app/src/notebooks,
#        app/src/ai, app/src/settings_view, app/src/workflows, app/src/view_components
# =============================================================================

common-update = Update
common-reject = Reject
common-open-link = Open link
common-open-file = Open file
common-open-folder = Open folder
common-name = Name
common-rule = Rule
common-skip-for-now = Skip for now
common-never = Never
common-save-changes = Save changes
common-do-not-show-again = Do not show again
common-dont-show-again-with-period = Don't show again.
common-refresh = Refresh
common-resource-not-found-or-access-denied = Resource not found or access denied
settings-search-empty-title = No settings match your search.
settings-search-empty-description = Try different keywords or check for typos.
settings-billing-monthly-spending-limit = Monthly spending limit
settings-billing-load-more = Load more
settings-billing-buy-more = Buy more
settings-billing-plan = Plan
settings-billing-manage-billing = Manage billing
settings-billing-open-admin-panel = Open admin panel
settings-billing-compare-plans = Compare plans
settings-billing-balance = Balance
settings-billing-base-credits = Base credits
settings-billing-personal-credits = Personal credits
settings-billing-team-credits = Team credits
settings-billing-workspace-credits = Workspace credits
settings-billing-cloud-agent-trial = Cloud agent trial
settings-billing-credit-remaining-one = 1 credit remaining
settings-billing-credit-remaining-many = { $credits } credits remaining
settings-billing-update-workspace-failed = Failed to update workspace settings
settings-billing-purchase-success = Successfully purchased add-on credits
settings-billing-addon-description = Add-on credits are purchased in prepaid packages that roll over each billing cycle and expire after one year. Larger purchases have a lower per-credit rate. Add-on credits are used after your base-plan credits run out.
settings-billing-addon-team-description = {" "}Purchased add-on credits are added to your personal balance.
settings-billing-auto-reload-enabled = Auto-reload is enabled
settings-billing-addon-restricted-admin = Restricted because of a billing issue. Update your payment method to purchase add-on credits.
settings-billing-addon-restricted-member = Restricted because of a billing issue. Ask a team admin to update the payment method.
settings-billing-auto-reload-failed-admin = Auto-reload is disabled after a failed reload. Update your payment method and try again.
settings-billing-auto-reload-failed-member = Auto-reload is disabled after a failed reload. Ask a team admin to update the payment method.
settings-billing-upgrade-build = Upgrade to Build
settings-billing-credit-price = { $credits } credits / { $price }
settings-billing-credit-count-one = 1 credit
settings-billing-credit-count-many = { $credits } credits
settings-billing-selected-credit-amount = selected credit amount
settings-billing-auto-reload-tooltip = When any team member's balance reaches 100 credits, automatically purchase { $amount }.
settings-billing-auto-reload-limit-admin = Auto-reload is paused because the next reload would exceed your monthly spending limit. Increase the limit to continue using auto-reload.
settings-billing-auto-reload-limit-member = Auto-reload is paused because the next reload would exceed your team's monthly spending limit. Ask a team admin to increase it.
settings-billing-purchase-limit-admin = This purchase would exceed your monthly spending limit. Increase the limit to continue.
settings-billing-purchase-limit-member = This purchase would exceed your team's monthly spending limit. Ask a team admin to increase it.
settings-billing-admin-auto-reload = Your admin enabled auto-reload for add-on credits. When your personal balance runs low, InfiniShell will purchase { $credits } credits for { $price } and add them to your balance.
settings-billing-admin-auto-reload-generic = Your admin enabled auto-reload for add-on credits. When your personal balance runs low, InfiniShell will purchase add-on credits and add them to your balance.
settings-billing-upgrade-purchase-suffix = {" "}to purchase add-on credits.
settings-billing-contact-account-executive = Contact your account executive for more add-on credits.
settings-billing-contact-team-admin = Ask a team admin to enable add-on credits.
settings-billing-buy-credits = Buy credits
settings-billing-monthly-limit-tooltip = Sets the monthly spending limit for add-on credits
settings-billing-monthly-limit = Monthly spending limit
settings-billing-purchased-this-month = Purchased this month
settings-billing-buying = Buying…
settings-billing-one-time-purchase = One-time purchase
settings-billing-auto-reload = Auto-reload
settings-billing-last-30-days = Last 30 days
settings-billing-no-usage-history = No usage history
settings-billing-no-usage-history-description = Start an agent task to see its usage history here.
settings-billing-auto-reload-pricing-unavailable = Auto-reload can't be enabled until pricing options finish loading.
settings-billing-auto-reload-toast-enabled = Auto-reload enabled. We'll add { $credits } credits when your balance runs low.
settings-billing-auto-reload-toast-disabled = Auto-reload disabled.
settings-billing-expires = Expires { $date }
settings-billing-resets = Resets { $date }
settings-billing-remaining-with-limit = / { $limit } remaining
settings-billing-remaining = remaining
settings-billing-usage = Usage
settings-billing-cost-base = Base
settings-billing-cost-addons = Add-ons
settings-billing-cost-payg = Pay-as-you-go
settings-billing-cost-cloud-only = Cloud-only
settings-billing-cost-combined = Combined
settings-billing-cost-other = Other
settings-billing-bucket-ai = AI
settings-billing-bucket-compute = Compute
settings-billing-bucket-platform = Platform
settings-billing-bucket-suggested-code-diffs = Suggested code diffs
settings-billing-bucket-voice = Voice
settings-billing-bucket-total = Total
settings-billing-total-usage = Total usage
settings-billing-source-all = All
settings-billing-source-local = Local
settings-billing-source-cloud = Cloud
settings-billing-your-usage = Your usage
settings-billing-other-members = Other members
settings-billing-automated-agent-description = This is an automated agent on your team.
settings-billing-former-member = Former member
settings-billing-members = Members
settings-billing-team = Team
settings-billing-overall-usage = Overall usage
settings-billing-local-agent-usage = Local agent usage
settings-billing-cloud-agent-usage = Cloud agent usage
settings-billing-credits-parenthetical = ({ $credits } credits)
settings-billing-limit-label = Limit: { $limit }
settings-billing-cta-open-admin = Open the admin panel
settings-billing-cta-manage-workspace-suffix = {" "}to manage workspace settings and spend limits.
settings-billing-cta-upgrade-build = Upgrade to Build
settings-billing-cta-team-usage-suffix = {" "}to see team-level credit usage.
settings-billing-cta-upgrade-business = Upgrade to Business
settings-billing-cta-user-attribution-suffix = {" "}to see per-user credit attribution.
settings-billing-cta-upgrade-enterprise = Upgrade to Enterprise
settings-billing-cta-fine-grained-suffix = {" "}to see fine-grained credit attribution and set per-user spend limits.
settings-billing-cta-spend-limits-suffix = {" "}to set per-user spend limits.
settings-billing-combined-tooltip = Other team members' usage across add-on, pay-as-you-go, and cloud-only credits.
workspace-close-session = Close session
workspace-close-session-title = Close session?
workspace-no-tabs-match-search = No tabs match your search.
workspace-orchestration-intro-title = Orchestrate any agent, anywhere
workspace-orchestration-intro-description = We've made major improvements to InfiniShell's agent orchestration platform.
workspace-orchestration-intro-cloud-title = Run any agent harness in the cloud
workspace-orchestration-intro-cloud-description = Use InfiniShell Agent to start Claude Code or Codex agents in the cloud, then track or steer their work.
workspace-orchestration-intro-multi-agent-title = Multi-agent orchestration
workspace-orchestration-intro-multi-agent-description = InfiniShell Agent can orchestrate groups of child agents so you can run tasks in parallel.
workspace-orchestration-intro-memory-title = Agent memory
workspace-orchestration-intro-memory-description = Agents can store and retrieve long-term memories, allowing them to improve over time.
workspace-orchestration-intro-research-preview = Research preview
workspace-auto-handoff-badge = Run connection lost
workspace-auto-handoff-title = Enable automatic handoff?
workspace-auto-handoff-description = Allow InfiniShell to move active local agents to the cloud automatically when your computer sleeps.
workspace-auto-handoff-enable = Enable
workspace-agent-cli-intro-title = Introducing InfiniShell TUI: your coding agent in any terminal
workspace-agent-cli-intro-anywhere-title = What's new: Use InfiniShell TUI anywhere
workspace-agent-cli-intro-anywhere-description = InfiniShell's coding agent is available in any terminal through its standalone TUI.
workspace-agent-cli-intro-multiplexer-title = What's special: Built-in terminal multiplexer
workspace-agent-cli-intro-multiplexer-description = Each InfiniShell TUI session creates its own PTY for REPLs, SSH, directory switching, and more.
workspace-auto-reload = Auto-reload
workspace-add-new-repo = {" "}+ Add new repo
workspace-notification-permission-denied-toast = InfiniShell doesn't have permission to send desktop notifications.
workspace-troubleshoot-notifications-link = Troubleshoot notifications
workspace-plan-synced-to-warp-drive-toast = Plan synced to your InfiniShell Drive
workspace-remote-control-link-copied-toast = Remote control link copied.
workspace-update-now = Update now
workspace-update-warp = Update InfiniShell
workspace-app-out-of-date-needs-update = Your app is out of date and needs to update.
workspace-restart-app-and-update-now = Restart app and update now
workspace-sampling-process-toast = Sampling process for 3 seconds…
workspace-version-deprecation-banner = Your app is out of date and some features may not work as expected. Please update immediately.
workspace-version-deprecation-without-permissions-banner = Some InfiniShell features may not work as expected without updating immediately, but InfiniShell is unable to perform the update.
workspace-new-version-unable-to-update-banner = A new version is available but InfiniShell is unable to perform the update.
workspace-unable-to-launch-new-installed-version = InfiniShell was unable to launch the new installed version.
tab-config-session-type = Session type
terminal-copy-error = Copy error
terminal-authenticate-with-github = Authenticate with GitHub
terminal-create-environment = Create an environment
terminal-regenerate-agents-file = Regenerate AGENTS.md file
terminal-view-index-status = View index status
terminal-shared-session-request-edit-access = Request edit access
terminal-create-team = Create team
terminal-warpify-without-tmux = Warpify without TMUX
terminal-continue-without-warpification = Continue without Warpify
terminal-always-install = Always install
terminal-never-install = Never install
terminal-ssh-report-issue-prefix = We are actively working on improving the stability of SSH in InfiniShell. Please consider{" "}
terminal-ssh-report-issue-link = filing an issue
terminal-ssh-report-issue-suffix = {" "}on GitHub so we can better identify the problem.
terminal-ssh-why-need-tmux = Why do I need tmux?
terminal-ssh-file-uploads-title = File Uploads
terminal-ssh-close-upload-session = Close upload session
terminal-ssh-view-upload-session = View upload session
terminal-reveal-secret = Reveal secret
terminal-hide-secret = Hide secret
terminal-copy-secret = Copy secret
terminal-tag-agent-for-assistance = Ask the agent for assistance
terminal-save-as-workflow-secrets-tooltip = Blocks containing secrets cannot be saved.
terminal-agent-mode-setup-title = Optimize InfiniShell for this codebase?
terminal-agent-mode-setup-description = Get smarter, more consistent responses by letting the agent understand your codebase and generate rules for it. You can also do this at any time by running /init.
terminal-agent-mode-setup-optimize = Optimize
terminal-no-active-conversation-to-export = No active conversation to export
terminal-slow-shell-startup-banner-prefix = Your shell seems to be taking a while to start…{"  "}
terminal-more-info = More info
terminal-show-initialization-block = Show initialization block
terminal-shell-process-exited = Shell process exited
terminal-shell-process-could-not-start = Shell process could not start!
terminal-shell-process-exited-prematurely = Shell process exited prematurely!
terminal-shell-premature-subtext = Something went wrong while starting { $shell_detail } and Warpifying it, causing the process to terminate. Warpify script output is displayed here, which may point at a cause.
terminal-file-issue = File issue
notifications-banner-troubleshoot = Troubleshoot
notifications-banner-dismissed-title = We won't show this banner again, but you can always go to Settings to enable notifications.
notifications-banner-disabled-title = Notifications were turned off, but you can always go to Settings to enable notifications.
notifications-banner-enable = Enable
notifications-banner-permissions-accepted-title = Success! You are now ready to receive desktop notifications.
notifications-banner-permissions-denied-title = InfiniShell was denied permissions to send you notifications.
notifications-banner-permissions-error-title = Something went wrong while requesting permissions.
notifications-banner-allow-permissions-title = Don't forget to 'Allow' the permissions request to finish setting up notifications.
notifications-banner-configure-notifications = Configure notifications
notifications-banner-set-permissions = Set permissions
ai-edit-api-keys = Edit API Keys
ai-block-manage-agent-permissions = Manage Agent permissions
agent-zero-state-visit-docs = Visit docs
ai-execution-profile-agent-decides = Agent decides
ai-execution-profile-always-ask = Always ask
ai-execution-profile-ask-on-first-write = Ask on first write
ai-execution-profile-never-ask = Never ask
ai-execution-profile-ask-unless-auto-approve = Ask unless auto-approve
code-accept-and-save = Accept and save
code-hunk-label = Hunk:
code-discard-this-version = Discard this version
code-overwrite = Overwrite
code-review-send-to-agent = Send to Agent
code-review-open-pr = Open PR
code-review-pr-created-toast = PR successfully created.
code-review-comments-sent-to-agent = Comments sent to agent
code-review-could-not-submit-comments = Could not submit comments to the agent
code-review-tooltip-view-changes = View changes
code-review-diffs-local-workspaces-only = Diffs only work for local workspaces.
code-review-diffs-git-repositories-only = Diffs only work for git repositories.
code-review-diffs-wsl-unsupported = Diffs don't currently work in WSL.
code-review-generating-commit-message-placeholder = Generating commit message…
code-review-type-commit-message-placeholder = Type a commit message
code-review-committing-loading = Committing…
code-review-commit-message-label = Commit message
code-review-no-non-outdated-comments-to-send = No non-outdated comments to send
code-review-send-diff-comments-to = Send diff comments to { $label }
code-review-ai-must-be-enabled-to-send-comments = AI must be enabled to send comments to Agent
code-review-agent-code-review-requires-ai-credits = Agent code review requires AI credits
code-review-all-terminals-are-busy = All terminals are busy
code-review-send-diff-comments-to-agent = Send diff comments to Agent
code-failed-to-load-file-toast = Failed to load file.
code-failed-to-save-file-toast = Failed to save file.
code-file-saved-toast = File saved.
notebook-apply-link = Apply link
notebook-sync-conflict-resolution-message = This notebook could not be saved because changes were made while you were editing. Please copy your work and refresh.
notebook-sync-feature-not-available-message = This notebook could not be saved to the server because the feature is temporarily unavailable. The changes are saved locally. Please retry later.
notebook-link-copied-toast = Link copied
settings-share-with-team = Save locally
tooltip-secrets-not-sent-to-warp-server = *Secrets are not sent to InfiniShell's server.
editor-voice-limit-hit-toast = You have hit the limit for Voice requests. Your limit will be refreshed as a part of your next cycle.
editor-voice-error-toast = An error occurred while processing your voice input.
ai-copied-branch-name-toast = Copied branch name
workflow-new-enum = New enum
workflow-edit-enum = Edit enum
workflow-enum-variant-placeholder = Variant
workflow-enum-variants = Variants
quit-warning-dont-save = Don't Save
quit-warning-show-running-processes = Show running processes
quit-warning-save-changes-title = Save changes?

# Third-pass localization: import, environment variables, workflows, notebooks, and legacy objects
settings-import-select-profile = Select a settings profile to import:
settings-import-looking-for-settings = Looking for settings to import…
settings-import-new-session-note = Some settings will take effect when you open a new session.
workspace-home-title = Welcome to InfiniShell
workspace-home-content = Welcome to InfiniShell.{"\u000A\u000A"}Use this local workspace to:{"\u000A"}* Create, view, and edit InfiniShell Drive objects{"\u000A"}* Manage local settings{"\u000A"}* Work with local agent sessions, notebooks, and workflows

env-vars-title-placeholder = Add a title
env-vars-description-placeholder = Add a description
env-vars-variable-placeholder = Variable
env-vars-value-placeholder = Value
env-vars-variable-description-placeholder = Description
env-vars-title-label = Title
env-vars-description-label = Description
env-vars-add-secret-or-command-tooltip = Add a secret or command. InfiniShell never stores external secrets.
env-vars-enterprise-secret-conflict = This environment variable cannot be created because it conflicts with your enterprise's secret redaction settings. Contact a team admin for details.
env-vars-user-secret-conflict = This environment variable cannot be created because it conflicts with your secret redaction settings. Save the secret as an environment variable in your shell configuration or a .env file, or update secret redaction in Settings → Privacy.
env-vars-invoke-error = An error occurred while trying to invoke the environment variables.
env-vars-unsaved-changes = You have unsaved changes.
env-vars-keep-editing = Keep editing
env-vars-discard-changes = Discard changes
env-vars-command-placeholder = Command
env-vars-secret-command = Secret command
env-vars-run = Run
env-vars-run-command-confirmation = Is it okay to run this command and read its output?

workflow-command-placeholder = echo "Hello {"{{"}your_name{"}}"}" # insert arguments with curly braces{"\u000A"}# enter a single-line command or an entire shell script
workflow-error-saving-aliases = Error saving aliases
workflow-error-contains-secrets = This workflow cannot be saved because it contains secrets.
workflow-error-create = Could not create workflow
workflow-prompt-copied = Prompt copied.
workflow-command-copied = Command copied.
workflow-alias-help = Aliases let you create short strings that run workflows. Each alias can use different argument values and environment variables, and aliases are personal to you.
workflow-aliases = Aliases
workflow-run-in-infinishell = Run in InfiniShell
workflow-no-longer-accessible = You no longer have access to this workflow
workflow-moved-to-trash = Workflow moved to trash
workflow-edit-prompt = Edit prompt
workflow-edit-workflow = Edit workflow
workflow-command-edited = Command edited.
workflow-cycle-parameters = to cycle parameters
workflow-save-as-workflow = Save as workflow
workflow-view-context = View context
workflow-add-alias = Add alias

notebook-block-text = Text
notebook-block-command = Command
notebook-block-bulleted-list = Bulleted list
notebook-block-numbered-list = Numbered list
notebook-block-code = Code
notebook-block-todo-list = To-do list
notebook-mermaid-raw = Raw
notebook-mermaid-rendered = Rendered
notebook-open-full-screen = Open full screen
notebook-run-in-terminal = Run in terminal
notebook-a11y-pasting = Pasting: { $text }
notebook-a11y-shift-tab = Shift+Tab
notebook-a11y-edit-link = Edit link
notebook-a11y-open-link = Open link: { $link }
notebook-a11y-secondary-click-link = Secondary click on { $link }
notebook-a11y-delete-line-left = Delete line to the left
notebook-a11y-delete-line-right = Delete line to the right
notebook-a11y-delete-word-left = Delete word to the left
notebook-a11y-delete-word-right = Delete word to the right
notebook-a11y-cut-line-left = Cut line to the left
notebook-a11y-cut-line-right = Cut line to the right
notebook-a11y-cut-word-left = Cut word to the left
notebook-a11y-cut-word-right = Cut word to the right
notebook-a11y-show-character-palette = Show character palette
notebook-a11y-show-find-bar = Show find bar
notebook-a11y-open-block-insertion-menu = Open block insertion menu
notebook-a11y-open-embedded-object-search = Open embedded object search menu
notebook-a11y-insert-block = Insert { $block } block
notebook-a11y-deselect-command = Deselect command
notebook-a11y-deselect-command-help = Switch from selecting commands to selecting text
notebook-a11y-change-code-block-language = Change code block language to { $language }
notebook-a11y-copy-code-block = Copy code block
notebook-a11y-toggle-task-list = Toggle task list
notebook-a11y-convert-to-block = Convert to { $block }
notebook-a11y-remove-link = Remove link

object-edit-access-title = This notebook is currently being edited
object-edit-access-description = If you take editing control, the current editor will be switched to view mode.
object-edit-access-confirm = Edit anyway
object-sync-failed = Failed to save
object-edited-time = Edited { $time }
object-edited-by-time = { $name } edited { $time }
object-last-edited-by = Last edited by { $name }
object-days-until-permanent-deletion = { $count ->
        [one] 1 day until permanent deletion
       *[other] { $count } days until permanent deletion
    }
object-space-personal = Personal
object-space-team = Team
object-space-shared-with-me = Shared with me

time-approx-years-ago = { $count ->
        [one] 1 year ago
       *[other] { $count } years ago
    }
time-approx-months-ago = { $count ->
        [one] 1 month ago
       *[other] { $count } months ago
    }
time-approx-weeks-ago = { $count ->
        [one] 1 week ago
       *[other] { $count } weeks ago
    }
time-approx-days-ago = { $count ->
        [one] 1 day ago
       *[other] { $count } days ago
    }
time-approx-hours-ago = { $count ->
        [one] 1 hour ago
       *[other] { $count } hours ago
    }
time-approx-minutes-ago = { $count } min ago
time-approx-minutes-ago-long = { $count ->
        [one] 1 minute ago
       *[other] { $count } minutes ago
    }
time-approx-just-now = just now
time-approx-just-now-sentence = Just now
time-elapsed-seconds = { $count ->
        [one] 1 second
       *[other] { $count } seconds
    }

settings-import-alacritty-theme-name = Imported Alacritty Theme
settings-import-iterm-theme-name = Imported iTerm Theme
settings-import-iterm-light-theme-name = Imported iTerm Theme (Light)
settings-import-iterm-dark-theme-name = Imported iTerm Theme (Dark)
settings-import-iterm-theme-name-with-suffix = Imported iTerm Theme{ $suffix }
settings-import-profile-name = Profile: { $name }
workflow-category-all = All
workflow-category-my-workflows = My Workflows
workflow-category-repository-workflows = Repository Workflows
workflow-a11y-showing-category = Showing workflows in the { $category } category
workflow-a11y-showing-all = Showing all workflows
workflow-a11y-showing-mine = Showing my workflows
workflow-a11y-showing-project = Showing repository workflows
workflow-a11y-selected = Selected { $name }: { $content }
workflow-a11y-menu-title = Workflows
workflow-a11y-menu-help = Search or use the up and down arrow keys to find a workflow. Press Enter to confirm or Escape to close.
workflow-name-with-description = { $name }: { $description }
notebook-a11y-selected-workflow = Selected workflow: { $command }
notebook-a11y-style-on = Turn { $style } on
notebook-a11y-style-off = Turn { $style } off
notebook-a11y-enable-regex-search = Enable regular expression search
notebook-a11y-disable-regex-search = Disable regular expression search
notebook-a11y-enable-case-sensitive-search = Enable case-sensitive search
notebook-a11y-disable-case-sensitive-search = Disable case-sensitive search
notebook-a11y-focus-next-match = Focus next match
notebook-a11y-focus-previous-match = Focus previous match
notebook-a11y-close-find-bar = Close find bar
notebook-a11y-notebook-title = { $title } notebook
notebook-a11y-image-title = { $title } image
notebook-editor-is-editing = { $editor } is editing
notebook-modifier-click = [{ $modifier } Click]

command-search-placeholder = Search your history, workflows, and more
external-secrets-search-placeholder = Search for a secret
notebook-reference-search-placeholder = Search for a reference
command-search-ai-translate = Translate into a shell command with InfiniShell AI
command-search-ai-ask = Ask InfiniShell AI for command suggestions
command-search-ai-accessibility-label = InfiniShell AI: { $action }
welcome-palette-add-repository = Add repository
welcome-palette-add-repository-with-shortcut = Add repository { $shortcut }
welcome-palette-terminal-session = Terminal session
welcome-palette-terminal-session-with-shortcut = Terminal session { $shortcut }
ai-context-category-files-and-folders = Files and folders
ai-context-category-commands = Commands
ai-context-category-blocks = Blocks
ai-context-category-workflows = Workflows
ai-context-category-notebooks = Notebooks
ai-context-category-plans = Plans
ai-context-category-diffs = Diffs
ai-context-category-docs = Docs
ai-context-category-past-tasks = Past tasks
ai-context-category-rules = Rules
ai-context-category-servers-and-integrations = Servers and integrations
ai-context-category-terminal = Terminal
ai-context-category-web = Web
ai-context-category-recent-diff = Most recent diff
ai-context-category-recent-block = Most recent block
ai-context-category-code = Code
ai-context-category-diff-sets = Diff sets
ai-context-category-conversations = Conversations
ai-context-category-skills = Skills

# =============================================================================
# SECTION: legacy Drive, cloud-object toasts, and workflow dialogs
# Files: app/src/drive/**, app/src/cloud_object/toast_message.rs
# =============================================================================

drive-sharing-access-can-view = Can view
drive-sharing-access-can-edit = Can edit
drive-sharing-access-full = Full access

drive-sort-last-updated = Last updated
drive-sort-last-trashed = Last trashed
drive-sort-a-to-z = A to Z
drive-sort-z-to-a = Z to A
drive-sort-type = Type

drive-item-unknown-user = unknown user
drive-item-unknown-team = unknown team
drive-item-from-owner = From { $owner }

drive-export-location-error = Could not choose an export location: { $error }
drive-export-open-in-finder = Open in Finder
drive-export-open-in-folder = Open in folder
drive-export-failed-named = Failed to export { $name }
drive-export-failed = Export failed
drive-export-completed-named = Exported { $name }
drive-export-completed-object = Exported object

workflow-enum-dynamic-placeholder = # Enter a shell command that generates variants, delimited by newlines.{"\u000A"}{"\u000A"}git branch -a

cloud-object-type-plan = Plan
cloud-object-type-rule = Rule
cloud-object-type-ai-execution-profile = AI execution profile
cloud-object-type-preference = Preference
cloud-object-type-workflow-enum = Workflow enum

cloud-object-toast-saved-to = { $object } saved to { $container }
cloud-object-toast-updated = { $object } updated
cloud-object-toast-moved-to = { $object } moved to { $container }
cloud-object-toast-trashed = { $object } trashed
cloud-object-toast-restored = { $object } restored
cloud-object-toast-left = Left { $object }
cloud-object-toast-create-failed = Failed to create { $object }
cloud-object-toast-update-failed = Failed to update { $object }
cloud-object-toast-move-failed = Failed to move { $object }
cloud-object-toast-trash-failed = Failed to trash { $object }
cloud-object-toast-restore-failed = Failed to restore { $object }
cloud-object-toast-delete-failed = Failed to delete { $object }
cloud-object-toast-leave-failed = Failed to leave { $object }
cloud-object-toast-workflow-conflict = This workflow could not be saved because changes were made while you were editing.
cloud-object-toast-env-vars-conflict = Environment variables could not be saved because changes were made while you were editing.
cloud-object-toast-rule-conflict = Rule could not be saved because changes were made while you were editing.
cloud-object-toast-start-editing-failed = Failed to start editing { $object }
cloud-object-toast-deleted-forever = { $count } { $count ->
        [one] object
       *[other] objects
    } deleted forever
cloud-object-toast-trash-emptied = Trash emptied: { $count } { $count ->
        [one] object
       *[other] objects
    } deleted forever
cloud-object-toast-empty-trash-failed = Failed to empty trash
cloud-object-toast-trash-already-empty = No objects in trash to empty

drive-import-file-picker-error = Could not select files to import: { $error }
drive-import-parse-file-error = Failed to parse file: { $error }

cloud-object-action-runs-last-day = { $count } { $count ->
        [one] run
       *[other] runs
    } in the last day
cloud-object-action-runs-last-week = { $count } { $count ->
        [one] run
       *[other] runs
    } in the last week
cloud-object-action-runs-last-month = { $count } { $count ->
        [one] run
       *[other] runs
    } in the last month
cloud-object-action-runs-last-year = { $count } { $count ->
        [one] run
       *[other] runs
    } in the last year

# =============================================================================
# SECTION: resource center, URI notifications, file pickers, and image attachments
# Files: app/src/resource_center/**, app/src/uri/mod.rs, app/src/editor/view/mod.rs
# =============================================================================

common-unknown-error = An unknown error occurred.

resource-center-footer-docs = Docs
resource-center-footer-slack = Join our Slack community
resource-center-footer-feedback = Feedback
resource-center-essentials-title = InfiniShell Essentials
resource-center-invite-friend = Invite a friend to InfiniShell
resource-center-mark-all-read = Mark all as read
resource-center-keybinding-new-window = Open New Window
resource-center-keybinding-hide-app = Hide InfiniShell
resource-center-keybinding-hide-others = Hide Others
resource-center-keybinding-quit-app = Quit InfiniShell
resource-center-keybinding-minimize = Minimize
resource-center-keybindings-toggle-panel = Toggle this panel
resource-center-keybindings-settings-description = Configure custom keybindings in Settings > Keyboard Shortcuts.
resource-center-keybindings-open-settings = Open settings
resource-center-changelog-fetch-error = Unable to fetch the latest changelog.
resource-center-read-all-changelogs = Read all changelogs
resource-center-changelog-new-features = New features
resource-center-changelog-improvements = Improvements
resource-center-changelog-bug-fixes = Bug fixes

uri-new-tab-created-title = New tab created
uri-new-tab-created-description = Open InfiniShell to view your new tab.

file-picker-select-files-error = Could not select files: { $error }
file-picker-select-file-error = Could not select a file: { $error }
file-picker-select-folder-error = Could not select a folder: { $error }
file-picker-select-image-error = Could not select an image: { $error }

editor-images-model-unsupported-tooltip = This model does not support image attachments
editor-images-disabled-query-limit = Image attachment is disabled—the limit is { $limit } per query
editor-images-disabled-conversation-limit = Image attachment is disabled—the limit is { $limit } per conversation
editor-images-attach = Attach images
editor-images-selected-model-unsupported = The selected model does not support images as context.
editor-images-not-attached-query-limit = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached—the limit is { $limit } per query.
editor-images-not-attached-conversation-limit = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached—the limit is { $limit } per conversation.
editor-image-unsupported-type = The image could not be attached. Supported types: PNG, JPG, GIF, and WEBP.
editor-images-unsupported-type = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached. Supported types: PNG, JPG, GIF, and WEBP.
editor-image-read-failed = The image could not be attached because the file could not be read.
editor-images-read-failed = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached because the { $count ->
        [one] file could not
       *[other] files could not
    } be read.
editor-image-too-large = The image could not be attached because the file is too large.
editor-images-too-large = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached because the { $count ->
        [one] file is
       *[other] files are
    } too large.
editor-image-processing-failed = The image could not be attached because processing failed.
editor-images-processing-failed = { $count } { $count ->
        [one] image wasn't
       *[other] images weren't
    } attached because processing failed.
editor-images-removed-model-unsupported = Attached images were removed because the selected model does not support images.
editor-images-removed-conversation-limit = { $count } { $count ->
        [one] image was
       *[other] images were
    } removed—the limit is { $limit } per conversation.

theme-creator-modal-title = Create a new theme from an image
theme-creator-process-image-failed-with-error = Failed to process the selected image: { $error }. Try a different image.
theme-deletion-modal-title = Delete this theme?
theme-deletion-modal-description = This will permanently delete the theme.
theme-deletion-confirm = Delete theme

# =============================================================================
# SECTION: localization-final-pass
# User-facing text discovered during the final GUI/TUI residual audit.
# =============================================================================

external-secrets-cli-not-installed = { $manager } CLI is not installed
external-secrets-view-installation-docs = View { $manager } CLI installation documentation
external-secrets-integrate-one-password = Integrate the 1Password app with its CLI
external-secrets-fetch-failed = { $manager } didn't return any secrets. It may not be configured or authenticated.
external-secrets-platform-unsupported = This platform is not supported.

terminal-execute-this-plan = Execute this plan
terminal-queued-initial-cloud-locked = The first cloud-mode prompt cannot be changed.
terminal-queued-wait-environment = Prompts cannot be sent until environment setup is complete.
terminal-queued-wait-full-terminal-agent = Prompts cannot be sent until the full terminal-use agent is initialized.
terminal-queued-send-full-terminal-agent = Send to the full terminal-use agent
terminal-queued-read-only = Read-only viewers cannot send prompts.
terminal-queued-until-command-finishes = (queued until the command finishes)
terminal-queued-count = { $count } queued

terminal-cloud-agent-start-failed = Cloud agent failed to start
terminal-continue-cloud = Continue
terminal-continue-cloud-tooltip = Continue this cloud conversation
terminal-viewing-snapshot = You're viewing a snapshot
terminal-agent-task = Agent task
terminal-snapshot-description = This shared conversation shows its state when you opened it. If the agent is still running, refresh to see the latest progress.
terminal-metadata-directory = Directory: { $directory }
terminal-metadata-skill = Skill: { $skill }
terminal-context-none-available = No objects are available in the current context.
terminal-context-ssh-unsupported = Context is not supported in SSH sessions without a remote server.
terminal-context-subshell-unsupported = Context is not supported in subshells.
terminal-context-filesystem-required = A filesystem is required.
terminal-context-disabled-terminal-mode = Context is disabled in terminal mode. Re-enable it in Settings.
terminal-attach-context = Attach context
terminal-input-mode-edit-access = Request edit access to change the input mode.
terminal-input-mode-agent-monitoring = The input mode is locked while the agent is monitoring a command.
terminal-input-mode-terminal = Terminal
terminal-input-mode-agent = Agent Mode
terminal-input-mode-shortcuts = { $keybinding } or { $prefix }

settings-scripting-title = Scripting
settings-scripting-install-success = The InfiniShell Control CLI was installed successfully. You can now run “{ $command }” from the command line.
settings-scripting-install-failed = Failed to install the InfiniShell Control command: { $error }
settings-scripting-installing = Installing…
settings-scripting-installed = Installed
settings-scripting-install = Install
settings-scripting-command-label = InfiniShell Control CLI command
settings-scripting-command-description = Install the warpctrl command to script InfiniShell from your terminal.
settings-scripting-cli-label = warpctrl CLI
settings-scripting-cli-description = warpctrl can script InfiniShell's UI. Use it with care.
settings-ai-incorrect-detection = Encountered an incorrect detection?{ " " }
settings-ai-natural-language-detection-description = Natural language detection recognizes natural-language text entered in the terminal and automatically switches to Agent Mode for AI queries.
settings-ai-incorrect-input-detection = { " " }Encountered an incorrect input detection?{ " " }
settings-ai-report-detection = Let us know

ai-summarization-cancel-title = Cancel summarization?
ai-summarization-cancel-description = Summarization is already running. If you cancel now, the request may still incur a cost, all progress will be lost, and restarting will take longer. Are you sure you want to cancel?
ai-rule-suggested-title = Suggested rule
ai-rule-offline-edit-disabled = Editing is disabled while offline.
keybinding-desc-code-review-toggle-file-navigation = Toggle file navigation in code review
keybinding-desc-code-review-send-comments = Send code review comments to the agent
ai-error-apology = I'm sorry, I couldn't complete that request.
ai-error-credit-limit = You've reached your credit limit. Your credit limit resets on { $date }.
ai-error-server-overloaded = InfiniShell is currently overloaded. Please try again later.
ai-error-internal = Internal InfiniShell error.
ai-error-invalid-api-key-title = The provided API key is not valid
ai-error-invalid-api-key-detail = Authentication with { $provider } failed while using { $model }. Double-check that your API key is correct.
ai-error-aws-credentials = AWS credentials for { $model } have expired or are missing. Refresh your AWS credentials.
ai-error-gemini-credentials = Gemini Enterprise credentials have expired or are invalid. InfiniShell couldn't authenticate with Google Cloud. Refresh your Gemini Enterprise credentials, then retry the request.
ai-error-usage-notice = This response won't count toward your usage.
ai-error-subscribe = Subscribe
ai-orchestration-parent-run-required = Remote child agents require the parent run ID to be available.
ai-orchestration-skill-resolution-failed = Failed to resolve child-agent skills: { $references }
ai-orchestration-cloud-environment-failed = Failed to start environment
ai-orchestration-github-auth-required = GitHub authentication required
ai-orchestration-github-auth-continue = Authenticate with GitHub to continue.
ai-orchestration-github-auth-rerun = Authenticate with GitHub, then run the orchestration request again.
ai-orchestration-authenticate-github = Authenticate with GitHub
workspace-free-ai-notice-title = InfiniShell no longer provides inference on the free plan.
workspace-free-ai-notice-body = To keep using InfiniShell's AI features, upgrade to a paid plan, bring your own API key or endpoint, or sign in with your Grok subscription.
workspace-free-ai-notice-bonus = If you have unused bonus credits, AI will keep working until they run out.
workspace-free-ai-suggestions-title = How to use AI features in InfiniShell
workspace-free-ai-suggestions-body = To use AI features in InfiniShell, subscribe to a paid plan, add an API key (OpenAI, Anthropic, or Google), add a custom inference endpoint (OpenRouter or LiteLLM), or sign in with your SuperGrok subscription.
workspace-free-ai-byok = Bring your own AI
workspace-free-ai-view-pricing = View pricing

feature-intro-new = NEW
feature-intro-custom-router-title = Build a custom model router for InfiniShell Agent.
feature-intro-custom-router-description = Custom routers can route tasks by their complexity or by a set of natural-language rules.
feature-intro-get-started = Get started

auth-secret-delete-title = Delete secret
auth-secret-delete-description = Are you sure you want to delete { $name }? This action cannot be undone. Any agents or environments that reference this secret will lose access to it.

ai-gemini-refresh-credentials = Refresh credentials
ai-gemini-refreshing = Refreshing…
ai-gemini-credentials-refreshed-button = Credentials refreshed
ai-gemini-try-again = Try again
ai-gemini-refreshed-title = Gemini Enterprise credentials refreshed
ai-gemini-refreshing-title = Refreshing Gemini Enterprise credentials…
ai-gemini-invalid-title = Gemini Enterprise credentials have expired or are invalid
ai-gemini-refreshed-detail = Your credentials are ready. Retry the request to continue.
ai-gemini-refreshing-detail = InfiniShell is refreshing your Google Cloud credentials.
ai-gemini-invalid-detail = InfiniShell couldn't authenticate with Google Cloud. Refresh your Gemini Enterprise credentials, then retry the request.

ai-orchestration-select-api-key = Select an API key for this harness to continue.
ai-orchestration-opencode-cloud-unsupported = OpenCode is not supported in the cloud yet. Switch to Local or choose a different harness.
ai-orchestration-parent-conversation = Parent conversation
ai-orchestration-back-to-parent = Back to parent conversation
ai-code-review-address-comments = Address these comments
ai-suggest-conversation-started = New conversation started
ai-suggest-conversation-continuing = Continuing current conversation
ai-suggest-conversation-cancelled = New conversation suggestion cancelled
ai-mcp-required-variables = Every required MCP variable must have a value.

ai-orchestration-agent-location = Agent location
ai-orchestration-location-local = Local
ai-orchestration-location-cloud = Cloud
ai-orchestration-agent-harness = Agent harness
ai-orchestration-api-key = API key
ai-orchestration-new-api-key = New API key…
ai-orchestration-host = Host
ai-orchestration-environment = Environment
ai-orchestration-runner = Runner
ai-orchestration-base-model = Base model
ai-orchestration-default-model = Default model
ai-orchestration-skip-api-key = Skip (advanced)
ai-orchestration-custom-host = Custom host…
ai-orchestration-secrets-load-failed = Unable to load secrets
ai-orchestration-disabled-by-admin = Disabled by your administrator
ai-orchestration-no-harnesses = No harnesses available
ai-orchestration-no-models = No models available
ai-orchestration-empty-environment = Empty environment
ai-orchestration-use-default = Use default
ai-orchestration-install-claude = Install Claude Code to use this local harness.
ai-orchestration-install-codex = Install Codex to use this local harness.
ai-orchestration-local-codex-disabled = Local Codex child agents are temporarily disabled.
ai-orchestration-badge-default = Default
ai-orchestration-badge-connected = Connected
ai-orchestration-badge-disconnected = Disconnected

run-agents-title = Can I start additional agents for this task?
run-agents-accept-without-orchestration = Accept without orchestration
run-agents-cancelled = Spawning agents cancelled
run-agents-configuring = Configuring agents…
run-agents-summary = Spawn { $count } { $count ->
        [one] agent
       *[other] agents
    } to address this task.
run-agents-child-agents-notice = These agents may start their own child agents
run-agents-section-label = Agents ({ $count })
run-agents-spawned = Spawned { $count } { $count ->
        [one] agent
       *[other] agents
    }
run-agents-failed = Failed to spawn { $count } { $count ->
        [one] agent
       *[other] agents
    }
run-agents-partially-spawned = Spawned { $launched } of { $total } agents
run-agents-disabled = Orchestration is currently disabled. Re-enable it on the plan card to launch agents.
run-agents-disabled-with-reason = Orchestration is currently disabled. Re-enable it on the plan card to launch agents. ({ $reason })
run-agents-start-failed = Failed to start orchestration
run-agents-start-failed-with-error = Failed to start orchestration: { $error }
run-agents-spawning = Spawning { $count } { $count ->
        [one] agent
       *[other] agents
    }…

custom-router-editor-title = Router Editor
custom-router-new = New Router
custom-router-name-placeholder-personal = { $name }'s custom router
custom-router-name-placeholder-generic = My custom router
custom-router-type-complexity = Complexity
custom-router-type-rules = Rules
custom-router-add-rule = + Add rule
custom-router-name-required = Router name is required.
custom-router-model-required = A model is required for { $field }.
custom-router-field-default = Default
custom-router-field-easy = Easy
custom-router-field-medium = Medium
custom-router-field-hard = Hard
custom-router-default-model-required = A default model is required.
custom-router-rule-required = At least one rule with a description and model is required.
custom-router-validation-error = Validation: { $error }
custom-router-serialization-error = Serialization: { $error }
custom-router-write-error = Write error: { $error }
custom-router-models = Models
custom-router-default-required = Default (required)
custom-router-easy-required = Easy (required)
custom-router-medium-required = Medium (required)
custom-router-hard-required = Hard (required)
custom-router-default-model = Default model
custom-router-rules-description = Rules are custom prompts that describe when to use a specific model. InfiniShell intelligently matches your tasks against these rules.
custom-router-rules-order = Rules are matched from top to bottom. Rules higher in the list take precedence.
custom-router-name = Router name
custom-router-complexity-based = Complexity-based
custom-router-complexity-description = { " " }routing chooses a model based on InfiniShell's classification of the task's difficulty.
custom-router-rule-based = Rule-based
custom-router-rule-description = { " " }routing chooses a model based on custom prompts.
custom-router-type = Router type
custom-router-rule = Rule
custom-router-model = Model

# Final cross-surface residual audit
common-enable = Enable
common-dont-show-again = Don't show again
common-cancelled = Cancelled
terminal-message-rewind = rewind
terminal-restore-original-directory-missing = couldn't find the original conversation directory{ " " }
terminal-restore-directory-changed = changed directory to continue the conversation{ " " }
terminal-restore-change-repositories = { " " }change repositories
terminal-history-title = History
terminal-fork-conversation = Fork conversation
terminal-current-directory = Current directory
terminal-model-base = Base
terminal-model-full-terminal-use = Full terminal use
terminal-banner-shell-exited-prematurely = Shell process exited prematurely!
terminal-banner-shell-exited-debug-description = The output from InfiniShell's initialization script is visible above to help with debugging.
terminal-banner-shell-exited = Shell process exited
terminal-banner-enable-alias-expansion = Enable alias expansion
terminal-banner-alias-expansion-title = InfiniShell can expand aliases automatically.
terminal-banner-enable-vim-keybindings = Enable InfiniShell's Vim keybindings?
terminal-banner-aws-cli-required-description = The AWS CLI is required to authenticate with your organization's AWS Bedrock. Install it to continue.
terminal-banner-aws-cli-not-installed = AWS CLI not installed
terminal-banner-log-into-aws = Log in to AWS
terminal-banner-aws-bedrock-enabled-description = AWS Bedrock support is enabled for this local workspace.
terminal-banner-use-aws-bedrock = Use AWS Bedrock?
terminal-ssh-extension-start-failed = Failed to start the SSH extension
terminal-bookmark-block-tooltip = Bookmark this block to return to it quickly
terminal-env-subshell-local-only = Environment-variable subshells are only available in local sessions.
terminal-bundled-skills-cannot-edit = Bundled skills can't be edited.
terminal-skill-editing-unsupported = Skill editing isn't supported in this build.
terminal-custom-model-cloud-unsupported = Custom models can't run in the cloud. Switch to an InfiniShell model before handing off.
terminal-conversation-navigation-failed = Couldn't navigate to the conversation.
terminal-export-file-overwritten = { $path } already exists and will be overwritten.
terminal-conversation-exported-to = Conversation exported to { $path }
terminal-command-already-running = Can't run `{ $command }` because another command is already running.
terminal-no-agent-harnesses = No agent harnesses are available. Contact your team admin.
terminal-new-conversation-monitoring-command = You can't start a new conversation while the agent is monitoring a command.
terminal-open-file-local-only = The /open-file command is only available in local sessions.
terminal-conversation-exported-clipboard = Conversation exported to the clipboard.
coding-project-create = Create new project
coding-project-create-tooltip = Create and initialize a brand-new project
coding-project-open-repository = Open repository
coding-project-open-repository-tooltip = Open an existing local folder or repository
coding-project-clone-repository = Clone repository
coding-project-clone-repository-tooltip = Clone a repository from GitHub or another source
notifications-title = Notifications
notifications-empty = No notifications
notifications-open-conversation = Open conversation
settings-billing-title = Billing and usage
settings-ai-mcp-empty-description = You haven't added any MCP servers yet. Once you do, you can control how much autonomy InfiniShell Agent has when interacting with them.{ " " }
settings-ai-mcp-empty-or = { " " }or{ " " }
settings-ai-mcp-empty-learn-more = learn more about MCP servers.
settings-ai-mcp-description = Add MCP servers to extend InfiniShell Agent's capabilities. MCP servers expose data sources or tools to agents through a standardized interface, much like plugins.{ " " }
settings-ai-file-based-mcp-description = Automatically detect and start MCP servers from globally scoped third-party AI-agent configuration files, such as files in your home directory. Servers detected inside a repository are never started automatically and must be enabled individually from the MCP settings page.{ " " }
settings-ai-file-based-mcp-supported-providers = See supported providers.
settings-ai-voice-description-prefix = Voice input lets you control InfiniShell by speaking directly to your terminal (powered by{ " " }
settings-ai-voice-description-suffix = ).
settings-ai-speech-language = Speech language
settings-ai-speech-language-description = Language used when transcribing voice input.
settings-ai-cli-toolbar-description-prefix = Show a toolbar with quick actions when running coding agents such as{ " " }
settings-ai-cli-toolbar-description-separator = ,{ " " }
settings-ai-cli-toolbar-description-last-separator = , or{ " " }
settings-ai-cli-toolbar-description-suffix = .
settings-ai-orchestration-message-display = Orchestration message display
settings-ai-orchestration-message-display-description = Controls whether orchestration messages remain expanded.
code-gutter-add-diff-context = Add diff hunk as context
code-gutter-save-to-attach-context = Save changes before attaching them as context.
code-gutter-revert-diff-hunk = Revert diff hunk
code-gutter-save-to-revert = Save changes before reverting.
code-gutter-add-line-comment = Add a comment on this line
code-gutter-save-to-add-comment = Save changes before adding a comment.
code-gutter-show-saved-comment = Show saved comment
code-review-push-tooltip = Push commits to the remote
code-review-create-pr-tooltip = Create a pull request
code-review-refreshing-pr-info = Refreshing pull request information
code-review-view-pr-github = View pull request on GitHub
code-review-publish-tooltip = Publish branch to the remote
code-review-enter-commit-message = Enter a commit message
code-review-diff-removed = Diff removed
code-review-cannot-attach-terminal-running = Context can't be attached while the terminal is running.
code-review-cannot-attach-input-unavailable = The diff can't be attached while input is unavailable.
code-cannot-save-remote-session-disconnected = Can't save because the remote session disconnected.
notebook-content-contains-secrets = This notebook can't be saved because its content contains secrets.
notebook-title-contains-secrets = This notebook can't be saved because its title contains secrets.
ambient-agent-starting-environment = Starting environment…
ambient-agent-working = Agent is working on the task
ambient-agent-failed = Agent failed
ambient-agent-authentication-required = Authentication required
ambient-agent-setting-up-environment = Setting up environment
ambient-agent-execution-host = Execution host
model-spec-context = Context
model-spec-output = Output
model-spec-cost = Cost
model-spec-intelligence = Intelligence
model-spec-speed = Speed
model-spec-inference-via-api-key = Inference via API key
toolbar-editor-title = Edit toolbar
toolbar-editor-available-items = Available items
toolbar-item-tabs-panel = Tabs panel
toolbar-item-tools-panel = Tools panel
toolbar-item-agent-management = Agent management
toolbar-item-code-review = Code review
toolbar-item-notifications = Notifications
mcp-path-required = A PATH is required to start an MCP server. Open a new terminal session to populate PATH automatically.
mcp-authentication-success = Successfully authenticated the { $server } MCP server.
mcp-credentials-rejected = The MCP server rejected the configured credentials (HTTP 401). Check the token, its expiry, and its permissions. The server did not request OAuth sign-in.
mcp-header-unresolved-secrets = The MCP header “{ $header }” contains unresolved secrets: { $secrets }. Configure these secrets and try again. No request was sent.
mcp-oauth-unavailable = The MCP server requires authentication. Enable MCP OAuth or configure a valid static credential header.
mcp-headless-oauth-required = This MCP server requires interactive OAuth, which is unavailable in headless mode. Configure a static credential header if the server supports it, or authenticate in InfiniShell on the same machine before trying again.
mcp-startup-missing-secrets = Missing MCP secrets: { $secrets }. Configure them before starting this server.
ai-code-diff-save-file-failed = Failed to save { $path }
ai-voice-enabled-toast = Voice input is enabled. You can also press and hold `{ $key }` to activate it. Configure this in Settings > AI > Voice.
workspace-crash-recovery-xwayland-description = We detected a crash during startup and changed your windowing setting to Xwayland. This may result in blurry text when using fractional scaling.
workspace-fix-with-agent = Fix with InfiniShell Agent
workspace-log-bundle-create-failed = Failed to create the log bundle: { $error }
workspace-warp-control-installed = Installed the Warp Control CLI globally. You can now run `{ $command }` from any terminal outside InfiniShell.
workspace-warp-control-install-failed = Failed to install the Warp Control command
workspace-warp-control-removed = Removed the global Warp Control CLI installation. It still works inside InfiniShell.
workspace-warp-control-uninstall-failed = Failed to uninstall the Warp Control command
workspace-workflow-unavailable = This workflow is no longer available.
workspace-check-latest-version = Check out the latest version and try again.
workspace-ai-warm-welcome-description = Ask InfiniShell AI to explain errors, suggest commands, or write scripts.
conversation-rename-empty-conversation = You can't rename an empty conversation.
conversation-rename-not-synced = Your conversation hasn't synced to the cloud yet. Send another message, then try renaming it again.
conversation-rename-in-progress = A rename is already in progress for this conversation.
conversation-rename-not-found = Conversation not found.
conversation-rename-not-ready = Your conversation is still syncing. Try renaming it again in a moment.
conversation-rename-empty-title = Enter a conversation title.
conversation-rename-too-long = Conversation titles must be { $max } characters or fewer.
cli-agent-plugin-update-not-effective = The plugin update didn't take effect.
cli-agent-platform-plugin-install-not-effective = The platform plugin installation didn't take effect.
cli-agent-platform-plugin-update-not-effective = The platform plugin update didn't take effect.
cli-agent-plugin-codex-warp-installed = Warp plugin installed. Restart Codex to activate it.
cli-agent-plugin-codex-warp-updated = Warp plugin updated. Restart Codex to activate it.
cli-agent-plugin-codex-warp-install-title = Install the Warp plugin for Codex
cli-agent-plugin-codex-run-commands-restart = Run the following commands, then restart Codex.
cli-agent-plugin-add-marketplace-step = Add the Warp plugin marketplace repository
cli-agent-plugin-codex-activate-note = Restart Codex to activate the plugin.
cli-agent-plugin-codex-warp-update-title = Update the Warp plugin for Codex
cli-agent-plugin-upgrade-marketplace-step = Upgrade the marketplace
cli-agent-plugin-reinstall-warp-plugin-step = Reinstall the Warp plugin
cli-agent-plugin-codex-activate-update-note = Restart Codex to activate the update.
cli-agent-plugin-codex-marketplace-recovery-note = If this fails because codex-warp isn't configured as a Git marketplace, remove and re-add the marketplace.
cli-agent-plugin-claude-installed = InfiniShell plugin installed. Run /reload-plugins to activate it.
cli-agent-plugin-claude-updated = InfiniShell plugin updated. Run /reload-plugins to activate it.
cli-agent-plugin-gemini-installed = InfiniShell plugin installed. Restart Gemini CLI to activate it.
cli-agent-plugin-gemini-updated = InfiniShell plugin updated. Restart Gemini CLI to activate it.
cli-agent-plugin-auto-install-unsupported = Automatic installation isn't supported for this agent.
cli-agent-plugin-auto-update-unsupported = Automatic updates aren't supported for this agent.
cli-agent-plugin-installed-restart-session = InfiniShell plugin installed. Restart the session to activate it.
cli-agent-plugin-updated-restart-session = InfiniShell plugin updated. Restart the session to activate it.
cli-agent-plugin-manager-unavailable = No plugin manager is available.
model-disabled-by-admin = This model has been disabled by your team admin.
model-upgrade-for-requests = Upgrade your plan to make more requests.
model-provider-outage = This model is temporarily unavailable because of a provider outage.
model-upgrade-to-access = Upgrade your plan to access this model.
model-unavailable = This model is unavailable.
cli-agent-generic-name = CLI agent
settings-key-side-left = { $key } (left)
settings-key-side-right = { $key } (right)
default-session-terminal = Terminal
default-session-agent = Agent
default-session-ambient-agent = Ambient agent
default-session-tab-config = Tab config
default-session-local-docker-sandbox = Local Docker sandbox
thinking-display-show-collapse-label = Show and collapse
thinking-display-always-show-label = Always show
thinking-display-never-show-label = Never show
orchestration-display-show-collapse-label = Show and collapse
orchestration-display-always-show-label = Always show
orchestration-display-always-collapse-label = Always collapse
orchestration-display-show-collapse-command = Set child-agent message display to show and collapse
orchestration-display-always-show-command = Set child-agent message display to always show
orchestration-display-always-collapse-command = Set child-agent message display to always collapse
prompt-submission-interrupt-label = Interrupt response
prompt-submission-queue-label = Queue until the response finishes
prompt-submission-interrupt-command = Set the default prompt submission mode to interrupt the response
prompt-submission-queue-command = Set the default prompt submission mode to queue until the response finishes
lrc-submission-send-immediately-label = Send immediately
lrc-submission-queue-label = Queue until the command finishes
lrc-submission-send-immediately-command = Set long-running command submission to send immediately
lrc-submission-queue-command = Set long-running command submission to queue until the command finishes
reasoning-effort-auto = Auto
reasoning-effort-off = Off
reasoning-effort-minimal = Minimal
reasoning-effort-low = Low
reasoning-effort-medium = Medium
reasoning-effort-high = High
reasoning-effort-xhigh = Extra high
reasoning-effort-max = Max
agent-task-cancelled-by-user = Cancelled by the user
agent-task-waiting-confirmation = The agent is waiting for confirmation of this action: { $action }
ai-read-file-not-found = File not found or couldn't be read
ai-read-files-failed = Failed to read files: { $files }
ai-no-output-received = No output was received.

# =============================================================================
# SECTION: final user-visible residual audit
# =============================================================================

common-show-more = Show more
common-show-less = Show less
common-allow = Allow
common-new-feature = NEW
common-view-details = View details
common-tasks = Tasks

tooltip-secret-not-included = This wasn't included in the AI conversation.
tooltip-secret-will-not-be-included = This won't be included in any AI conversations or shared blocks.
tooltip-secret-matched-organization-pattern = Pattern matched your organization's secret redaction regex list.
tooltip-secret-matched-user-pattern = Pattern matched your secret redaction regex list.
tooltip-secret-matched-pattern = Pattern matched the secret redaction regex list.

editor-search-files-and-directories = Search files and directories
editor-cycle-suggestions = Cycle suggestions

notifications-agent-completed-title = { $agent } completed
notifications-from-agent = Notification from { $agent }
notifications-task-completed = Task completed.
notifications-agent-failed-title = { $agent } failed
notifications-agent-error = The agent encountered an error.
notifications-agent-needs-attention-title = { $agent } needs attention
notifications-waiting-for-input = Waiting for input.
notifications-agent-cancelled-title = { $agent } cancelled
notifications-cancelled-by-user = Cancelled by user.
notifications-task-cancelled = Task was cancelled.
notifications-something-went-wrong = Something went wrong.

tui-mcp-no-longer-available = This MCP server is no longer available to enable.
tui-mcp-synced-template-unavailable = The synced MCP template is no longer available.
tui-mcp-gallery-template-unavailable = The gallery MCP template is no longer available.
tui-mcp-already-installed = This MCP server is already installed.
tui-mcp-install-failed = Unable to install this MCP server.
tui-mcp-select-allowed-value = Select one of the allowed values for this MCP variable.
tui-mcp-variable-once = Each MCP variable may only be provided once.
tui-mcp-start-failed = Failed to start

coding-project-build-placeholder = What do you want to build?
coding-project-suggestion-minesweeper = Build a Minesweeper clone in React
coding-project-suggestion-node-quotes = Code a Node.js server that returns random quotes from a JSON file
coding-project-suggestion-csv-json = Write a CSV-to-JSON converter CLI
coding-project-suggestion-resume = Create a starter template for a résumé web page
coding-project-suggestion-game-of-life = Make a Conway's Game of Life simulation

code-close-saved = Close saved files
code-reveal-in-finder = Reveal in Finder
code-reveal-in-explorer = Reveal in Explorer
code-reveal-in-file-manager = Reveal in file manager
code-comment-imported-from-github = Comment imported from GitHub
code-saved-changes-not-reflected = This file has saved changes that are not reflected here.
code-remote-host-disconnected = The remote host disconnected. You won't be able to see updates or save changes.

context-node-install-nvm-title = Install nvm to enable version switching
context-node-install-nvm-description = This menu helps you switch between Node.js versions, but it requires nvm to be installed.
context-node-no-versions = No Node.js versions installed
context-node-try-installing-versions = Try installing versions with nvm

code-review-error-loading-diffs = Error loading diffs
code-review-cannot-detect-diffs = Can't detect diffs for this folder
code-review-no-changes-description = As you or the Agent make changes, you'll be able to track them here.
code-review-repo-initialized-with-file = Repository initialized with a { $file } file.
code-review-binary-no-diff = Binary file — no diff available
code-review-file-renamed-without-changes = File renamed without changes
code-review-new-empty-file = New empty file
code-review-file-content-unavailable = Unable to load file content
code-review-no-file-selected = No file selected
code-review-no-files-to-discard = No files to discard
code-review-outdated-count = { $count ->
    [one] 1 outdated
   *[other] { $count } outdated
}
code-review-outdated = Outdated
code-review-from-github = From GitHub
code-review-branch = Branch
code-review-default-branch = default branch
code-review-changes = Changes
code-review-included-commits = Included commits
code-review-include-unstaged = Include unstaged changes

tab-config-auto-create-worktree = Automatically create a worktree when opening a new tab
tab-config-auto-generate-worktree-branch = Automatically generate the worktree branch name

ai-block-queued = Queued
ai-block-refine = Refine
ai-block-take-over = Take over
ai-block-take-control = Take control
ai-debug-information = Debug information: { $info }
ai-debug-output = Debug output
ai-question-skip-all = Skip all
ai-command-profile-always-asks-permission = Your profile is set to always ask for permission to execute commands.
ai-usage-other-context-description = Includes other request context and temporary instructions added to help the agent respond better.
ai-full-terminal-agent-default-model = Now using Full Terminal Agent's default model.
ai-aws-running-login-command = Running `{ $command }`…
ai-aws-credentials-expired-title = AWS credentials expired or missing
ai-aws-authentication-failed-description = Authentication with AWS Bedrock failed while using { $model }. Run `{ $command }` to refresh your credentials.
ai-aws-always-run-automatically = Always run automatically

ambient-agent-environment-start-failed = Failed to start environment
ambient-agent-github-auth-required = GitHub authentication required
ambient-agent-github-auth-description = Authenticate with GitHub to continue
ambient-agent-run-cancelled = Agent run cancelled
ambient-agent-no-environment-started = No environment was started

terminal-secrets-skip-advanced = Skip (advanced)
terminal-secrets-skip-advanced-description = Only if your key is already set in the environment (for example, injected as a Kubernetes secret)
terminal-secrets-none-found-helper = No secrets found. Save to use this value directly, or select the key to add a secret.

hoa-switch-horizontal-tabs = Switch back to horizontal tabs
hoa-vertical-tabs-callout-title = Introducing vertical tabs — the new default
hoa-vertical-tabs-callout-description = Vertical tabs show all open agent and terminal panes, grouped by tab. Customize the information shown to support your workflow.
hoa-agent-inbox-callout-title = Meet your new agent inbox
hoa-agent-inbox-callout-description = InfiniShell routes notifications from every CLI coding agent into one notification center that works across agents and harnesses.{ " " }
hoa-create-first-tab-config = Create your first tab config
hoa-create-first-tab-config-description = Set up a reusable starting point for your tabs. Pick a repository, choose a session type, and optionally attach a worktree. Reuse it whenever you open a tab with this setup.

terminal-clipboard-access-blocked = A terminal program tried to access your clipboard. This is disabled by default for security.
terminal-notification-discovery-long-command = InfiniShell can notify you when long-running commands finish.
terminal-notification-discovery-agent = InfiniShell can notify you when an agent finishes responding.
terminal-notification-discovery-attention = InfiniShell can notify you when a command or agent needs your attention.
terminal-notification-discovery-password = InfiniShell can notify you when you're prompted to enter a password.
terminal-notification-command-finished-after = { " " }finished after { $duration }s
terminal-notification-command-failed-after = { " " }failed after { $duration }s
terminal-notification-agent-finished-suffix = { " " }finished
terminal-notification-agent-failed-suffix = { " " }failed
terminal-notification-blocked-suffix = { " " }blocked
terminal-notification-waiting-password-suffix = { " " }is waiting for a password
terminal-notification-latest-output-prefix = Latest output:{ " " }
terminal-notification-error-prefix = Error:{ " " }
terminal-warpify-a11y-with-keybinding = Press { $keybinding } to Warpify this { $subject } and enable more InfiniShell features.
terminal-warpify-a11y = Warpify this { $subject } to enable more InfiniShell features.
terminal-warpify-recognized = { $subject } recognized.
terminal-notification-enable-command-palette = You can enable notifications from the command palette.
terminal-notification-permission-error = InfiniShell tried to send a notification for the last block but doesn't have permission.
terminal-notification-send-error = InfiniShell tried to send a notification for the last block, but something went wrong.
terminal-notification-send-error-short = Error sending notification
terminal-notification-check-system-settings = Make sure InfiniShell is allowed to send notifications in System Settings.
terminal-correction-suggested-command = Suggested corrected command: { $command }
terminal-correction-a11y-help = Press Right Arrow to insert it, or keep editing to ignore it.
terminal-command-waiting-password = Command is waiting for a password
terminal-cloud-task-continue-failed = Couldn't continue this cloud task.
terminal-clipboard-write-blocked = A terminal program tried to write to your clipboard. This is disabled by default to protect you from malicious software.
terminal-clipboard-read-blocked = A terminal program tried to read your clipboard. This is disabled by default to protect you from malicious software.
terminal-clipboard-allow-writes = Allow clipboard writes
terminal-clipboard-allow-read-write = Allow clipboard reads and writes
terminal-lock-scrolling-at-block-bottom = Lock scrolling at the bottom of the block
terminal-jump-to-block-bottom = Jump to the bottom of this block
terminal-block-a11y-failed-status = failed, status code { $code }
terminal-block-a11y-background = running in the background
terminal-block-a11y-succeeded = succeeded
terminal-block-a11y-in-progress = in progress
terminal-block-a11y-summary = Block { $index }: { $command }, { $status }.
terminal-block-a11y-help = Press Command-C to read and copy the command and output, or Command-Option-Shift-C to read and copy only the output. Press Command-B to bookmark the block; use Option-Up and Option-Down to move between bookmarks.
terminal-subshell-title = Subshell
terminal-subshell-subject = subshell
terminal-explain-following = Explain the following:
    { $selection }
terminal-what-happened-here = What happened here?
terminal-command-for-query = What is the command to: { $query }

time-precise-over-one-week = >1 week
time-precise-days = { $value } days
time-precise-hours = { $value } hours
time-precise-minutes = { $value } min
time-precise-seconds = { $value } sec
time-precise-milliseconds = { $value } ms
terminal-history-exit-code = Exit code { $code }
terminal-history-finished-in = Finished in { $duration }
terminal-history-last-ran = Last ran { $time }
terminal-history-ran = Ran { $time }
code-requested-edit = Requested edit
code-review-file-count = { $count ->
    [one] 1 file
   *[other] { $count } files
}

# Final settings, context, and terminal surface labels
settings-voice-input-hold-key = Voice input (hold the { $key } key)
settings-language-simplified-chinese = Simplified Chinese
settings-ctrl-tab-previous-next = Activate previous/next tab
settings-ctrl-tab-recent-session = Cycle most recent session
settings-ctrl-tab-recent-tab = Cycle most recent tab
settings-hotkey-dedicated-window = Dedicated hotkey window
settings-hotkey-toggle-all-windows = Show/hide all windows
settings-cursor-bar = Bar
settings-cursor-block = Block
settings-cursor-underline = Underline
settings-line-numbers-absolute = Absolute
settings-line-numbers-relative = Relative
settings-clipboard-write-only = Write only
settings-clipboard-read-write = Read and write
settings-secrets-asterisks = Asterisks
settings-secrets-strikethrough = Strikethrough
settings-secrets-always-show = Always show secrets
notifications-filter-all-tabs = All tabs
notifications-filter-unread = Unread
notifications-filter-errors = Errors
tab-config-built-in-agent = Built-in agent
code-review-commit-your-changes = Commit your changes
code-review-publish-branch = Publish branch
code-review-push-changes = Push changes
code-review-create-pull-request = Create pull request
context-search-directories = Search directories…
context-search-branches = Search branches…
context-search-environments = Search environments…
code-find-a11y-description = Find bar for searching text in the editor.
code-find-a11y-match-description = Find bar with { $count } matches. Currently on match { $current } of { $count }.
code-find-a11y-replace-help = Replace field focused. Type replacement text, press Enter to replace the current match, or Tab to return to the find field. Use the up and down arrows to navigate matches, or Escape to close.
code-find-a11y-find-help = Find field focused. Type to search. Use Enter and Shift-Enter or the up and down arrows to navigate matches. Press Escape to close the find bar.
terminal-section-commands = Commands
terminal-section-skills = Skills
terminal-section-prompts = Prompts
terminal-section-workflows = Workflows
terminal-menu-commands = /Commands
terminal-menu-model = /Models
terminal-menu-conversations = /Conversations
terminal-menu-profiles = /Profiles
terminal-menu-prompts = /Prompts
terminal-menu-skills = /Skills
terminal-menu-fork = /Fork
terminal-menu-rewind = /Rewind
terminal-menu-history = History
terminal-menu-repositories = /Repositories
terminal-menu-plans = /Plans
terminal-file-uploading = Uploading…
terminal-file-uploaded = Uploaded
terminal-file-upload-failed = Upload failed
terminal-open-file-banner-markdown-description = Did you know that InfiniShell can display Markdown files directly?
terminal-open-file-banner-language-description = Did you know that InfiniShell can edit { $language } files directly?
terminal-open-file-banner-code-description = Did you know that InfiniShell can edit code directly?
terminal-open-file-banner-view = View in InfiniShell
terminal-open-file-banner-edit = Edit in InfiniShell
terminal-prompt-suggestion-explain = Explain this to me.
terminal-prompt-suggestion-fix = Help me fix this.
terminal-prompt-suggestion-install = Help me install a binary or dependency. What information do you need from me?
terminal-prompt-suggestion-code = Help me write some code. What information do you need from me?
terminal-prompt-suggestion-deploy = Help me deploy my project. What information do you need from me?
terminal-prompt-suggestion-other = Something else?
custom-router-complexity-routing = Complexity routing
custom-router-prompt-routing = Prompt routing
model-inference-user-api-key = Your API key
model-inference-team-api-key = Team API key
context-requires-local-session = Requires a local session
context-requires-github-cli = Requires the GitHub CLI
context-requires-command = Requires the `{ $command }` command
context-git-tracking-rebased = Tracking { $upstream } • branch was rebased
context-git-tracking-counts = Tracking { $upstream } • ahead { $ahead }, behind { $behind }
context-git-tracking-counts-unavailable = Tracking { $upstream }; ahead/behind counts are unavailable
context-git-rebased-no-upstream = Branch was rebased; upstream name is unavailable
context-git-counts-no-upstream = Ahead { $ahead }, behind { $behind }; upstream name is unavailable
context-git-no-upstream = No upstream configured
terminal-shell-system-default = System default shell
terminal-shell-wsl = Windows Subsystem for Linux
terminal-shell-custom-path = Custom: { $path }
terminal-shell-docker-sandbox = Docker Sandbox
terminal-shell-custom-command = Custom ({ $command })
voltron-ai-command-search = AI command search
voltron-history-search = History search
tui-mcp-source-cli-local = Local CLI
tui-mcp-source-another-device = Synced from another device
tui-mcp-source-shared-by = Shared by { $creator }
tui-mcp-source-shared-template = Shared template
tui-mcp-source-shared-by-warp = Shared by InfiniShell
tui-mcp-source-provider-global = { $provider } global configuration
tui-mcp-source-project = Project
tui-mcp-source-provider-project = { $provider } project configuration ({ $project })
tui-mcp-source-file-config = File-based configuration
settings-tui-statusline-auto-approve = Auto-approve indicator
settings-tui-statusline-vim-mode = Vim mode indicator
settings-tui-statusline-model = Model
settings-tui-statusline-working-directory = Working directory
settings-tui-statusline-git-branch = Git branch
settings-tui-statusline-git-branch-status = Git branch status
settings-tui-statusline-git-diff-status = Git diff status
settings-tui-statusline-github-pull-request = GitHub pull request
settings-tui-statusline-credit-usage = Credit usage
settings-tui-statusline-context-window-usage = Context window usage
settings-tui-statusline-date = Date
settings-tui-statusline-time-12-hour = Time (12-hour)
settings-tui-statusline-time-24-hour = Time (24-hour)
settings-tui-statusline-agent-todo-list = Agent to-do list
settings-tui-statusline-voice-input = Voice input
model-spec-inference-via-bedrock = Inference via Bedrock
model-spec-inference-via-gemini-enterprise = Inference via Gemini Enterprise Agent Platform
model-spec-inference-may-use-hosted = Inference may use your hosted provider
settings-responses-state-local-zdr = Local / ZDR
settings-responses-state-provider-chain = Provider chain
settings-responses-state-cloud-conversation = Cloud conversation
workspace-home-directory-unavailable = Failed to determine home directory
code-new-file-suffix = { " " }(new)
notebook-other-user = Other user
ai-aws-default-profile-capitalized = The default AWS profile
ai-aws-default-profile = the default AWS profile
ai-aws-named-profile-capitalized = The AWS profile `{ $profile }`
ai-aws-named-profile = the AWS profile `{ $profile }`
ai-aws-credentials-not-found = AWS credentials were not found for { $profile }. Log in with the AWS CLI or update your AWS credentials configuration, then refresh.
ai-aws-credentials-timeout = Timed out while loading AWS credentials. Refresh and try again.
ai-aws-credentials-invalid = { $profile } is invalid or incomplete in your local AWS configuration. Update your AWS profile settings and credentials, then refresh.
ai-aws-credentials-provider-error = Unable to load AWS credentials from your configured provider. Refresh your AWS login and try again.
ai-aws-credentials-unexpected-error = Unexpected error while loading AWS credentials. Refresh your AWS login and try again.
ai-aws-credentials-load-error = Unable to load AWS credentials. Refresh your AWS login and try again.
ai-aws-not-configured = No AWS credentials are configured
ai-aws-load-failed-with-message = Failed to load AWS credentials: { $message }
ambient-agent-source-scheduled = Scheduled
ambient-agent-source-local = InfiniShell (local agent)
ai-mcp-other-agents = Other agents
custom-router-routes-by-complexity = Routes by task complexity
custom-router-routes-by-prompt = Routes by prompt content
ai-fallback-primary-failed-named = The primary model ({ $primary }) failed. Retrying with the fallback model.
ai-fallback-primary-failed = The primary model failed. Retrying with the fallback model.
ai-fallback-warping-with-model = Warping with { $model }.
ai-fallback-warping-with-another-model = Warping with another model.
terminal-remote-host = Remote host
ai-orchestration-participant-orchestrator = Orchestrator
ai-orchestration-participant-unknown = Unknown agent
ai-orchestration-participant-agent = Agent
code-find-replaced-match-a11y = Successfully replaced the match. The selected match is { $current } of { $count }.
code-find-replaced-match-help = Continue pressing Enter to replace more matches, or use the up and down arrows to navigate.
code-find-replaced-last-match-a11y = Successfully replaced the last match.
ai-document-untitled-filename = Untitled.md
settings-mcp-debug-template-sync-id = Template sync ID: { $id }
settings-mcp-debug-gallery-id = Gallery ID: { $id }
settings-mcp-debug-gallery-id-none = Gallery ID: None
settings-mcp-debug-template-not-found = Could not find the template object
context-branch-worktree-path-unavailable = The branch is already checked out in another worktree, but InfiniShell couldn't find its path.
tab-config-new-worktree-branch-name = New worktree branch name
auth-secret-placeholder-bearer-token = Bearer token
auth-secret-placeholder-secret-access-key = Secret access key
auth-secret-placeholder-session-token = Session token (temporary credentials only)
auth-secret-openai-api-key = OpenAI API key
auth-secret-anthropic-api-key = Anthropic API key
auth-secret-bedrock-api-key = Bedrock API key
auth-secret-bedrock-access-key = Bedrock access key
editor-a11y-selected = selected
editor-a11y-unselected = unselected
editor-a11y-selection-state = , { $state }
find-result-a11y = Result { $current } of { $count }.
find-result-help-a11y = Use Enter and Shift-Enter to navigate between matches. Press Escape to quit.
ai-status-waiting-for-instructions = Agent waiting for instructions…
ai-status-warping = Warping…
ai-status-adjusting-tasks = Adjusting tasks…
ai-status-generating-fix = Generating fix…
ai-status-creating-diff = Creating diff…
ai-status-preparing-question = Preparing question…
ai-status-generating-plan = Generating plan…
ai-status-updating-plan = Updating plan…
ai-status-summarizing-conversation = Summarizing conversation…
ai-status-summarizing-command-output = Summarizing command output…
ai-status-reading-files = Reading files…
ai-status-grepping = Grepping…
ai-status-finding-files = Finding files…
ai-status-executing-command = Executing command…
ai-status-writing-command-input = Writing command input…
ai-status-waiting-for-command-exit = Waiting for command to exit…
ai-status-searching-web = Searching the web…
ai-status-searching-web-for-query = Searching the web for “{ $query }”
ai-status-calling-mcp-tool-on-server = Calling “{ $name }” MCP tool on { $server }…
ai-status-calling-mcp-tool = Calling “{ $name }” MCP tool…
ai-status-reading-mcp-resource = Reading “{ $name }” MCP resource…
ai-status-next-check-suffix = { " " }· Next check in { $duration }
ai-permission-write-running-command = Can I write the following to this running command?
ai-permission-read-files = Grant access to the following files?
ai-permission-search-directory = OK if I search the files in this directory?
ai-permission-agent-asks-user-control = Agent is asking you to take control.
ai-requested-command-run = Run
ai-requested-command-done = Done
ai-requested-command-generating = Generating command…
ai-requested-command-permission = OK if I run this command and read the output?
ai-requested-mcp-permission = OK if I call this MCP tool?
ai-requested-command-monitoring = Agent is monitoring the command…
ai-requested-command-needs-input = Agent needs your input to continue
ai-requested-command-user-control = User is in control.
ai-requested-command-agent-paused = Agent paused. User is in control.
ai-requested-command-user-in-control = User in control
ai-requested-command-agent-error = Agent ran into an issue. Take over control.
ai-requested-command-viewing-detail = Viewing command details
ai-requested-mcp-viewing-detail = Viewing MCP tool call details
ai-requested-mcp-permission-on-server = OK if I call MCP tool { $tool } on server { $server }?
ai-requested-mcp-permission-named = OK if I call MCP tool { $tool }?
ai-requested-mcp-viewing-on-server = Viewing MCP tool { $tool } on { $server }
ai-requested-mcp-viewing-named = Viewing MCP tool { $tool }
ai-context-selected-text = Selected text
ai-context-block-count = { $count ->
    [one] 1 block
   *[other] { $count } blocks
}
ai-footer-enable-agent-notifications = Enable { $agent } notifications
common-find = Find
common-replace = Replace
common-out-of-credits = Out of credits
tab-config-fetching-branches = Fetching branches…
rules-editor-title = Rule editor
ai-byop-history-corrupted = Can't continue this conversation: an earlier tool result is missing or corrupted in this conversation's history, so InfiniShell can't safely send the request to your provider. Start a new conversation or fork from an earlier point to continue.
model-custom-endpoint = Custom endpoint
ai-assistant-ask = Ask InfiniShell AI
ai-assistant-restart = Restart
ai-assistant-generating-answer = Generating answer…
ai-assistant-accuracy-notice = AI responses can be inaccurate.
find-regex-tooltip = Regex search
find-case-sensitive-tooltip = Case-sensitive search
find-selected-block-tooltip = Find in selected block
find-preserve-case-tooltip = Preserve case
find-invert-filter-tooltip = Invert filter
find-context-lines-tooltip = Show context lines around matches
terminal-dynamic-enum-command-pending = Command pending…
terminal-dynamic-enum-command-failed = Command failed
terminal-dynamic-enum-no-results = Command returned no results
terminal-dynamic-enum-generate-message = Run the following command to generate variants:
terminal-dynamic-enum-run-command = Run command
terminal-local-skills-remote-error = Local skills cannot run on a remote machine. Try forking the conversation locally and running the skill.
terminal-model-edit-access-tooltip = Request edit access to change the model
ambient-agent-inherit-key-from-environment = Inherit key from environment
ambient-agent-choose-key-type = Choose a type
terminal-ssh-extension-connect-failed = Couldn't connect to the InfiniShell SSH extension
terminal-ssh-extension-connect-failed-description = Advanced features such as file browsing and code review are currently unavailable, but the rest of your terminal session remains fully functional.
terminal-ssh-tmux-deprecated = Tmux-based SSH integration has been deprecated
terminal-ssh-tmux-deprecated-description = InfiniShell now connects to remote sessions through its more reliable SSH extension. The tmux-based option has been removed.
settings-warpify-reuse-control-master-description = Attach to an active SSH ControlMaster already configured for the destination host instead of creating one managed by InfiniShell. This change applies to new tabs.
ai-prompt-alert-no-connection = No internet connection
ai-prompt-alert-at-limit = At limit —
ai-prompt-alert-configure-local-provider = Configure a local AI provider
ai-prompt-alert-use-own-api-keys = Use your own API keys
onboarding-prompt-setup-description = Next, set up your prompt. Use InfiniShell's customizable prompt, or select PS1 to keep your existing prompt configuration.
onboarding-prompt-custom-prompt-support = InfiniShell supports many custom prompts, including Oh My Zsh, Starship, and Powerlevel10k.{" "}
onboarding-prompt-shell-prompt-title = Shell prompt (PS1)
onboarding-prompt-no-existing-prompt = No existing prompt.
onboarding-prompt-look-incorrect = Doesn't look right?{" "}
onboarding-prompt-let-us-know = Let us know.
onboarding-prompt-infinishell-prompt-title = InfiniShell prompt
onboarding-prompt-customizable-description = Customize it in Appearance settings.
ai-code-diff-accept-and-continue = Accept and continue with agent
ai-code-diff-iterate-with-agent = Iterate with agent
ai-code-diff-open-config = Open config
ai-code-diff-manage-banner-settings = Manage suggested-code banner settings
ai-code-diff-new-file-name = { $file } (new)
ai-code-diff-deleted-file-name = { $file } (deleted)
code-review-outdated-comments-omitted = { $count ->
    [one] 1 comment will be omitted because it is outdated.
   *[other] { $count } comments will be omitted because they are outdated.
}
code-review-outdated-comment-count = { $count ->
    [one] 1 outdated comment
   *[other] { $count } outdated comments
}
code-review-comment-count = { $count ->
    [one] 1 comment
   *[other] { $count } comments
}
tab-config-add-new-repository = + Add new repository…
theme-scope-all-windows = All windows
theme-scope-this-window = This window
ai-orchestration-use-orchestration = Use orchestration
ai-orchestration-use-description = Break this work into coordinated streams with multiple agents.
ai-orchestration-base-model-helper = The primary model all agents will use.
common-prompt = Prompt
ai-assistant-question-placeholder = { " " }Ask a question…
ai-assistant-follow-up-placeholder = { " " }Type a response or select one above…
ai-assistant-explain-selection-prefix = Explain the following:
ai-assistant-next-step-question = What should I do next?
ai-assistant-fix-question = How do I fix this?
ai-assistant-command-output-prefix = I ran the command: `
ai-assistant-command-output-suffix = ` and got the following output:
ai-assistant-transcript-heading = ## InfiniShell AI transcript ({ $time })
ai-assistant-transcript-prompt = Prompt: { $prompt }
ai-assistant-transcript-answer = InfiniShell AI: { $answer }
ai-assistant-character-limit-exceeded = Character limit exceeded.
ai-assistant-prepared-prompt-git = How do I undo the most recent Git commits?
ai-assistant-prepared-prompt-files = How do I find all files containing specific text?
ai-assistant-prepared-prompt-script = Write a script to connect to an AWS EC2 instance.
ai-assistant-zero-state-help = Select a block or text, then press Shift+Ctrl+Space to ask InfiniShell AI.
ai-assistant-copy-code-tooltip = Copy code to clipboard [Cmd+C]
ai-assistant-insert-code-tooltip = Insert code into terminal input [Cmd+Enter]
ai-assistant-save-workflow-tooltip = Save as workflow [Cmd+S]
ai-assistant-copy-answer-tooltip = Copy answer to clipboard
ai-assistant-prepared-prompt-next = What should I do next?
ai-assistant-prepared-prompt-examples = Show examples.
ai-assistant-prepared-prompt-fix = How do I fix this?
ai-assistant-missing-context-notice = InfiniShell AI might forget earlier answers as conversations get long.
ai-assistant-credits-used = Credits used: { $used } / { $limit }.
ai-assistant-time-until-refresh = { $duration } until refresh.
ai-assistant-cloud-disabled = InfiniShell AI Assistant cloud requests are disabled in InfiniShell. Use Agent Mode with a configured BYOP model instead.
ai-assistant-duration-days = { $count ->
    [one] 1 day
   *[other] { $count } days
}
ai-assistant-duration-hours = { $count ->
    [one] 1 hour
   *[other] { $count } hours
}
ai-assistant-duration-minutes = { $count ->
    [one] 1 minute
   *[other] { $count } minutes
}
tab-config-invalid-worktree-branch-name = Name can contain only letters, numbers, hyphens, and underscores.
settings-workspace-override-tooltip = This option is enforced by your organization's settings and cannot be customized.
ambient-agent-usage-limit-reached = Agent usage limit reached. Try again later.
ambient-agent-server-overloaded = InfiniShell is temporarily overloaded. Try again shortly.
ai-error-request-failed = Request failed with error: { $error }
ai-error-quota-limit-reached = Quota limit reached.
ai-error-context-window-exceeded = Context window exceeded: { $message }
ai-error-context-too-large = The request context is too large. Automatic retries have stopped. Compact the conversation or reduce attached content, then try again.
ai-error-compaction-save-failed = Your pending request could not be saved. Check disk space and write permissions, then try again.
ai-error-compaction-invalid = The model did not return a complete conversation summary. Your original history is preserved. Try compacting the conversation again or switch models.
ai-error-invalid-api-key = Invalid API key for { $provider }
ai-error-bedrock-credentials-invalid = AWS Bedrock credentials for { $model } have expired or are invalid.
ai-error-gemini-enterprise-credentials-invalid = Gemini Enterprise credentials have expired or are invalid.
ai-error-transient-network = InfiniShell lost its connection while receiving the agent response. This is usually temporary.

    Debug info: { $debug }
ai-error-agent-exited-shell = The shell exited while the agent was running the command `{ $command }`, so the run could not continue. Make sure the agent is not asked to run commands or source scripts that can exit the shell.
terminal-ssh-error-detect-platform = Failed to detect the remote platform
terminal-ssh-error-preinstall-check = Failed to run the preinstallation check
terminal-ssh-error-check-extension = Failed to verify the SSH extension
terminal-ssh-error-install-extension = Failed to install the SSH extension
terminal-ssh-error-launch-extension = Failed to start the SSH extension
terminal-ssh-error-timeout = The operation timed out. Check your network connection.
terminal-ssh-error-unsupported-os = Unsupported operating system: { $os }
terminal-ssh-error-unsupported-architecture = Unsupported architecture: { $arch }
terminal-ssh-error-script-failed = Script exited with code { $code }: { $stderr }
terminal-ssh-error-with-detail = { $body }. { $detail }
terminal-ssh-error-body-only = { $body }.
terminal-ssh-enhanced-opt-in-enable-error = InfiniShell could not enable enhanced SSH for this host: { $error }
terminal-ssh-enhanced-opt-in-required = InfiniShell enhanced SSH requires per-host OpenSSH settings because { $options } are enabled.
terminal-ssh-enhanced-opt-in-security-impact = This disables automatic host-key updates and keystroke-timing obfuscation for this host.
terminal-ssh-enhanced-opt-in-prompt = Press Enter or type yes to update ~/.ssh/config and continue, or type no to use native OpenSSH [Y/n]:
terminal-ssh-enhanced-opt-in-native-fallback = Using native OpenSSH without InfiniShell SSH extension features.
terminal-ssh-enhanced-opt-in-enabled = Enhanced SSH is enabled for { $host }; continuing this connection.
terminal-ssh-enhanced-opt-in-invalid-choice = Please press Enter or type yes or no.
code-review-creating-pull-request-loading = Creating pull request…
code-review-publishing-loading = Publishing…
code-review-pushing-loading = Pushing…
terminal-default-shell-unsupported = InfiniShell doesn't currently support your default shell, so it is falling back to zsh.{"  "}
editor-go-to-line = Go to line
code-add-selection-as-context = Add as context
tab-config-worktree-name-placeholder = my-feature-branch
tab-config-new-worktree-title = New worktree
tab-config-autogenerate-worktree-branch-name = Generate worktree branch name automatically
tab-config-select-directory = Select directory
tab-config-select-repository-for-worktree = Select a Git repository to enable worktree support
sftp-file-manager-title = File manager
sftp-pane-title = SFTP: { $path }
sftp-rename-placeholder = Enter a new name
sftp-folder-name-placeholder = Folder name
sftp-search-placeholder = Search files…
sftp-status-connecting = Connecting…
sftp-status-disconnected = Disconnected
sftp-status-loading = Loading…
sftp-folder-empty = This folder is empty
sftp-drop-files-to-upload = Drop files to upload
sftp-column-name = Name
sftp-column-size = Size
sftp-column-modified = Modified
sftp-menu-open = Open
sftp-menu-download = Download
sftp-menu-rename = Rename
sftp-menu-delete = Delete
sftp-menu-details = Details
sftp-delete-description-one = Are you sure you want to delete “{ $name }”? This action cannot be undone.
sftp-delete-description-many = Are you sure you want to delete { $count } items? This action cannot be undone.
sftp-delete-title = Confirm deletion
sftp-rename-title = Rename
sftp-rename-current-name = Current name: { $name }
sftp-create-folder-title = New folder
sftp-file-details-title = File details
sftp-file-type-file = File
sftp-file-type-directory = Directory
sftp-file-type-symlink = Symbolic link
sftp-file-type-other = Other
sftp-detail-type = Type
sftp-detail-size = Size
sftp-detail-modified = Modified
sftp-detail-permissions = Permissions
sftp-detail-path = Path
sftp-move-description = Move “{ $source }” to { $target }
sftp-move-title = Move file
sftp-move-button = Move
sftp-overwrite-upload-description = A remote file named { $name } already exists. Overwrite it?
sftp-overwrite-download-description = A destination file named { $name } already exists. Overwrite it?
sftp-overwrite-title = Confirm overwrite
sftp-overwrite-button = Overwrite
sftp-close-transfers-title = Close transfer panel
sftp-close-transfers-description = Transfers are still in progress. Closing the panel will interrupt all transfers and clear the history. Close it anyway?
sftp-transfer-pending = Pending
sftp-transfer-in-progress = Transferring
sftp-transfer-completed = Completed
sftp-transfer-failed = Failed
sftp-transfer-cancelled = Cancelled
sftp-transfer-panel-title = Transfers ({ $count })
sftp-error-connection = Connection error: { $message }
sftp-error-operation = Operation error: { $message }
sftp-error-local-io = Local I/O error: { $message }
sftp-error-no-credentials = Credentials not found: { $message }
sftp-error-transfer-cancelled = Transfer cancelled
sftp-error-create-channel = Failed to create the SFTP channel: { $error }
sftp-error-connection-cancelled = Connection cancelled
sftp-error-server-config-not-found = Server configuration not found
sftp-error-read-server-config = Failed to read the server configuration: { $error }
sftp-error-not-connected = Not connected to the server
sftp-error-list-directory = Failed to list the directory: { $error }
sftp-error-delete = Failed to delete: { $error }
sftp-error-invalid-file-name = The file name contains invalid characters
sftp-error-upload = Upload failed: { $error }
sftp-error-download = Download failed: { $error }
sftp-error-name-empty = Name cannot be empty
sftp-error-folder-name-empty = Folder name cannot be empty
sftp-error-name-path-separator = Names cannot contain path separators
sftp-error-rename = Rename failed: { $error }
sftp-error-create-folder = Failed to create the folder: { $error }
sftp-error-invalid-target-path = The target path is invalid
sftp-error-move = Move failed: { $error }
sftp-backend-list-directory = Failed to list directory { $path }: { $error }
sftp-backend-read-directory-entry = Failed to read a directory entry: { $error }
sftp-backend-read-metadata = Failed to read metadata: { $error }
sftp-backend-delete-file = Failed to delete file { $path }: { $error }
sftp-backend-delete-directory = Failed to recursively delete directory { $path }: { $error }
sftp-backend-create-directory = Failed to create directory { $path }: { $error }
sftp-backend-rename = Failed to rename { $source } to { $target }: { $error }
sftp-backend-resolve-path = Failed to resolve path { $path }: { $error }
sftp-backend-stat = Failed to get information for { $path }: { $error }
sftp-backend-create-local-directory = Failed to create the local directory: { $error }
sftp-backend-upload-file = Failed to upload the file: { $error }
sftp-backend-open-remote-file = Failed to open the remote file: { $error }
sftp-backend-create-local-file = Failed to create the local file: { $error }
sftp-backend-read = Read failed: { $error }
sftp-backend-write = Write failed: { $error }
sftp-backend-flush = Flush failed: { $error }
sftp-ops-resolve-auth = Failed to resolve authentication: { $error }
sftp-ops-rename-remote-temporary = Failed to rename the remote temporary file: { $error }. Temporary file: { $path }
sftp-ops-rename-downloaded-temporary = Rename failed: { $error }. The downloaded temporary file was preserved at: { $path }
sftp-ops-read-password = Failed to read the password: { $error }
sftp-ops-password-not-stored = No password is stored for server { $host }
sftp-ops-key-path-missing = Key authentication is selected, but no key path is configured
agent-local-child-wasm-unsupported = Local child agents are not supported in WASM builds.
terminal-loading-prompt = Loading prompt…
terminal-project-skill-badge = Project skill
terminal-rewind-no-code-to-restore = No code to restore
terminal-sharing-inactivity-title = Are you still there?
terminal-sharing-inactivity-countdown = Sharing will end in { $time } due to inactivity.
ambient-agent-api-key-name-placeholder = e.g. My API key
ambient-agent-api-key-save-failed = Failed to save API key: { $error }
ambient-agent-api-key-saved = API key “{ $name }” saved.
ambient-agent-enter-credentials = Enter your credentials below.
ambient-agent-select-api-key-type-description = Select an API key type to use { $harness } in the cloud with InfiniShell Agent.
ambient-agent-credentials-encrypted = Your credentials are end-to-end encrypted.{" "}
ambient-agent-authentication-learn-more = Learn more about authentication for { $harness } in InfiniShell.
ambient-agent-share-with-team = Share with team
ambient-agent-optional-field = { $field } (optional)
command-search-title = Command Search
command-search-looking-for = I'm looking for…
command-search-example-queries = Example queries
ai-long-context-pricing-warning = OpenAI automatically applies long-context pricing when context exceeds 272,000 tokens.{" "}
ai-web-fetch-no-urls = No URLs fetched
ai-web-search-no-urls = No URLs found
ai-toggle-selection-hint = to toggle selection
ai-github-authentication-missing = GitHub authentication is missing.
ai-authenticate-github = Authenticate with GitHub
ai-cloud-agent-run-cancelled = Cloud agent run cancelled
ai-search-target-conversation = conversation
ai-search-target-agent-run = agent run
ai-search-status-searched = Searched
ai-search-status-searching = Searching
ai-search-target-this-conversation = this conversation
ai-search-query-suffix = : { $query }
ai-code-diff-apply-failed = Could not apply changes to the file.
ai-code-diff-edited-in-another-tab = This suggestion is being edited in another tab.
ai-references-title = References
ai-suggestions-title = Suggestions:
common-beta = Beta
common-beta-uppercase = BETA
terminal-file-upload-to = { " " }to{" " }
common-or-spaced = { " " }or{" " }
common-custom = Custom
input-suggestion-last-ran = Last ran { $time }
input-suggestion-a11y = Suggestion: { $text }.
input-suggestion-selected-a11y = Selected: { $text }
editor-pasting-a11y = Pasting: { $content }
session-last-run-command-a11y = Last run command: { $command }
session-last-ai-interaction-a11y = Last AI interaction: { $prompt }
session-running-command-a11y = Currently running: { $command }
session-running-ai-interaction-a11y = Currently running AI interaction: { $prompt }
code-review-unsaved-changes-tooltip = This file has unsaved changes. { $shortcut } to save
tab-config-remove-title = Remove “{ $name }”?
tab-config-remove-description = This tab config will be permanently deleted. This action cannot be undone.
tab-config-enter-parameter = Enter { $name }
tab-config-default-value = Default: { $value }
search-directory-a11y = Directory: { $path }
search-file-a11y = File: { $path }
search-directory-help = Press Enter to navigate to this directory
search-file-help = Press Enter to open this file
search-create-file-label = Create a file named { $name }…
search-create-file-a11y = Create file: { $name }
search-create-file-help = Press Enter to create { $name } in the current directory
context-chip-create-branch = Create new branch “{ $name }”
search-selected-a11y = Selected: { $item }
search-selected-tab-a11y = Selected tab: { $title }.
search-section-a11y = Section: { $title }
search-notebook-a11y = Notebook: { $name }
search-notebook-with-description-a11y = Notebook: { $name } — { $description }
search-workflow-a11y = Workflow: { $name }
search-workflow-with-description-a11y = Workflow: { $name } — { $description }
search-secret-a11y = Secret: { $name }
search-loading-suggestions-a11y = Loading { $filter } suggestions
search-skill-a11y = Skill: { $name }
search-command-a11y = Command: { $command }
search-rule-a11y = Rule: { $rule }
search-conversation-a11y = Conversation: { $title }
search-ai-query-a11y = AI query: { $query }
search-ai-prompt-a11y = AI prompt: { $prompt }
search-history-item-a11y = History item: { $item }
search-project-a11y = Project: { $name }
search-query-a11y = Query: { $query }
search-prompt-a11y = Prompt: { $name }
search-plan-a11y = Plan: { $title }
search-profile-a11y = Profile: { $name }
search-indexed-repository-a11y = Indexed repository: { $name }
terminal-rewind-no-code-a11y = Rewind to: { $query } (no code changes)
common-show-count-more = Show { $count } more
ai-mcp-server-default-name = MCP Server { $id }
terminal-open-path-in-infinishell = Open { $path } in InfiniShell
ambient-agent-running-harness = Running { $name }…
ambient-agent-api-key-deleted = API key “{ $name }” deleted.
ambient-agent-api-key-delete-failed = Failed to delete API key “{ $name }”: { $error }
ambient-agent-delete-api-key-a11y = Delete API key { $name }
terminal-file-not-found = File not found: { $path }
ai-create-environment-failed = Failed to create environment: { $error }
common-error-with-detail = Error: { $error }
ai-load-conversation-failed = Failed to load conversation with ID: { $id }
ai-plan-not-found = Plan document { $id } was not found in InfiniShell Drive.
ai-web-search-searching = Searching the web for “{ $query }”…
ai-web-search-searched = Searched the web for “{ $query }”
ai-web-search-failed = Web search failed for “{ $query }”
ai-web-fetch-fetching = Fetching { $count } web pages…
ai-web-fetch-fetched = Fetched { $count } web pages
ai-web-fetch-fetched-partial = Fetched { $successful } of { $total } web pages
ai-add-rule = Add rule: { $rule }
ai-suggested-prompt-a11y = Suggested prompt:
    { $prompt }
ai-mcp-tool-title = MCP Tool: { $name }
ai-mcp-tool-title-with-input = MCP Tool: { $name } ({ $input })
ai-navigate-to-open-comments = Navigate to { $path } to open these comments
ai-thought-for-duration = Thought for { $duration }
ai-stopped-task = Stopped task: “{ $name }”
ai-comment-addressed = Comment addressed: “{ $content }”
ai-completed-task = Completed { $title }
ai-grep-one-pattern = Grep for `{ $query }` in { $path }
ai-grep-multiple-patterns = Grep for the following patterns in { $path }:
    { $patterns }
ai-invalid-api-key-update-settings = Invalid API key for { $provider }. Update your API key in settings.
terminal-selected-blocks-a11y = Selected { $count } blocks.
terminal-copied-blocks-a11y = Copied { $count } blocks.
    { $content }
terminal-open-block-filter-a11y = Open block filter editor for block { $index }
common-copy-item = Copy { $item }
workspace-new-worktree-with-branch = New worktree: { $repo }, { $branch }
workspace-new-worktree = New worktree: { $repo }
workspace-worktree = Worktree: { $repo }
search-ssh-server-a11y = SSH server: { $name } { $host }
ai-delete-conversation-title = Delete “{ $title }”?
terminal-workflow-command-inserted-a11y = Workflow command { $command } inserted.
terminal-selected-workflow-argument-a11y = Selected workflow argument { $name }
terminal-executed-command-a11y = Executed: { $command }
search-model-a11y = Model: { $name }
ai-export-file-already-exists = File { $path } already exists
notebook-command-from = Command from { $source }
ai-custom-model-description = Custom · { $name }
ai-usage-models-category = Models ({ $category })
ambient-agent-create-hidden-pane-failed = Failed to create a hidden pane for the local child agent.
ambient-agent-create-local-child-failed = Failed to create local child task: { $error }
ambient-agent-local-harness-missing = Local child harness type is missing.
ambient-agent-local-harness-unsupported = Unsupported local child harness “{ $name }”.
ssh-manager-imported-from = Imported from { $path }
tab-config-auto-worktree-required-tooltip = Select automatic worktree creation before enabling this option.
ai-export-permission-denied = Permission denied writing to { $path }. Check file permissions.
ai-export-directory-not-found = Directory not found: { $path }
ai-export-failed = Failed to export to { $path }: { $error }
code-save-file-failed = Failed to save file: { $error }
code-delete-file-failed = Failed to delete file: { $error }
settings-mcp-parse-markdown-failed = Failed to parse Markdown: { $error }

# Settings UI residual localization audit
common-off = Off
settings-section-agents = Agents
toolbar-editor-left-side = Left side
toolbar-editor-right-side = Right side

settings-ai-usage-duration-monthly = monthly
settings-ai-usage-limit-description = This is the { $duration } limit of AI credits for your account.
settings-ai-default-prompt-submission-mode = Default prompt submission mode
settings-ai-default-prompt-submission-description = What happens when you submit a new prompt while the agent is still responding. You can override this per conversation using the auto-queue toggle.
settings-ai-default-long-running-submission-mode = Default long-running command submission mode
settings-ai-default-long-running-submission-description = What happens when you submit a prompt while an agent is driving an agent-requested long-running command. Queued prompts are sent to the agent when the command finishes.
settings-ai-submit-rich-input = Submit Rich Input with Ctrl+Enter
settings-ai-gemini-enterprise-section = Gemini Enterprise
settings-ai-gemini-enterprise-toggle = Use Gemini Enterprise credentials
settings-ai-gemini-enterprise-description = InfiniShell routes eligible requests through your workspace's Gemini Enterprise Google Cloud project.
settings-ai-gemini-enterprise-description-managed = InfiniShell routes eligible requests through your workspace's Gemini Enterprise Google Cloud project. This setting is managed by your organization.
settings-ai-custom-routers-section = Custom Routers
settings-ai-custom-routers-description = Automatically route tasks to specific models based on task complexity or custom rules. Custom routers will appear in your model selector menu.
custom-router-summary-default = Default:
custom-router-summary-easy = Easy:
custom-router-summary-medium = Medium:
custom-router-summary-hard = Hard:
custom-router-summary-rule-count = { $count ->
    [one] 1 rule
   *[other] { $count } rules
}

settings-command-show-agent-tips = Show agent tips
settings-command-hide-agent-tips = Hide agent tips
settings-command-show-use-agent-footer = Show “Use Agent” footer
settings-command-hide-use-agent-footer = Hide “Use Agent” footer
settings-command-allow-full-access-denylist-bypass = Allow Full Access to bypass command denylist
settings-command-require-full-access-denylist-approval = Require approval for denylisted commands in Full Access
settings-command-allow-auto-approve-denylist-bypass = Allow auto-approve to bypass command denylist
settings-command-require-auto-approve-denylist-approval = Require approval for denylisted commands in auto-approve
settings-command-show-code-review-tab-button = Show code review button in tab bar
settings-command-hide-code-review-tab-button = Hide code review button in tab bar

toggle-suffix-git-operations = commit and pull request generation
toggle-suffix-agent-command-history = agent-executed commands in history
toggle-suffix-conversation-history = conversation history in the tools panel
toggle-suffix-model-picker = model picker in the prompt
toggle-suffix-coding-agent-toolbar = coding agent toolbar
toggle-suffix-warp-credit-fallback = InfiniShell credit fallback
toggle-suffix-auto-toggle-rich-input = automatic Rich Input visibility based on agent status
toggle-suffix-auto-open-rich-input = automatically opening Rich Input when a coding agent session starts
toggle-suffix-auto-dismiss-rich-input = automatically dismissing Rich Input after prompt submission
toggle-suffix-window-custom-size = opening new windows with a custom size
toggle-suffix-window-blur-texture = acrylic window blur texture
toggle-suffix-tools-panel-consistency = consistent tools panel visibility across tabs
toggle-suffix-agent-font-match-terminal = matching the agent font to the terminal font
toggle-suffix-notebook-font-match-terminal = matching the notebook font size to the terminal font size
toggle-suffix-vertical-tabs-restored = showing the vertical tabs panel in restored windows
toggle-suffix-latest-prompt-tab-title = using the latest user prompt as the conversation title in tab names
toggle-suffix-preserve-active-tab-color = preserving the active tab color for new tabs
toggle-suffix-alt-screen-custom-padding = custom padding in the alternate screen
toggle-suffix-auto-open-code-review = automatically opening the code review panel
toggle-suffix-code-review-button = the code review button
toggle-suffix-code-review-diff-stats = diff stats on the code review button
toggle-suffix-project-explorer = the project explorer
toggle-suffix-global-file-search = global file search
toggle-suffix-show-hidden-project-files = hidden files in the project explorer
toggle-suffix-autosuggestion-ignore = the autosuggestion ignore button
toggle-suffix-ssh-control-master = reusing an existing SSH ControlMaster in the InfiniShell SSH wrapper
toggle-suffix-long-running-notifications = long-running command notifications
toggle-suffix-agent-task-notifications = agent task completion notifications
toggle-suffix-needs-attention-notifications = needs-attention notifications
toggle-suffix-notification-sounds = notification sounds
toggle-suffix-in-app-agent-notifications = in-app agent notifications
toggle-suffix-mouse-reporting = mouse reporting
toggle-suffix-help-block = the help block in new sessions
toggle-suffix-at-context-menu = the “@” context menu in terminal mode
toggle-suffix-preserve-input-focus = preserving input focus on block selection
toggle-suffix-global-workflows-command-search = global workflows in Command Search

settings-code-show-hidden-files = Show hidden files in project explorer
settings-code-show-hidden-files-description = Show dotfiles and hidden files (starting with .) in the project explorer.
settings-code-auto-save = Auto save
settings-code-auto-save-description = Automatically save changes in the InfiniShell text editor as you type and when the editor loses focus.

settings-appearance-category-tools-panel = Tools panel
settings-appearance-font-default-option = { $font } (default)
settings-appearance-font-weight-normal = Normal
settings-appearance-font-weight-bold = Bold
settings-appearance-show-in-dock = Show InfiniShell in Dock
settings-appearance-tab-hide-title-bar-search-label = Hide search bar in vertical tab layout
settings-appearance-tab-hide-title-bar-search-description = When using the vertical tab layout, hide the search bar in the title bar. Search stays available via the command palette and keyboard shortcuts.
settings-appearance-icon-aurora = Aurora
settings-appearance-icon-default = Default
settings-appearance-icon-classic-1 = Classic 1
settings-appearance-icon-classic-2 = Classic 2
settings-appearance-icon-classic-3 = Classic 3
settings-appearance-icon-comets = Comets
settings-appearance-icon-glass-sky = Glass Sky
settings-appearance-icon-glitch = Glitch
settings-appearance-icon-cow = Cow
settings-appearance-icon-glow = Glow
settings-appearance-icon-holographic = Holographic
settings-appearance-icon-mono = Mono
settings-appearance-icon-neon = Neon
settings-appearance-icon-original = Original
settings-appearance-icon-starburst = Starburst
settings-appearance-icon-sticker = Sticker
settings-appearance-icon-infinishell-1 = InfiniShell 1

settings-features-main-screen = Main Screen
settings-features-numbered-screen = Screen { $number }
settings-features-code-editor-line-numbers = Code editor line numbers:
settings-features-preserve-input-focus = Preserve input focus on block selection
settings-features-osc52-clipboard-access = Clipboard access (OSC 52)
settings-features-osc52-clipboard-access-description = Control whether programs running in the terminal can read or write your system clipboard.
settings-features-asynchronous-find = Asynchronous find
settings-features-asynchronous-find-description = Use an improved implementation of find to keep the UI responsive while searching for matches in large outputs.

settings-warp-drive-local-description = InfiniShell Drive is a local workspace in your terminal where you can save Workflows, Notebooks, Prompts, and Environment Variables on this device.
settings-warpify-reuse-control-master = Reuse existing SSH ControlMaster
settings-exec-profile-run-agents = Run agents:
settings-network-error-invalid-proxy-url = Invalid proxy URL
settings-network-error-timeout = Timed out after { $seconds } seconds
