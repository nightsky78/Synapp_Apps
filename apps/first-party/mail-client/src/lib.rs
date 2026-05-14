use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::ffi::{c_char, CString};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;
const MAX_BATCH_IDS: usize = 250;
const MAX_RECIPIENTS: usize = 100;
const MAX_SUBJECT_LENGTH: usize = 998;
const MAX_FOLDER_NAME_LENGTH: usize = 128;
const MAX_RULE_NAME_LENGTH: usize = 120;
const MAX_HOST_ID_LENGTH: usize = 128;
const MAX_SEARCH_QUERY_LENGTH: usize = 256;
const MAX_FILTER_TEXT_LENGTH: usize = 256;
const MAX_FILTER_IDS: usize = 64;
const MAX_RULE_CONDITIONS: usize = 10;
const MAX_RULE_ACTIONS: usize = 10;
const MAX_BODY_LENGTH: usize = 512_000;
const MAX_ALLOWED_UNDO_SECONDS: u32 = 30;

const EFFECT_ACCOUNT_CREDENTIAL_READ: &str = "AccountCredentialRead";
const EFFECT_ACCOUNT_CREDENTIAL_WRITE: &str = "AccountCredentialWrite";
const EFFECT_PROVIDER_CAPABILITY_READ: &str = "ProviderCapabilityRead";
const EFFECT_STORE_PLATFORM_SECRET: &str = "StorePlatformSecret";
const EFFECT_OAUTH_CONNECT: &str = "OAuthConnect";
const EFFECT_OAUTH_DISCONNECT: &str = "OAuthDisconnect";
const EFFECT_IMAP_FETCH: &str = "ImapFetch";
const EFFECT_IMAP_SYNC: &str = "ImapSync";
const EFFECT_SMTP_SEND: &str = "SmtpSend";
const EFFECT_CONTACT_LOOKUP: &str = "ContactLookup";
const EFFECT_NOTIFY_USER: &str = "NotifyUser";
const EFFECT_SCHEDULE_JOB: &str = "ScheduleJob";
const EFFECT_CANCEL_JOB: &str = "CancelJob";
const EFFECT_MAIL_STORE_READ: &str = "MailStoreRead";
const EFFECT_MAIL_STORE_WRITE: &str = "MailStoreWrite";
const EFFECT_MAIL_STORE_DELETE: &str = "MailStoreDelete";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Permission {
    None,
    Read,
    Draft,
    Send,
    Organize,
    Accounts,
    Admin,
}

impl Permission {
    fn parse(value: &str) -> Result<Self, ErrorResponse> {
        match value {
            "none" => Ok(Self::None),
            "read" => Ok(Self::Read),
            "draft" => Ok(Self::Draft),
            "send" => Ok(Self::Send),
            "organize" => Ok(Self::Organize),
            "accounts" => Ok(Self::Accounts),
            "admin" => Ok(Self::Admin),
            _ => Err(ErrorResponse::new(
                "InvalidInput",
                format!("Unknown permission level: {value}"),
            )),
        }
    }

    fn satisfies(self, required: Self) -> bool {
        match self {
            Self::Admin => true,
            Self::Accounts => matches!(required, Self::None | Self::Read | Self::Accounts),
            Self::Organize => matches!(required, Self::None | Self::Read | Self::Organize),
            Self::Send => matches!(required, Self::None | Self::Read | Self::Draft | Self::Send),
            Self::Draft => matches!(required, Self::None | Self::Read | Self::Draft),
            Self::Read => matches!(required, Self::None | Self::Read),
            Self::None => matches!(required, Self::None),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

impl ErrorResponse {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum AppResult<T: Serialize> {
    Ok { ok: T },
    Err { err: ErrorResponse },
}

#[derive(Serialize, Debug)]
struct OperationResponse<T: Serialize> {
    operation: &'static str,
    status: &'static str,
    data: T,
    host_effects: Vec<HostEffect>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<WarningMessage>,
}

#[derive(Serialize, Debug, Clone)]
struct HostEffect {
    effect: &'static str,
    intent: &'static str,
    payload: Value,
    idempotency_key: String,
}

#[derive(Serialize, Debug, Clone)]
struct WarningMessage {
    code: &'static str,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RequestContext {
    user_id: String,
    permission: String,
    #[serde(default)]
    available_effects: Vec<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
}

impl RequestContext {
    fn permission(&self) -> Result<Permission, ErrorResponse> {
        ensure_non_empty("context.user_id", &self.user_id)?;
        Permission::parse(&self.permission)
    }

    fn idempotency_key(&self, operation: &'static str, effect: &str, seed: &str) -> String {
        let payload_hash = hash_text(seed);
        if let Some(request_id) = &self.request_id {
            return format!(
                "{operation}:{}:{effect}:{}:{request_id}",
                self.user_id, payload_hash
            );
        }

        format!("{operation}:{}:{effect}:{payload_hash}", self.user_id)
    }
}

trait HasContext {
    fn context(&self) -> &RequestContext;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MailAccountSummary {
    account_id: String,
    display_name: String,
    email_address: String,
    #[serde(default)]
    provider: Option<String>,
    enabled: bool,
    is_default: bool,
    sync_interval_minutes: u32,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    connection_state: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MailAccountStatus {
    account_id: String,
    connection_state: String,
    #[serde(default)]
    last_sync_at: Option<i64>,
    #[serde(default)]
    unread_count: Option<u32>,
    #[serde(default)]
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Mailbox {
    mailbox_id: String,
    #[serde(default)]
    account_id: Option<String>,
    name: String,
    kind: String,
    unread_count: u32,
    total_count: u32,
    #[serde(default)]
    parent_id: Option<String>,
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    sync_state: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EmailAddress {
    email: String,
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Attachment {
    attachment_id: String,
    file_name: String,
    #[serde(default)]
    content_type: Option<String>,
    size_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EmailMessage {
    email_id: String,
    account_id: String,
    mailbox_id: String,
    #[serde(default)]
    thread_id: Option<String>,
    from: EmailAddress,
    #[serde(default)]
    to: Vec<EmailAddress>,
    #[serde(default)]
    cc: Vec<EmailAddress>,
    #[serde(default)]
    bcc: Vec<EmailAddress>,
    subject: String,
    #[serde(default)]
    preview: Option<String>,
    #[serde(default)]
    body_html: Option<String>,
    #[serde(default)]
    body_text: Option<String>,
    #[serde(default)]
    received_at: Option<i64>,
    #[serde(default)]
    sent_at: Option<i64>,
    is_read: bool,
    is_flagged: bool,
    has_attachments: bool,
    importance: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    attachments: Vec<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DraftDocument {
    #[serde(default)]
    draft_id: Option<String>,
    account_id: String,
    #[serde(default)]
    to: Vec<EmailAddress>,
    #[serde(default)]
    cc: Vec<EmailAddress>,
    #[serde(default)]
    bcc: Vec<EmailAddress>,
    subject: String,
    #[serde(default)]
    body_html: Option<String>,
    #[serde(default)]
    body_text: Option<String>,
    #[serde(default)]
    attachments: Vec<Attachment>,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    importance: Option<String>,
    #[serde(default)]
    request_read_receipt: bool,
    #[serde(default)]
    scheduled_send_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RuleCondition {
    field: String,
    operator: String,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RuleAction {
    action_type: String,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MailRule {
    rule_id: String,
    name: String,
    enabled: bool,
    stop_processing: bool,
    priority: u32,
    #[serde(default)]
    conditions: Vec<RuleCondition>,
    #[serde(default)]
    actions: Vec<RuleAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Pagination {
    offset: u32,
    limit: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SortSpec {
    field: String,
    direction: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct SearchFilters {
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    subject_contains: Option<String>,
    #[serde(default)]
    body_contains: Option<String>,
    #[serde(default)]
    has_attachments: Option<bool>,
    #[serde(default)]
    unread: Option<bool>,
    #[serde(default)]
    flagged: Option<bool>,
    #[serde(default)]
    importance: Option<String>,
    #[serde(default)]
    account_ids: Option<Vec<String>>,
    #[serde(default)]
    mailbox_ids: Option<Vec<String>>,
    #[serde(default)]
    before: Option<i64>,
    #[serde(default)]
    after: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EmailPageSnapshot {
    messages: Vec<EmailMessage>,
    total_count: u32,
    unread_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchSnapshot {
    results: Vec<EmailMessage>,
    total_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnreadCount {
    mailbox_id: String,
    unread_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerConfig {
    protocol: String,
    host: String,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    security: Option<String>,
    username: String,
    auth_method: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_input_ref: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_ref: Option<String>,
    #[serde(default)]
    use_incoming_secret: Option<bool>,
    #[serde(default)]
    path_prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OAuthAccountConfig {
    provider: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    connection_ref: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_state_ref: Option<String>,
    #[serde(default)]
    scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AccountSyncConfig {
    #[serde(default)]
    interval_minutes: Option<u32>,
    #[serde(default)]
    initial_range: Option<String>,
    #[serde(default)]
    download_attachments: Option<String>,
    #[serde(default)]
    offline_cache: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FolderMapping {
    #[serde(default)]
    inbox: Option<String>,
    #[serde(default)]
    sent: Option<String>,
    #[serde(default)]
    drafts: Option<String>,
    #[serde(default)]
    trash: Option<String>,
    #[serde(default)]
    archive: Option<String>,
    #[serde(default)]
    junk: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserMailAccountSetup {
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    revision: Option<String>,
    account_label: String,
    display_name: String,
    email_address: String,
    #[serde(default)]
    reply_to: Option<String>,
    provider: String,
    auth_method: String,
    #[serde(default)]
    incoming: Option<ServerConfig>,
    #[serde(default)]
    outgoing: Option<ServerConfig>,
    #[serde(default)]
    oauth: Option<OAuthAccountConfig>,
    #[serde(default)]
    sync: Option<AccountSyncConfig>,
    #[serde(default)]
    folder_mapping: Option<FolderMapping>,
    #[serde(default)]
    desired_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ProviderCapabilities {
    #[serde(default)]
    providers: Vec<Value>,
    #[serde(default)]
    manual_imap_smtp: Value,
    #[serde(default)]
    policy: Value,
    #[serde(default)]
    oauth_placeholders: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ConnectionTestStepResult {
    step: String,
    state: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    field_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ConnectionTestResult {
    #[serde(default)]
    test_id: Option<String>,
    #[serde(default)]
    steps: Vec<ConnectionTestStepResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserMailPreferences {
    reading: Value,
    compose: Value,
    notifications: Value,
    sync_defaults: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MailIdentity {
    identity_id: String,
    account_id: String,
    kind: String,
    display_name: String,
    email_address: String,
    #[serde(default)]
    reply_to: Option<String>,
    #[serde(default)]
    signature_id: Option<String>,
    enabled: bool,
    is_default_for_account: bool,
    is_global_default: bool,
    verification_state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MailSignature {
    signature_id: String,
    account_id: String,
    #[serde(default)]
    identity_id: Option<String>,
    label: String,
    #[serde(default)]
    body_html: Option<String>,
    #[serde(default)]
    body_text: Option<String>,
    enabled: bool,
    use_for_new: bool,
    use_for_replies: bool,
    use_for_forwards: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LegacySettings {
    #[serde(default)]
    ui_schemas: Value,
    #[serde(default)]
    settings: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetProviderCapabilitiesRequest {
    context: RequestContext,
    #[serde(default)]
    provider_ids: Vec<String>,
}

impl HasContext for GetProviderCapabilitiesRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidateAccountSetupRequest {
    context: RequestContext,
    account: UserMailAccountSetup,
    #[serde(default)]
    admin_policy_snapshot: Option<ProviderCapabilities>,
    #[serde(default)]
    existing_account_snapshot: Option<UserMailAccountSetup>,
}

impl HasContext for ValidateAccountSetupRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlanConnectionTestRequest {
    context: RequestContext,
    account: UserMailAccountSetup,
    test_scope: String,
    #[serde(default)]
    admin_policy_snapshot: Option<ProviderCapabilities>,
}

impl HasContext for PlanConnectionTestRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CompleteAccountSetupRequest {
    context: RequestContext,
    account: UserMailAccountSetup,
    #[serde(default)]
    connection_test_result: Option<ConnectionTestResult>,
    #[serde(default)]
    make_default: bool,
    #[serde(default)]
    save_mode: Option<String>,
}

impl HasContext for CompleteAccountSetupRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BeginOAuthAccountSetupRequest {
    context: RequestContext,
    provider: String,
    email_address: String,
    #[serde(default)]
    account_label: Option<String>,
    #[serde(default)]
    requested_scopes: Vec<String>,
    #[serde(default)]
    redirect_route: Option<String>,
}

impl HasContext for BeginOAuthAccountSetupRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CompleteOAuthAccountSetupRequest {
    context: RequestContext,
    provider: String,
    authorization_ref: String,
    oauth_connection_ref: String,
    #[serde(default)]
    account: Option<UserMailAccountSetup>,
    #[serde(default)]
    connection_test_result: Option<ConnectionTestResult>,
}

impl HasContext for CompleteOAuthAccountSetupRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DisconnectOAuthAccountRequest {
    context: RequestContext,
    account_id: String,
    #[serde(default)]
    oauth_connection_ref: Option<String>,
    #[serde(default)]
    disable_account: bool,
}

impl HasContext for DisconnectOAuthAccountRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RemoveAccountRequest {
    context: RequestContext,
    account_id: String,
    remove_local_cache: bool,
    cancel_scheduled_sends: bool,
    #[serde(default)]
    delete_drafts: bool,
    #[serde(default)]
    revision: Option<String>,
}

impl HasContext for RemoveAccountRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct IdentitySnapshot {
    #[serde(default)]
    identities: Vec<MailIdentity>,
    #[serde(default)]
    signatures: Vec<MailSignature>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ListIdentitiesRequest {
    context: RequestContext,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    snapshot: Option<IdentitySnapshot>,
}

impl HasContext for ListIdentitiesRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveIdentityRequest {
    context: RequestContext,
    identity: MailIdentity,
    #[serde(default)]
    revision: Option<String>,
}

impl HasContext for SaveIdentityRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteIdentityRequest {
    context: RequestContext,
    identity_id: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    force: bool,
}

impl HasContext for DeleteIdentityRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveSignatureRequest {
    context: RequestContext,
    signature: MailSignature,
    #[serde(default)]
    revision: Option<String>,
}

impl HasContext for SaveSignatureRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteSignatureRequest {
    context: RequestContext,
    signature_id: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    force: bool,
}

impl HasContext for DeleteSignatureRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetUserPreferencesRequest {
    context: RequestContext,
    #[serde(default)]
    snapshot: Option<UserMailPreferences>,
    #[serde(default)]
    include_defaults: bool,
}

impl HasContext for GetUserPreferencesRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveUserPreferencesRequest {
    context: RequestContext,
    preferences: UserMailPreferences,
    #[serde(default)]
    revision: Option<String>,
}

impl HasContext for SaveUserPreferencesRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InspectLegacySettingsRequest {
    context: RequestContext,
    #[serde(default)]
    legacy_settings: Option<LegacySettings>,
}

impl HasContext for InspectLegacySettingsRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MigrateLegacySettingsRequest {
    context: RequestContext,
    legacy_settings: LegacySettings,
    #[serde(default)]
    migration_options: Value,
    #[serde(default)]
    dry_run: bool,
}

impl HasContext for MigrateLegacySettingsRequest {
    fn context(&self) -> &RequestContext { &self.context }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ListAccountsRequest {
    context: RequestContext,
    #[serde(default)]
    snapshot: Option<Vec<MailAccountSummary>>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for ListAccountsRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetAccountStatusRequest {
    context: RequestContext,
    account_id: String,
    #[serde(default)]
    snapshot: Option<MailAccountStatus>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for GetAccountStatusRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ListMailboxesRequest {
    context: RequestContext,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    snapshot: Option<Vec<Mailbox>>,
    #[serde(default)]
    include_favorites: bool,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for ListMailboxesRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReadEmailsRequest {
    context: RequestContext,
    #[serde(default)]
    account_id: Option<String>,
    mailbox_id: String,
    #[serde(default)]
    pagination: Option<Pagination>,
    #[serde(default)]
    sort: Option<SortSpec>,
    #[serde(default)]
    filters: SearchFilters,
    #[serde(default)]
    thread_view: bool,
    #[serde(default)]
    focused_inbox: bool,
    #[serde(default)]
    snapshot: Option<EmailPageSnapshot>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for ReadEmailsRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchEmailsRequest {
    context: RequestContext,
    query: String,
    #[serde(default)]
    pagination: Option<Pagination>,
    #[serde(default)]
    filters: SearchFilters,
    #[serde(default)]
    snapshot: Option<SearchSnapshot>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for SearchEmailsRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetEmailRequest {
    context: RequestContext,
    email_id: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    snapshot: Option<EmailMessage>,
    #[serde(default = "default_true")]
    mark_read: bool,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for GetEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetThreadRequest {
    context: RequestContext,
    thread_id: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    snapshot: Option<Vec<EmailMessage>>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for GetThreadRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MarkReadRequest {
    context: RequestContext,
    email_ids: Vec<String>,
    read: bool,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    mailbox_id: Option<String>,
}

impl HasContext for MarkReadRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FlagEmailRequest {
    context: RequestContext,
    email_ids: Vec<String>,
    flagged: bool,
    #[serde(default)]
    account_id: Option<String>,
}

impl HasContext for FlagEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DraftEmailRequest {
    context: RequestContext,
    draft: DraftDocument,
    #[serde(default)]
    autocomplete_recipients: bool,
}

impl HasContext for DraftEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateDraftRequest {
    context: RequestContext,
    draft: DraftDocument,
}

impl HasContext for UpdateDraftRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscardDraftRequest {
    context: RequestContext,
    draft_id: String,
    #[serde(default)]
    account_id: Option<String>,
}

impl HasContext for DiscardDraftRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SendEmailRequest {
    context: RequestContext,
    draft_id: String,
    account_id: String,
    #[serde(default)]
    undo_window_seconds: Option<u32>,
    #[serde(default)]
    notify: bool,
}

impl HasContext for SendEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReplyEmailRequest {
    context: RequestContext,
    email_id: String,
    account_id: String,
    #[serde(default)]
    body_html: Option<String>,
    #[serde(default)]
    body_text: Option<String>,
    #[serde(default)]
    include_original: bool,
    #[serde(default)]
    reply_all: bool,
    #[serde(default)]
    send_now: bool,
    #[serde(default)]
    undo_window_seconds: Option<u32>,
}

impl HasContext for ReplyEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ForwardEmailRequest {
    context: RequestContext,
    email_id: String,
    account_id: String,
    to: Vec<EmailAddress>,
    #[serde(default)]
    cc: Vec<EmailAddress>,
    #[serde(default)]
    bcc: Vec<EmailAddress>,
    #[serde(default)]
    note_html: Option<String>,
    #[serde(default)]
    note_text: Option<String>,
    #[serde(default)]
    send_now: bool,
    #[serde(default)]
    undo_window_seconds: Option<u32>,
}

impl HasContext for ForwardEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MoveEmailRequest {
    context: RequestContext,
    email_ids: Vec<String>,
    destination_mailbox_id: String,
    #[serde(default)]
    account_id: Option<String>,
}

impl HasContext for MoveEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteEmailRequest {
    context: RequestContext,
    email_ids: Vec<String>,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    permanent: bool,
}

impl HasContext for DeleteEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArchiveEmailRequest {
    context: RequestContext,
    email_ids: Vec<String>,
    #[serde(default)]
    account_id: Option<String>,
}

impl HasContext for ArchiveEmailRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CreateFolderRequest {
    context: RequestContext,
    account_id: String,
    name: String,
    #[serde(default)]
    parent_mailbox_id: Option<String>,
    #[serde(default)]
    favorite: bool,
}

impl HasContext for CreateFolderRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RenameFolderRequest {
    context: RequestContext,
    account_id: String,
    mailbox_id: String,
    new_name: String,
}

impl HasContext for RenameFolderRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteFolderRequest {
    context: RequestContext,
    account_id: String,
    mailbox_id: String,
    #[serde(default)]
    force: bool,
}

impl HasContext for DeleteFolderRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetUnreadCountRequest {
    context: RequestContext,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    snapshot: Option<Vec<UnreadCount>>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for GetUnreadCountRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CreateRuleRequest {
    context: RequestContext,
    account_id: String,
    rule: MailRule,
}

impl HasContext for CreateRuleRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ListRulesRequest {
    context: RequestContext,
    account_id: String,
    #[serde(default)]
    snapshot: Option<Vec<MailRule>>,
    #[serde(default)]
    refresh: bool,
}

impl HasContext for ListRulesRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteRuleRequest {
    context: RequestContext,
    account_id: String,
    rule_id: String,
}

impl HasContext for DeleteRuleRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScheduleSendRequest {
    context: RequestContext,
    draft_id: String,
    account_id: String,
    scheduled_for: i64,
    #[serde(default)]
    undo_window_seconds: Option<u32>,
}

impl HasContext for ScheduleSendRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CancelScheduledSendRequest {
    context: RequestContext,
    job_id: String,
    account_id: String,
    #[serde(default)]
    draft_id: Option<String>,
}

impl HasContext for CancelScheduledSendRequest {
    fn context(&self) -> &RequestContext {
        &self.context
    }
}

#[derive(Serialize)]
struct QueryCollection<T: Serialize> {
    snapshot: Vec<T>,
    total_count: usize,
    refresh_requested: bool,
}

#[derive(Serialize)]
struct ReadEmailsData {
    mailbox_id: String,
    page: Pagination,
    sort: SortSpec,
    filters: SearchFilters,
    thread_view: bool,
    focused_inbox: bool,
    snapshot: EmailPageSnapshot,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct SearchEmailsData {
    query: String,
    page: Pagination,
    filters: SearchFilters,
    snapshot: SearchSnapshot,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct ThreadData {
    thread_id: String,
    messages: Vec<EmailMessage>,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct MutationAck {
    mutation: &'static str,
    resource_ids: Vec<String>,
    applied: bool,
}

#[derive(Serialize)]
struct DraftMutationData {
    draft: DraftDocument,
    suggested_draft_id: String,
}

#[derive(Serialize)]
struct SendPlan {
    draft_id: String,
    account_id: String,
    undo_window_seconds: u32,
    notify: bool,
}

#[derive(Serialize)]
struct ReplyPlan {
    email_id: String,
    account_id: String,
    send_now: bool,
    undo_window_seconds: u32,
}

#[derive(Serialize)]
struct ForwardPlan {
    email_id: String,
    account_id: String,
    recipient_count: usize,
    send_now: bool,
    undo_window_seconds: u32,
}

#[derive(Serialize)]
struct FolderMutationData {
    mailbox_id: String,
    account_id: String,
    name: String,
}

#[derive(Serialize)]
struct UnreadCountData {
    counts: Vec<UnreadCount>,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct SchedulePlan {
    job_id: String,
    draft_id: String,
    account_id: String,
    scheduled_for: i64,
    undo_window_seconds: u32,
}

#[derive(Serialize)]
struct FieldIssue {
    field_path: String,
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct AccountValidationData {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    normalized_account: Option<UserMailAccountSetup>,
    field_errors: Vec<FieldIssue>,
    warnings: Vec<WarningMessage>,
    next_required_action: &'static str,
}

#[derive(Serialize)]
struct ConnectionTestPlanData {
    account_id: String,
    test_id: String,
    steps: Vec<String>,
    requires_host_execution: bool,
}

#[derive(Serialize)]
struct AccountSetupCompletionData {
    account: MailAccountSummary,
    status: String,
    requires_reconnect: bool,
    requires_sync: bool,
}

#[derive(Serialize)]
struct OAuthBeginData {
    provider: String,
    authorization_ref_present: bool,
    status: &'static str,
    next_route: String,
}

#[derive(Serialize)]
struct OAuthCompletionData {
    account: MailAccountSummary,
    identity: MailIdentity,
    requires_sync: bool,
}

#[derive(Serialize)]
struct OAuthDisconnectData {
    account_id: String,
    credential_state: &'static str,
    disabled: bool,
}

#[derive(Serialize)]
struct AccountRemovalData {
    account_id: String,
    status: &'static str,
    cleanup_planned: Vec<&'static str>,
}

#[derive(Serialize)]
struct IdentityListData {
    identities: Vec<MailIdentity>,
    signatures: Vec<MailSignature>,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct IdentityMutationData {
    identity: MailIdentity,
    verification_required: bool,
}

#[derive(Serialize)]
struct IdentityDeleteData {
    identity_id: String,
    removed: bool,
    default_reassigned_to: Option<String>,
}

#[derive(Serialize)]
struct SignatureMutationData {
    signature: MailSignature,
}

#[derive(Serialize)]
struct SignatureDeleteData {
    signature_id: String,
    removed: bool,
    identity_updates: Vec<Value>,
}

#[derive(Serialize)]
struct UserPreferencesData {
    preferences: UserMailPreferences,
    defaults_applied: bool,
    needs_host_refresh: bool,
}

#[derive(Serialize)]
struct UserPreferencesMutationData {
    preferences: UserMailPreferences,
    changed_groups: Vec<&'static str>,
}

#[derive(Serialize)]
struct LegacyInspectionData {
    accounts_detected: usize,
    preferences_detected: Vec<&'static str>,
    credential_risks: Vec<&'static str>,
    migration_preview: Value,
    requires_user_reconnect: Vec<String>,
}

#[derive(Serialize)]
struct LegacyMigrationData {
    migrated_accounts: Vec<Value>,
    migrated_preferences: Value,
    migration_map: Value,
    requires_reconnect: Vec<String>,
    obsolete_keys_removed: Vec<&'static str>,
}

fn default_true() -> bool {
    true
}

fn parse_string_from_raw(ptr: *const u8, len: usize) -> Result<String, ErrorResponse> {
    if ptr.is_null() || len == 0 {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "Expected a non-empty UTF-8 JSON request payload",
        ));
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        ErrorResponse::new("InvalidInput", "Request payload was not valid UTF-8")
    })
}

fn parse_request<T: DeserializeOwned>(ptr: *const u8, len: usize) -> Result<T, ErrorResponse> {
    let payload = parse_string_from_raw(ptr, len)?;
    serde_json::from_str(&payload).map_err(|error| {
        ErrorResponse::new("InvalidInput", format!("Request payload was not valid JSON: {error}"))
    })
}

fn write_json_response<T: Serialize>(response: &T) -> *mut u8 {
    let serialized = match serde_json::to_string(response) {
        Ok(value) => value,
        Err(error) => {
            let fallback = AppResult::<Value>::Err {
                err: ErrorResponse::new(
                    "InternalError",
                    format!("Failed to serialize response: {error}"),
                ),
            };
            serde_json::to_string(&fallback).unwrap_or_else(|_| {
                "{\"err\":{\"code\":\"InternalError\",\"message\":\"Response serialization failed\"}}"
                    .to_string()
            })
        }
    };

    match CString::new(serialized) {
        Ok(value) => value.into_raw() as *mut u8,
        Err(_) => CString::new("{\"err\":{\"code\":\"InternalError\",\"message\":\"Response contained interior null bytes\"}}")
            .expect("static JSON literal is valid")
            .into_raw() as *mut u8,
    }
}

fn ensure_non_empty(field: &str, value: &str) -> Result<(), ErrorResponse> {
    if value.trim().is_empty() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field} must not be empty"),
        ));
    }

    Ok(())
}

fn contains_ascii_control(value: &str) -> bool {
    value.chars().any(|character| character.is_ascii_control())
}

fn ensure_host_string(field: &str, value: &str, max_len: usize) -> Result<(), ErrorResponse> {
    ensure_non_empty(field, value)?;
    let trimmed = value.trim();
    if trimmed.len() != value.len() || trimmed.len() > max_len || contains_ascii_control(trimmed) {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field} contains unsupported characters or exceeds {max_len} characters"),
        ));
    }
    Ok(())
}

fn is_safe_ref_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | ':' | '@')
}

fn validate_host_id(field: &str, value: &str) -> Result<(), ErrorResponse> {
    ensure_host_string(field, value, MAX_HOST_ID_LENGTH)?;
    if !value.chars().all(is_safe_ref_char) {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field} contains unsupported characters"),
        ));
    }
    Ok(())
}

fn validate_optional_host_id(field: &str, value: &Option<String>) -> Result<(), ErrorResponse> {
    if let Some(value) = value {
        validate_host_id(field, value)?;
    }
    Ok(())
}

fn is_opaque_secret_ref(value: &str) -> bool {
    const PREFIXES: [&str; 4] = [
        "host-secure-field:",
        "host-secret:",
        "oauth-connection:",
        "authorization:",
    ];
    PREFIXES.iter().any(|prefix| {
        value.strip_prefix(prefix).is_some_and(|id| {
            !id.is_empty()
                && id.len() <= MAX_HOST_ID_LENGTH
                && id.chars().all(is_safe_ref_char)
                && !contains_ascii_control(id)
        })
    })
}

fn validate_secret_ref(field: &str, value: &Option<String>) -> Result<(), ErrorResponse> {
    if let Some(value) = value {
        ensure_host_string(field, value, MAX_HOST_ID_LENGTH + 32)?;
        if !is_opaque_secret_ref(value) {
            return Err(ErrorResponse::new(
                "InvalidInput",
                format!("{field} must be an opaque host-managed reference"),
            ));
        }
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), ErrorResponse> {
    let trimmed = email.trim();
    let invalid = trimmed.len() != email.len()
        || trimmed.len() > 320
        || trimmed.chars().any(|character| character.is_ascii_control() || character.is_whitespace())
        || trimmed.matches('@').count() != 1;
    if invalid {
        return Err(ErrorResponse::new(
            "InvalidRecipient",
            "Invalid email address",
        ));
    }

    let (local, domain) = trimmed.split_once('@').unwrap_or(("", ""));
    let local_ok = !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '/' | '=' | '?' | '^' | '_' | '`' | '{' | '|' | '}' | '~' | '.')
        });
    let domain_ok = !domain.is_empty()
        && domain.len() <= 255
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|character| character.is_ascii_alphanumeric() || character == '-')
        });
    if !local_ok || !domain_ok {
        return Err(ErrorResponse::new("InvalidRecipient", "Invalid email address"));
    }

    Ok(())
}

fn validate_addresses(field: &str, addresses: &[EmailAddress]) -> Result<(), ErrorResponse> {
    for address in addresses {
        ensure_non_empty(field, &address.email)?;
        validate_email(&address.email)?;
    }

    Ok(())
}

fn validate_pagination(pagination: Option<Pagination>) -> Result<Pagination, ErrorResponse> {
    let page = pagination.unwrap_or(Pagination {
        offset: 0,
        limit: DEFAULT_PAGE_SIZE,
    });
    if page.limit == 0 || page.limit > MAX_PAGE_SIZE {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("pagination.limit must be between 1 and {MAX_PAGE_SIZE}"),
        ));
    }

    Ok(page)
}

fn validate_sort(sort: Option<SortSpec>) -> Result<SortSpec, ErrorResponse> {
    let sort = sort.unwrap_or(SortSpec {
        field: "received_at".to_string(),
        direction: "desc".to_string(),
    });
    match sort.field.as_str() {
        "received_at" | "sent_at" | "subject" | "from" | "importance" | "is_read" | "is_flagged" => {}
        _ => return Err(ErrorResponse::new("InvalidInput", "sort.field is not supported")),
    }
    match sort.direction.as_str() {
        "asc" | "desc" => Ok(sort),
        _ => Err(ErrorResponse::new(
            "InvalidInput",
            "sort.direction must be either 'asc' or 'desc'",
        )),
    }
}

fn validate_message_ids(ids: &[String]) -> Result<(), ErrorResponse> {
    if ids.is_empty() || ids.len() > MAX_BATCH_IDS {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("email_ids must contain between 1 and {MAX_BATCH_IDS} items"),
        ));
    }
    for id in ids {
        validate_host_id("email_ids[]", id)?;
    }
    Ok(())
}

fn validate_search_text(field: &str, value: &Option<String>) -> Result<(), ErrorResponse> {
    if let Some(value) = value {
        if value.len() > MAX_FILTER_TEXT_LENGTH || contains_ascii_control(value) {
            return Err(ErrorResponse::new(
                "InvalidInput",
                format!("{field} exceeds limits or contains control characters"),
            ));
        }
    }
    Ok(())
}

fn validate_id_list(field: &str, values: &Option<Vec<String>>) -> Result<(), ErrorResponse> {
    if let Some(values) = values {
        if values.len() > MAX_FILTER_IDS {
            return Err(ErrorResponse::new(
                "InvalidInput",
                format!("{field} must contain at most {MAX_FILTER_IDS} items"),
            ));
        }
        for value in values {
            validate_host_id(field, value)?;
        }
    }
    Ok(())
}

fn validate_filters(filters: &SearchFilters) -> Result<(), ErrorResponse> {
    validate_search_text("filters.from", &filters.from)?;
    validate_search_text("filters.to", &filters.to)?;
    validate_search_text("filters.subject_contains", &filters.subject_contains)?;
    validate_search_text("filters.body_contains", &filters.body_contains)?;
    validate_id_list("filters.account_ids[]", &filters.account_ids)?;
    validate_id_list("filters.mailbox_ids[]", &filters.mailbox_ids)?;
    if let Some(importance) = &filters.importance {
        validate_importance(importance)?;
    }
    Ok(())
}

fn validate_query(query: &str) -> Result<(), ErrorResponse> {
    ensure_host_string("query", query, MAX_SEARCH_QUERY_LENGTH)
}

fn validate_importance(importance: &str) -> Result<(), ErrorResponse> {
    match importance {
        "low" | "normal" | "high" => Ok(()),
        _ => Err(ErrorResponse::new(
            "InvalidInput",
            format!("Unsupported importance value: {importance}"),
        )),
    }
}

fn validate_compose_body(field: &str, html: &Option<String>, text: &Option<String>) -> Result<(), ErrorResponse> {
    let body_len = html.as_ref().map_or(0, String::len) + text.as_ref().map_or(0, String::len);
    if body_len == 0 {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field} must include body_html or body_text"),
        ));
    }
    if body_len > MAX_BODY_LENGTH {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field} body exceeds {MAX_BODY_LENGTH} bytes"),
        ));
    }
    Ok(())
}

fn validate_draft(draft: &DraftDocument) -> Result<(), ErrorResponse> {
    ensure_non_empty("draft.account_id", &draft.account_id)?;
    validate_addresses("draft.to", &draft.to)?;
    validate_addresses("draft.cc", &draft.cc)?;
    validate_addresses("draft.bcc", &draft.bcc)?;

    if draft.to.is_empty() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "draft.to must contain at least one recipient",
        ));
    }

    let recipient_count = draft.to.len() + draft.cc.len() + draft.bcc.len();
    if recipient_count > MAX_RECIPIENTS {
        return Err(ErrorResponse::new(
            "TooManyRecipients",
            format!("A message cannot exceed {MAX_RECIPIENTS} recipients"),
        ));
    }

    if draft.subject.len() > MAX_SUBJECT_LENGTH {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("draft.subject must be at most {MAX_SUBJECT_LENGTH} characters"),
        ));
    }

    validate_compose_body("draft", &draft.body_html, &draft.body_text)?;

    if let Some(importance) = &draft.importance {
        validate_importance(importance)?;
    }

    Ok(())
}

fn require_permission(context: &RequestContext, required: Permission) -> Result<(), ErrorResponse> {
    let actual = context.permission()?;
    if actual.satisfies(required) {
        Ok(())
    } else {
        Err(ErrorResponse::new(
            "PermissionDenied",
            format!("Operation requires {:?} permission", required),
        ))
    }
}

fn require_effects(context: &RequestContext, required: &[&'static str]) -> Result<(), ErrorResponse> {
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|effect| !context.available_effects.iter().any(|value| value == effect))
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(
            ErrorResponse::new(
                "EffectUnavailable",
                format!("Required host effects are unavailable: {}", missing.join(", ")),
            )
            .with_details(json!({ "missing_effects": missing })),
        )
    }
}

fn host_effect(
    context: &RequestContext,
    operation: &'static str,
    effect: &'static str,
    intent: &'static str,
    payload: Value,
) -> HostEffect {
    let seed = serde_json::to_string(&payload).unwrap_or_else(|_| effect.to_string());
    HostEffect {
        effect,
        intent,
        payload,
        idempotency_key: context.idempotency_key(operation, effect, &seed),
    }
}

fn snapshot_warning(refresh: bool, has_snapshot: bool) -> Vec<WarningMessage> {
    if refresh && has_snapshot {
        vec![WarningMessage {
            code: "STALE_SNAPSHOT",
            message: "A host refresh was requested; the returned snapshot may be stale until effects complete".to_string(),
        }]
    } else {
        Vec::new()
    }
}

fn hash_text(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn suggested_draft_id(context: &RequestContext, draft: &DraftDocument) -> String {
    draft.draft_id.clone().unwrap_or_else(|| {
        let seed = serde_json::to_string(draft).unwrap_or_else(|_| draft.account_id.clone());
        format!("draft_{}", context.idempotency_key("draft", EFFECT_MAIL_STORE_WRITE, &seed).replace(':', "_"))
    })
}

fn clamp_undo_window(value: Option<u32>) -> u32 {
    match value.unwrap_or(10) {
        0 | 5 | 10 | 30 => value.unwrap_or(10),
        candidate if candidate < MAX_ALLOWED_UNDO_SECONDS => 10,
        _ => 30,
    }
}

fn handle_operation<Req, Res, F>(
    input_ptr: *const u8,
    input_len: usize,
    required_permission: Permission,
    required_effects_list: &[&'static str],
    handler: F,
) -> *mut u8
where
    Req: DeserializeOwned + HasContext,
    Res: Serialize,
    F: FnOnce(Req) -> Result<OperationResponse<Res>, ErrorResponse>,
{
    let response: AppResult<OperationResponse<Res>> = match parse_request::<Req>(input_ptr, input_len) {
        Ok(request) => {
            let context = request.context();
            match require_permission(context, required_permission)
                .and_then(|_| require_effects(context, required_effects_list))
            {
                Ok(()) => match handler(request) {
                    Ok(result) => AppResult::Ok { ok: result },
                    Err(err) => AppResult::Err { err },
                },
                Err(err) => AppResult::Err { err },
            }
        }
        Err(err) => AppResult::Err { err },
    };

    write_json_response(&response)
}

fn build_query_collection<T: Serialize>(
    snapshot: Option<Vec<T>>,
    refresh: bool,
) -> QueryCollection<T> {
    let rows = snapshot.unwrap_or_default();
    let total_count = rows.len();
    QueryCollection {
        snapshot: rows,
        total_count,
        refresh_requested: refresh,
    }
}

fn handle_list_accounts(
    request: ListAccountsRequest,
) -> Result<OperationResponse<QueryCollection<MailAccountSummary>>, ErrorResponse> {
    let data = build_query_collection(request.snapshot.clone(), request.refresh);
    let mut host_effects = Vec::new();
    if request.refresh || request.snapshot.is_none() {
        host_effects.push(host_effect(
            &request.context,
            "list_accounts",
            EFFECT_ACCOUNT_CREDENTIAL_READ,
            "Load configured mail account metadata from host settings",
            json!({ "user_id": request.context.user_id }),
        ));
    }

    Ok(OperationResponse {
        operation: "list_accounts",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_get_account_status(
    request: GetAccountStatusRequest,
) -> Result<OperationResponse<MailAccountStatus>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    let data = request.snapshot.clone().unwrap_or(MailAccountStatus {
        account_id: request.account_id.clone(),
        connection_state: "unknown".to_string(),
        last_sync_at: None,
        unread_count: None,
        error_message: None,
    });

    let mut host_effects = Vec::new();
    if request.refresh || request.snapshot.is_none() {
        host_effects.push(host_effect(
            &request.context,
            "get_account_status",
            EFFECT_IMAP_SYNC,
            "Refresh mailbox sync and connection health",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "get_account_status",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_list_mailboxes(
    request: ListMailboxesRequest,
) -> Result<OperationResponse<QueryCollection<Mailbox>>, ErrorResponse> {
    if let Some(account_id) = &request.account_id {
        validate_host_id("account_id", account_id)?;
    }

    let data = build_query_collection(request.snapshot.clone(), request.refresh);
    let mut host_effects = Vec::new();
    if request.refresh || request.snapshot.is_none() {
        host_effects.push(host_effect(
            &request.context,
            "list_mailboxes",
            EFFECT_IMAP_FETCH,
            "List folders for the selected account or unified inbox",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "include_favorites": request.include_favorites
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "list_mailboxes",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_read_emails(
    request: ReadEmailsRequest,
) -> Result<OperationResponse<ReadEmailsData>, ErrorResponse> {
    ensure_non_empty("mailbox_id", &request.mailbox_id)?;
    validate_host_id("mailbox_id", &request.mailbox_id)?;
    if let Some(account_id) = &request.account_id {
        validate_host_id("account_id", account_id)?;
    }
    let page = validate_pagination(request.pagination.clone())?;
    let sort = validate_sort(request.sort.clone())?;
    validate_filters(&request.filters)?;

    let data = ReadEmailsData {
        mailbox_id: request.mailbox_id.clone(),
        page,
        sort,
        filters: request.filters.clone(),
        thread_view: request.thread_view,
        focused_inbox: request.focused_inbox,
        snapshot: request.snapshot.clone().unwrap_or(EmailPageSnapshot {
            messages: Vec::new(),
            total_count: 0,
            unread_count: 0,
        }),
        needs_host_refresh: request.refresh || request.snapshot.is_none(),
    };

    let mut host_effects = Vec::new();
    if data.needs_host_refresh {
        host_effects.push(host_effect(
            &request.context,
            "read_emails",
            EFFECT_IMAP_FETCH,
            "Fetch a paginated mail listing for a mailbox",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "mailbox_id": request.mailbox_id,
                "offset": data.page.offset,
                "limit": data.page.limit,
                "sort": data.sort,
                "filters": data.filters,
                "thread_view": data.thread_view,
                "focused_inbox": data.focused_inbox
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "read_emails",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_search_emails(
    request: SearchEmailsRequest,
) -> Result<OperationResponse<SearchEmailsData>, ErrorResponse> {
    validate_query(&request.query)?;
    let page = validate_pagination(request.pagination.clone())?;
    validate_filters(&request.filters)?;
    let data = SearchEmailsData {
        query: request.query.clone(),
        page,
        filters: request.filters.clone(),
        snapshot: request.snapshot.clone().unwrap_or(SearchSnapshot {
            results: Vec::new(),
            total_count: 0,
        }),
        needs_host_refresh: request.refresh || request.snapshot.is_none(),
    };

    let mut host_effects = Vec::new();
    if data.needs_host_refresh {
        host_effects.push(host_effect(
            &request.context,
            "search_emails",
            EFFECT_IMAP_FETCH,
            "Run a host-backed search across one or more mailboxes",
            json!({
                "user_id": request.context.user_id,
                "query": data.query,
                "offset": data.page.offset,
                "limit": data.page.limit,
                "filters": data.filters
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "search_emails",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_get_email(
    request: GetEmailRequest,
) -> Result<OperationResponse<EmailMessage>, ErrorResponse> {
    validate_host_id("email_id", &request.email_id)?;
    if let Some(account_id) = &request.account_id {
        validate_host_id("account_id", account_id)?;
    }

    let data = request.snapshot.clone().unwrap_or(EmailMessage {
        email_id: request.email_id.clone(),
        account_id: request.account_id.clone().unwrap_or_else(|| "unified".to_string()),
        mailbox_id: "INBOX".to_string(),
        thread_id: None,
        from: EmailAddress {
            email: "unknown@example.invalid".to_string(),
            display_name: None,
        },
        to: Vec::new(),
        cc: Vec::new(),
        bcc: Vec::new(),
        subject: String::new(),
        preview: None,
        body_html: None,
        body_text: None,
        received_at: None,
        sent_at: None,
        is_read: !request.mark_read,
        is_flagged: false,
        has_attachments: false,
        importance: "normal".to_string(),
        categories: Vec::new(),
        attachments: Vec::new(),
    });

    let mut host_effects = Vec::new();
    if request.refresh || request.snapshot.is_none() {
        host_effects.push(host_effect(
            &request.context,
            "get_email",
            EFFECT_IMAP_FETCH,
            "Fetch a full email body and attachment metadata",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "email_id": request.email_id,
                "mark_read": request.mark_read
            }),
        ));
    }
    if request.mark_read {
        host_effects.push(host_effect(
            &request.context,
            "get_email",
            EFFECT_MAIL_STORE_WRITE,
            "Persist read status after opening an email",
            json!({
                "user_id": request.context.user_id,
                "email_ids": [request.email_id],
                "is_read": true
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "get_email",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_get_thread(
    request: GetThreadRequest,
) -> Result<OperationResponse<ThreadData>, ErrorResponse> {
    validate_host_id("thread_id", &request.thread_id)?;
    if let Some(account_id) = &request.account_id {
        validate_host_id("account_id", account_id)?;
    }
    let data = ThreadData {
        thread_id: request.thread_id.clone(),
        messages: request.snapshot.clone().unwrap_or_default(),
        needs_host_refresh: request.refresh || request.snapshot.is_none(),
    };
    let mut host_effects = Vec::new();
    if data.needs_host_refresh {
        host_effects.push(host_effect(
            &request.context,
            "get_thread",
            EFFECT_IMAP_FETCH,
            "Fetch an ordered conversation thread",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "thread_id": request.thread_id
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "get_thread",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_mark_read(
    request: MarkReadRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_message_ids(&request.email_ids)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    validate_optional_host_id("mailbox_id", &request.mailbox_id)?;
    let data = MutationAck {
        mutation: if request.read { "mark_read" } else { "mark_unread" },
        resource_ids: request.email_ids.clone(),
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "mark_read",
        EFFECT_MAIL_STORE_WRITE,
        "Update read state for one or more emails",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "mailbox_id": request.mailbox_id,
            "email_ids": request.email_ids,
            "is_read": request.read
        }),
    )];

    Ok(OperationResponse {
        operation: "mark_read",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_flag_email(
    request: FlagEmailRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_message_ids(&request.email_ids)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    let data = MutationAck {
        mutation: if request.flagged { "flag_email" } else { "unflag_email" },
        resource_ids: request.email_ids.clone(),
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "flag_email",
        EFFECT_MAIL_STORE_WRITE,
        "Toggle flagged status for one or more emails",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_ids": request.email_ids,
            "flagged": request.flagged
        }),
    )];

    Ok(OperationResponse {
        operation: "flag_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_draft_email(
    request: DraftEmailRequest,
) -> Result<OperationResponse<DraftMutationData>, ErrorResponse> {
    validate_draft(&request.draft)?;
    let draft_id = suggested_draft_id(&request.context, &request.draft);
    let data = DraftMutationData {
        draft: request.draft.clone(),
        suggested_draft_id: draft_id.clone(),
    };
    let mut host_effects = vec![host_effect(
        &request.context,
        "draft_email",
        EFFECT_MAIL_STORE_WRITE,
        "Persist a draft in the host mail store",
        json!({
            "user_id": request.context.user_id,
            "draft_id": draft_id,
            "draft": request.draft
        }),
    )];
    if request.autocomplete_recipients {
        host_effects.push(host_effect(
            &request.context,
            "draft_email",
            EFFECT_CONTACT_LOOKUP,
            "Resolve recipients for compose autocomplete",
            json!({
                "user_id": request.context.user_id,
                "to": data.draft.to,
                "cc": data.draft.cc,
                "bcc": data.draft.bcc
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "draft_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_update_draft(
    request: UpdateDraftRequest,
) -> Result<OperationResponse<DraftMutationData>, ErrorResponse> {
    validate_draft(&request.draft)?;
    let draft_id = suggested_draft_id(&request.context, &request.draft);
    let data = DraftMutationData {
        draft: request.draft.clone(),
        suggested_draft_id: draft_id.clone(),
    };
    let host_effects = vec![host_effect(
        &request.context,
        "update_draft",
        EFFECT_MAIL_STORE_WRITE,
        "Update an existing draft in the host mail store",
        json!({
            "user_id": request.context.user_id,
            "draft_id": draft_id,
            "draft": request.draft
        }),
    )];

    Ok(OperationResponse {
        operation: "update_draft",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_discard_draft(
    request: DiscardDraftRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_host_id("draft_id", &request.draft_id)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    let data = MutationAck {
        mutation: "discard_draft",
        resource_ids: vec![request.draft_id.clone()],
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "discard_draft",
        EFFECT_MAIL_STORE_DELETE,
        "Delete a draft from the host mail store",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "draft_id": request.draft_id
        }),
    )];

    Ok(OperationResponse {
        operation: "discard_draft",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_send_email(
    request: SendEmailRequest,
) -> Result<OperationResponse<SendPlan>, ErrorResponse> {
    validate_host_id("draft_id", &request.draft_id)?;
    validate_host_id("account_id", &request.account_id)?;
    let data = SendPlan {
        draft_id: request.draft_id.clone(),
        account_id: request.account_id.clone(),
        undo_window_seconds: clamp_undo_window(request.undo_window_seconds),
        notify: request.notify,
    };
    let mut host_effects = vec![host_effect(
        &request.context,
        "send_email",
        EFFECT_SMTP_SEND,
        "Send an existing draft via the selected account",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "draft_id": request.draft_id,
            "undo_window_seconds": data.undo_window_seconds
        }),
    )];
    host_effects.push(host_effect(
        &request.context,
        "send_email",
        EFFECT_MAIL_STORE_WRITE,
        "Move the sent draft into Sent Items and update client state",
        json!({
            "user_id": request.context.user_id,
            "account_id": data.account_id,
            "draft_id": data.draft_id,
            "target_mailbox": "SENT"
        }),
    ));
    if data.notify {
        host_effects.push(host_effect(
            &request.context,
            "send_email",
            EFFECT_NOTIFY_USER,
            "Show a human-facing send confirmation",
            json!({
                "user_id": request.context.user_id,
                "title": "Email queued",
                "body": format!("Draft {} was queued for delivery", request.draft_id)
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "send_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_reply_email(
    request: ReplyEmailRequest,
) -> Result<OperationResponse<ReplyPlan>, ErrorResponse> {
    validate_host_id("email_id", &request.email_id)?;
    validate_host_id("account_id", &request.account_id)?;
    validate_compose_body("reply_email", &request.body_html, &request.body_text)?;

    let data = ReplyPlan {
        email_id: request.email_id.clone(),
        account_id: request.account_id.clone(),
        send_now: request.send_now,
        undo_window_seconds: clamp_undo_window(request.undo_window_seconds),
    };
    let mut host_effects = vec![host_effect(
        &request.context,
        "reply_email",
        EFFECT_MAIL_STORE_READ,
        "Load the source email so reply headers and thread metadata stay consistent",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_id": request.email_id,
            "reply_all": request.reply_all,
            "include_original": request.include_original
        }),
    )];
    host_effects.push(host_effect(
        &request.context,
        "reply_email",
        if request.send_now { EFFECT_SMTP_SEND } else { EFFECT_MAIL_STORE_WRITE },
        if request.send_now {
            "Send the composed reply"
        } else {
            "Persist the reply as a draft"
        },
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_id": request.email_id,
            "body_html": request.body_html,
            "body_text": request.body_text,
            "reply_all": request.reply_all,
            "include_original": request.include_original,
            "undo_window_seconds": data.undo_window_seconds
        }),
    ));

    Ok(OperationResponse {
        operation: "reply_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_forward_email(
    request: ForwardEmailRequest,
) -> Result<OperationResponse<ForwardPlan>, ErrorResponse> {
    validate_host_id("email_id", &request.email_id)?;
    validate_host_id("account_id", &request.account_id)?;
    validate_addresses("to", &request.to)?;
    validate_addresses("cc", &request.cc)?;
    validate_addresses("bcc", &request.bcc)?;
    let recipient_count = request.to.len() + request.cc.len() + request.bcc.len();
    if request.to.is_empty() || recipient_count > MAX_RECIPIENTS {
        return Err(ErrorResponse::new(
            "TooManyRecipients",
            format!("Forwarding requires 1-{MAX_RECIPIENTS} recipients"),
        ));
    }
    validate_compose_body("forward_email", &request.note_html, &request.note_text)?;

    let data = ForwardPlan {
        email_id: request.email_id.clone(),
        account_id: request.account_id.clone(),
        recipient_count,
        send_now: request.send_now,
        undo_window_seconds: clamp_undo_window(request.undo_window_seconds),
    };
    let mut host_effects = vec![host_effect(
        &request.context,
        "forward_email",
        EFFECT_MAIL_STORE_READ,
        "Load the source email and attachment references for forwarding",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_id": request.email_id
        }),
    )];
    host_effects.push(host_effect(
        &request.context,
        "forward_email",
        if request.send_now { EFFECT_SMTP_SEND } else { EFFECT_MAIL_STORE_WRITE },
        if request.send_now {
            "Send the forward immediately"
        } else {
            "Persist the forward as a draft"
        },
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_id": request.email_id,
            "to": request.to,
            "cc": request.cc,
            "bcc": request.bcc,
            "note_html": request.note_html,
            "note_text": request.note_text,
            "undo_window_seconds": data.undo_window_seconds
        }),
    ));

    Ok(OperationResponse {
        operation: "forward_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_move_email(
    request: MoveEmailRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_message_ids(&request.email_ids)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    validate_host_id("destination_mailbox_id", &request.destination_mailbox_id)?;
    let data = MutationAck {
        mutation: "move_email",
        resource_ids: request.email_ids.clone(),
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "move_email",
        EFFECT_MAIL_STORE_WRITE,
        "Move messages into another mailbox",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_ids": request.email_ids,
            "destination_mailbox_id": request.destination_mailbox_id
        }),
    )];

    Ok(OperationResponse {
        operation: "move_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_delete_email(
    request: DeleteEmailRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_message_ids(&request.email_ids)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    let data = MutationAck {
        mutation: if request.permanent { "delete_email_permanent" } else { "delete_email" },
        resource_ids: request.email_ids.clone(),
        applied: false,
    };
    let effect_name = if request.permanent {
        EFFECT_MAIL_STORE_DELETE
    } else {
        EFFECT_MAIL_STORE_WRITE
    };
    let host_effects = vec![host_effect(
        &request.context,
        "delete_email",
        effect_name,
        if request.permanent {
            "Permanently delete messages"
        } else {
            "Move messages to Trash"
        },
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_ids": request.email_ids,
            "permanent": request.permanent
        }),
    )];

    Ok(OperationResponse {
        operation: "delete_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_archive_email(
    request: ArchiveEmailRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_message_ids(&request.email_ids)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    let data = MutationAck {
        mutation: "archive_email",
        resource_ids: request.email_ids.clone(),
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "archive_email",
        EFFECT_MAIL_STORE_WRITE,
        "Move messages into the Archive mailbox",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "email_ids": request.email_ids,
            "destination_mailbox_id": "ARCHIVE"
        }),
    )];

    Ok(OperationResponse {
        operation: "archive_email",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_create_folder(
    request: CreateFolderRequest,
) -> Result<OperationResponse<FolderMutationData>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_optional_host_id("parent_mailbox_id", &request.parent_mailbox_id)?;
    ensure_host_string("name", &request.name, MAX_FOLDER_NAME_LENGTH)?;
    if request.name.len() > MAX_FOLDER_NAME_LENGTH {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("Folder names must be at most {MAX_FOLDER_NAME_LENGTH} characters"),
        ));
    }
    let mailbox_id = format!(
        "fld_{}",
        hash_text(&format!("{}:{}", request.account_id, request.name.to_lowercase()))
    );
    let data = FolderMutationData {
        mailbox_id: mailbox_id.clone(),
        account_id: request.account_id.clone(),
        name: request.name.clone(),
    };
    let host_effects = vec![host_effect(
        &request.context,
        "create_folder",
        EFFECT_MAIL_STORE_WRITE,
        "Create a custom mailbox folder",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "mailbox_id": mailbox_id,
            "name": request.name,
            "parent_mailbox_id": request.parent_mailbox_id,
            "favorite": request.favorite
        }),
    )];

    Ok(OperationResponse {
        operation: "create_folder",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_rename_folder(
    request: RenameFolderRequest,
) -> Result<OperationResponse<FolderMutationData>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_host_id("mailbox_id", &request.mailbox_id)?;
    ensure_host_string("new_name", &request.new_name, MAX_FOLDER_NAME_LENGTH)?;
    if request.new_name.len() > MAX_FOLDER_NAME_LENGTH {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("Folder names must be at most {MAX_FOLDER_NAME_LENGTH} characters"),
        ));
    }
    let data = FolderMutationData {
        mailbox_id: request.mailbox_id.clone(),
        account_id: request.account_id.clone(),
        name: request.new_name.clone(),
    };
    let host_effects = vec![host_effect(
        &request.context,
        "rename_folder",
        EFFECT_MAIL_STORE_WRITE,
        "Rename an existing mailbox folder",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "mailbox_id": request.mailbox_id,
            "new_name": request.new_name
        }),
    )];

    Ok(OperationResponse {
        operation: "rename_folder",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_delete_folder(
    request: DeleteFolderRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_host_id("mailbox_id", &request.mailbox_id)?;
    let data = MutationAck {
        mutation: "delete_folder",
        resource_ids: vec![request.mailbox_id.clone()],
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "delete_folder",
        EFFECT_MAIL_STORE_DELETE,
        "Delete a mailbox folder after optional force validation",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "mailbox_id": request.mailbox_id,
            "force": request.force
        }),
    )];

    Ok(OperationResponse {
        operation: "delete_folder",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_get_unread_count(
    request: GetUnreadCountRequest,
) -> Result<OperationResponse<UnreadCountData>, ErrorResponse> {
    if let Some(account_id) = &request.account_id {
        validate_host_id("account_id", account_id)?;
    }
    let data = UnreadCountData {
        counts: request.snapshot.clone().unwrap_or_default(),
        needs_host_refresh: request.refresh || request.snapshot.is_none(),
    };
    let mut host_effects = Vec::new();
    if data.needs_host_refresh {
        host_effects.push(host_effect(
            &request.context,
            "get_unread_count",
            EFFECT_IMAP_FETCH,
            "Fetch unread counters across folders",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "get_unread_count",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_create_rule(
    request: CreateRuleRequest,
) -> Result<OperationResponse<MailRule>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_rule(&request.rule)?;
    let host_effects = vec![host_effect(
        &request.context,
        "create_rule",
        EFFECT_MAIL_STORE_WRITE,
        "Persist a server-side automation rule",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "rule": request.rule
        }),
    )];

    Ok(OperationResponse {
        operation: "create_rule",
        status: "accepted",
        data: request.rule,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_list_rules(
    request: ListRulesRequest,
) -> Result<OperationResponse<QueryCollection<MailRule>>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    let data = build_query_collection(request.snapshot.clone(), request.refresh);
    let mut host_effects = Vec::new();
    if request.refresh || request.snapshot.is_none() {
        host_effects.push(host_effect(
            &request.context,
            "list_rules",
            EFFECT_MAIL_STORE_READ,
            "Load automation rules for an account",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id
            }),
        ));
    }

    Ok(OperationResponse {
        operation: "list_rules",
        status: "accepted",
        data,
        host_effects,
        warnings: snapshot_warning(request.refresh, request.snapshot.is_some()),
    })
}

fn handle_delete_rule(
    request: DeleteRuleRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_host_id("rule_id", &request.rule_id)?;
    let data = MutationAck {
        mutation: "delete_rule",
        resource_ids: vec![request.rule_id.clone()],
        applied: false,
    };
    let host_effects = vec![host_effect(
        &request.context,
        "delete_rule",
        EFFECT_MAIL_STORE_DELETE,
        "Delete a server-side automation rule",
        json!({
            "user_id": request.context.user_id,
            "account_id": request.account_id,
            "rule_id": request.rule_id
        }),
    )];

    Ok(OperationResponse {
        operation: "delete_rule",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_schedule_send(
    request: ScheduleSendRequest,
) -> Result<OperationResponse<SchedulePlan>, ErrorResponse> {
    validate_host_id("draft_id", &request.draft_id)?;
    validate_host_id("account_id", &request.account_id)?;
    let job_id = format!(
        "job_{}",
        hash_text(&format!("{}:{}", request.draft_id, request.scheduled_for))
    );
    let data = SchedulePlan {
        job_id: job_id.clone(),
        draft_id: request.draft_id.clone(),
        account_id: request.account_id.clone(),
        scheduled_for: request.scheduled_for,
        undo_window_seconds: clamp_undo_window(request.undo_window_seconds),
    };
    let host_effects = vec![
        host_effect(
            &request.context,
            "schedule_send",
            EFFECT_SCHEDULE_JOB,
            "Queue a future send job",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "draft_id": request.draft_id,
                "job_id": job_id,
                "scheduled_for": request.scheduled_for,
                "undo_window_seconds": data.undo_window_seconds
            }),
        ),
        host_effect(
            &request.context,
            "schedule_send",
            EFFECT_MAIL_STORE_WRITE,
            "Persist scheduled-send metadata on the draft",
            json!({
                "user_id": request.context.user_id,
                "account_id": data.account_id,
                "draft_id": data.draft_id,
                "scheduled_for": data.scheduled_for,
                "job_id": data.job_id
            }),
        ),
    ];

    Ok(OperationResponse {
        operation: "schedule_send",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn handle_cancel_scheduled_send(
    request: CancelScheduledSendRequest,
) -> Result<OperationResponse<MutationAck>, ErrorResponse> {
    validate_host_id("job_id", &request.job_id)?;
    validate_host_id("account_id", &request.account_id)?;
    validate_optional_host_id("draft_id", &request.draft_id)?;
    let data = MutationAck {
        mutation: "cancel_scheduled_send",
        resource_ids: vec![request.job_id.clone()],
        applied: false,
    };
    let host_effects = vec![
        host_effect(
            &request.context,
            "cancel_scheduled_send",
            EFFECT_CANCEL_JOB,
            "Cancel a queued send job",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "job_id": request.job_id
            }),
        ),
        host_effect(
            &request.context,
            "cancel_scheduled_send",
            EFFECT_MAIL_STORE_WRITE,
            "Clear scheduled-send metadata from the draft",
            json!({
                "user_id": request.context.user_id,
                "account_id": request.account_id,
                "job_id": request.job_id,
                "draft_id": request.draft_id
            }),
        ),
    ];

    Ok(OperationResponse {
        operation: "cancel_scheduled_send",
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    })
}

fn default_preferences() -> UserMailPreferences {
    UserMailPreferences {
        reading: json!({
            "preview_pane": "right",
            "thread_view": true,
            "focused_inbox": true,
            "page_size": DEFAULT_PAGE_SIZE,
            "list_density": "comfortable",
            "remote_content": "ask"
        }),
        compose: json!({
            "undo_send_delay_seconds": 10,
            "default_format": "html",
            "default_sender_strategy": "last_used",
            "reply_quote_mode": "collapsed",
            "forward_attachments_default": false
        }),
        notifications: json!({ "enabled": true, "per_account": {} }),
        sync_defaults: json!({
            "interval_minutes": 5,
            "initial_range": "30_days",
            "download_attachments": "metadata_only",
            "offline_cache": false
        }),
    }
}

fn default_provider_capabilities() -> ProviderCapabilities {
    ProviderCapabilities {
        providers: vec![
            json!({ "provider": "google_oauth", "label": "Google Workspace", "auth_method": "oauth2", "status": "host_connector" }),
            json!({ "provider": "microsoft_oauth", "label": "Microsoft 365", "auth_method": "oauth2", "status": "host_connector" }),
        ],
        manual_imap_smtp: json!({
            "enabled": true,
            "security_modes": ["ssl_tls", "starttls"],
            "default_imap_port": 993,
            "default_smtp_port": 587,
            "allow_receive_only": true
        }),
        policy: json!({
            "allow_manual_imap_smtp": true,
            "allowed_oauth_providers": ["google_oauth", "microsoft_oauth"],
            "allow_insecure_transport": false,
            "min_sync_interval_minutes": 5,
            "max_recipients": MAX_RECIPIENTS,
            "attachment_max_bytes": 25_000_000,
            "connection_test_modes": ["all", "incoming", "outgoing", "folder_mapping"]
        }),
        oauth_placeholders: vec!["google_oauth".to_string(), "microsoft_oauth".to_string()],
    }
}

fn normalize_account(mut account: UserMailAccountSetup) -> UserMailAccountSetup {
    if account.provider.trim().is_empty() {
        account.provider = "manual_imap_smtp".to_string();
    }
    if account.auth_method.trim().is_empty() {
        account.auth_method = if account.provider.ends_with("oauth") { "oauth2" } else { "app_password" }.to_string();
    }
    if account.desired_status.is_none() {
        account.desired_status = Some("active".to_string());
    }
    if let Some(incoming) = &mut account.incoming {
        if incoming.port.is_none() {
            incoming.port = Some(993);
        }
        if incoming.security.is_none() {
            incoming.security = Some("ssl_tls".to_string());
        }
    }
    if let Some(outgoing) = &mut account.outgoing {
        if outgoing.port.is_none() {
            outgoing.port = Some(587);
        }
        if outgoing.security.is_none() {
            outgoing.security = Some("starttls".to_string());
        }
    }
    account
}

fn push_field_error(errors: &mut Vec<FieldIssue>, field_path: &str, message: impl Into<String>) {
    errors.push(FieldIssue {
        field_path: field_path.to_string(),
        code: "InvalidInput",
        message: message.into(),
    });
}

fn push_validation_result(errors: &mut Vec<FieldIssue>, field_path: &str, result: Result<(), ErrorResponse>) {
    if let Err(error) = result {
        push_field_error(errors, field_path, error.message);
    }
}

fn is_valid_host_name(value: &str) -> bool {
    value.len() <= 255
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
}

fn redact_secret_refs(account: &mut UserMailAccountSetup) {
    if let Some(incoming) = &mut account.incoming {
        incoming.secret_input_ref = None;
        incoming.secret_ref = None;
    }
    if let Some(outgoing) = &mut account.outgoing {
        outgoing.secret_input_ref = None;
        outgoing.secret_ref = None;
    }
    if let Some(oauth) = &mut account.oauth {
        oauth.connection_ref = None;
        oauth.redirect_state_ref = None;
    }
}

fn has_secret_ref(config: &ServerConfig) -> bool {
    config.secret_input_ref.as_ref().is_some_and(|value| !value.trim().is_empty())
        || config.secret_ref.as_ref().is_some_and(|value| !value.trim().is_empty())
}

fn validate_server_config(errors: &mut Vec<FieldIssue>, field: &str, expected_protocol: &str, config: &ServerConfig) {
    if config.protocol != expected_protocol {
        push_field_error(errors, &format!("{field}.protocol"), format!("{field}.protocol must be {expected_protocol}"));
    }
    if config.host.trim().is_empty() || !is_valid_host_name(&config.host) || contains_ascii_control(&config.host) {
        push_field_error(errors, &format!("{field}.host"), format!("{field}.host must be a valid host name"));
    }
    if config.username.trim().is_empty() || config.username.len() > 320 || contains_ascii_control(&config.username) || config.username.chars().any(char::is_whitespace) {
        push_field_error(errors, &format!("{field}.username"), format!("{field}.username contains unsupported characters"));
    }
    match config.security.as_deref().unwrap_or(if expected_protocol == "imap" { "ssl_tls" } else { "starttls" }) {
        "ssl_tls" | "starttls" => {}
        "none" => push_field_error(errors, &format!("{field}.security"), "security none requires admin policy and is disabled by default"),
        _ => push_field_error(errors, &format!("{field}.security"), "security must be ssl_tls or starttls"),
    }
    if matches!(config.port, Some(0)) {
        push_field_error(errors, &format!("{field}.port"), "port must be greater than zero");
    }
    push_validation_result(errors, &format!("{field}.secret_input_ref"), validate_secret_ref(&format!("{field}.secret_input_ref"), &config.secret_input_ref));
    push_validation_result(errors, &format!("{field}.secret_ref"), validate_secret_ref(&format!("{field}.secret_ref"), &config.secret_ref));
    if let Some(path_prefix) = &config.path_prefix {
        push_validation_result(errors, &format!("{field}.path_prefix"), ensure_host_string(&format!("{field}.path_prefix"), path_prefix, 128));
    }
}

fn validate_rule_value(field_path: &str, value: &Value, allow_bool: bool) -> Result<(), ErrorResponse> {
    match value {
        Value::String(value) => ensure_host_string(field_path, value, MAX_FILTER_TEXT_LENGTH),
        Value::Bool(_) if allow_bool => Ok(()),
        Value::Array(values) if values.len() <= MAX_FILTER_IDS => {
            for item in values {
                let value = item.as_str().ok_or_else(|| ErrorResponse::new("InvalidInput", format!("{field_path} array items must be strings")))?;
                ensure_host_string(field_path, value, MAX_FILTER_TEXT_LENGTH)?;
            }
            Ok(())
        }
        _ => Err(ErrorResponse::new("InvalidInput", format!("{field_path} has unsupported value shape"))),
    }
}

fn validate_rule(rule: &MailRule) -> Result<(), ErrorResponse> {
    validate_host_id("rule.rule_id", &rule.rule_id)?;
    ensure_host_string("rule.name", &rule.name, MAX_RULE_NAME_LENGTH)?;
    if rule.conditions.is_empty() || rule.conditions.len() > MAX_RULE_CONDITIONS || rule.actions.is_empty() || rule.actions.len() > MAX_RULE_ACTIONS {
        return Err(ErrorResponse::new("InvalidInput", "Rules require bounded non-empty conditions and actions"));
    }
    for condition in &rule.conditions {
        match condition.field.as_str() {
            "from" | "to" | "cc" | "subject" | "body" | "importance" | "account_id" | "mailbox_id" | "has_attachments" | "unread" | "flagged" => {}
            _ => return Err(ErrorResponse::new("InvalidInput", "rule.conditions[].field is not supported")),
        }
        match condition.operator.as_str() {
            "equals" | "contains" | "starts_with" | "ends_with" | "exists" | "not_exists" => {}
            _ => return Err(ErrorResponse::new("InvalidInput", "rule.conditions[].operator is not supported")),
        }
        let bool_field = matches!(condition.field.as_str(), "has_attachments" | "unread" | "flagged");
        validate_rule_value("rule.conditions[].value", &condition.value, bool_field)?;
    }
    for action in &rule.actions {
        match action.action_type.as_str() {
            "move_to" | "copy_to" => {
                let value = action.value.as_str().ok_or_else(|| ErrorResponse::new("InvalidInput", "folder rule actions require a mailbox id"))?;
                validate_host_id("rule.actions[].value", value)?;
            }
            "categorize" => validate_rule_value("rule.actions[].value", &action.value, false)?,
            "mark_read" | "mark_unread" | "flag" | "unflag" | "delete" | "archive" | "stop" => {
                if !action.value.is_null() && action.value != json!(true) {
                    return Err(ErrorResponse::new("InvalidInput", "rule action value is not supported"));
                }
            }
            _ => return Err(ErrorResponse::new("InvalidInput", "rule.actions[].action_type is not supported")),
        }
    }
    Ok(())
}

fn validate_account_fields(account: &UserMailAccountSetup) -> Vec<FieldIssue> {
    let mut errors = Vec::new();
    if account.account_label.trim().is_empty() {
        push_field_error(&mut errors, "account.account_label", "account label must not be empty");
    }
    if account.display_name.trim().is_empty() {
        push_field_error(&mut errors, "account.display_name", "display name must not be empty");
    }
    if validate_email(&account.email_address).is_err() {
        push_field_error(&mut errors, "account.email_address", "email address is not valid");
    }
    if let Some(reply_to) = &account.reply_to {
        if !reply_to.trim().is_empty() && validate_email(reply_to).is_err() {
            push_field_error(&mut errors, "account.reply_to", "reply-to address is not valid");
        }
    }
    match account.provider.as_str() {
        "manual_imap_smtp" | "google_oauth" | "microsoft_oauth" => {}
        value if !value.trim().is_empty() => {}
        _ => push_field_error(&mut errors, "account.provider", "provider must not be empty"),
    }
    match account.auth_method.as_str() {
        "password" | "app_password" | "oauth2" | "host_connector" => {}
        _ => push_field_error(&mut errors, "account.auth_method", "auth_method must be password, app_password, oauth2, or host_connector"),
    }

    let desired_status = account.desired_status.as_deref().unwrap_or("active");
    match desired_status {
        "active" | "disabled" | "incomplete" | "receive_only" => {}
        _ => push_field_error(&mut errors, "account.desired_status", "desired_status must be active, disabled, incomplete, or receive_only"),
    }

    if account.auth_method == "oauth2" {
        match &account.oauth {
            Some(oauth) => {
                if oauth.provider.trim().is_empty() {
                    push_field_error(&mut errors, "account.oauth.provider", "oauth provider must not be empty");
                }
                push_validation_result(&mut errors, "account.oauth.connection_ref", validate_secret_ref("account.oauth.connection_ref", &oauth.connection_ref));
                push_validation_result(&mut errors, "account.oauth.redirect_state_ref", validate_secret_ref("account.oauth.redirect_state_ref", &oauth.redirect_state_ref));
                if oauth.scopes.len() > 32 || oauth.scopes.iter().any(|scope| ensure_host_string("account.oauth.scopes[]", scope, 128).is_err()) {
                    push_field_error(&mut errors, "account.oauth.scopes", "oauth scopes exceed limits or contain unsupported characters");
                }
            }
            None => push_field_error(&mut errors, "account.oauth", "OAuth account setup requires oauth metadata"),
        }
    } else {
        match &account.incoming {
            Some(incoming) => validate_server_config(&mut errors, "account.incoming", "imap", incoming),
            None => push_field_error(&mut errors, "account.incoming", "manual setup requires incoming IMAP settings"),
        }
        if desired_status != "receive_only" {
            match &account.outgoing {
                Some(outgoing) => validate_server_config(&mut errors, "account.outgoing", "smtp", outgoing),
                None => push_field_error(&mut errors, "account.outgoing", "sending requires outgoing SMTP settings"),
            }
        }
    }

    if let Some(sync) = &account.sync {
        if let Some(interval) = sync.interval_minutes {
            if interval < 5 {
                push_field_error(&mut errors, "account.sync.interval_minutes", "sync interval must be at least 5 minutes");
            }
        }
    }

    errors
}

fn next_account_action(account: &UserMailAccountSetup, valid: bool) -> &'static str {
    if !valid {
        return "enter_secret";
    }
    if account.auth_method == "oauth2" {
        if account.oauth.as_ref().and_then(|oauth| oauth.connection_ref.as_ref()).is_none() {
            return "connect_oauth";
        }
        return "test_connection";
    }
    if account.incoming.as_ref().is_some_and(|incoming| !has_secret_ref(incoming)) {
        return "enter_secret";
    }
    if account.desired_status.as_deref() != Some("receive_only")
        && account.outgoing.as_ref().is_some_and(|outgoing| !outgoing.use_incoming_secret.unwrap_or(false) && !has_secret_ref(outgoing))
    {
        return "enter_secret";
    }
    if matches!(account.desired_status.as_deref(), Some("disabled") | Some("incomplete")) {
        return "save_disabled";
    }
    "test_connection"
}

fn test_steps_for(scope: &str, receive_only: bool) -> Result<Vec<String>, ErrorResponse> {
    let steps = match scope {
        "all" => {
            let mut steps = vec!["imap_auth", "imap_mailbox_discovery", "folder_mapping"];
            if !receive_only {
                steps.insert(2, "smtp_auth");
                steps.insert(3, "smtp_send_capability");
            }
            steps
        }
        "incoming" => vec!["imap_auth", "imap_mailbox_discovery"],
        "outgoing" if !receive_only => vec!["smtp_auth", "smtp_send_capability"],
        "folder_mapping" => vec!["folder_mapping"],
        "outgoing" => return Err(ErrorResponse::new("AccountIncomplete", "Receive-only accounts do not plan outgoing SMTP tests")),
        _ => return Err(ErrorResponse::new("InvalidInput", "test_scope must be all, incoming, outgoing, or folder_mapping")),
    };
    Ok(steps.into_iter().map(str::to_string).collect())
}

fn connection_test_passed(result: &ConnectionTestResult, receive_only: bool) -> bool {
    let required = if receive_only {
        vec!["imap_auth", "imap_mailbox_discovery"]
    } else {
        vec!["imap_auth", "imap_mailbox_discovery", "smtp_auth", "smtp_send_capability"]
    };
    required.iter().all(|required_step| {
        result.steps.iter().any(|step| step.step == *required_step && matches!(step.state.as_str(), "passed" | "warning"))
    })
}

fn account_summary_from_setup(account: &UserMailAccountSetup, make_default: bool, status: &str) -> MailAccountSummary {
    let account_id = account.account_id.clone().unwrap_or_else(|| {
        format!("acc_{}", hash_text(&format!("{}:{}", account.email_address, account.provider)))
    });
    MailAccountSummary {
        account_id,
        display_name: account.display_name.clone(),
        email_address: account.email_address.clone(),
        provider: Some(account.provider.clone()),
        enabled: matches!(status, "active" | "receive_only"),
        is_default: make_default,
        sync_interval_minutes: account.sync.as_ref().and_then(|sync| sync.interval_minutes).unwrap_or(5),
        color: None,
        connection_state: Some(if status == "receive_only" { "receive_only" } else { "pending" }.to_string()),
    }
}

fn validate_preferences(preferences: &UserMailPreferences) -> Result<(), ErrorResponse> {
    let page_size = preferences.reading.get("page_size").and_then(Value::as_u64).unwrap_or(DEFAULT_PAGE_SIZE as u64);
    if page_size == 0 || page_size > MAX_PAGE_SIZE as u64 {
        return Err(ErrorResponse::new("InvalidInput", format!("reading.page_size must be between 1 and {MAX_PAGE_SIZE}")));
    }
    let undo = preferences.compose.get("undo_send_delay_seconds").and_then(Value::as_u64).unwrap_or(10) as u32;
    match undo {
        0 | 5 | 10 | 30 => Ok(()),
        _ => Err(ErrorResponse::new("InvalidInput", "compose.undo_send_delay_seconds must be 0, 5, 10, or 30")),
    }
}

fn inspect_legacy(legacy: &LegacySettings) -> LegacyInspectionData {
    let accounts = legacy.settings.get("accounts").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut preferences_detected = Vec::new();
    if legacy.ui_schemas.get("main").is_some() || legacy.settings.get("general").is_some() {
        preferences_detected.push("reading");
        preferences_detected.push("compose");
        preferences_detected.push("notifications");
    }
    LegacyInspectionData {
        accounts_detected: accounts.len(),
        preferences_detected,
        credential_risks: if accounts.is_empty() { Vec::new() } else { vec!["legacy_account_metadata_requires_reconnect"] },
        migration_preview: json!({
            "accounts": accounts.iter().map(|account| json!({
                "email_address": account.get("email_address").cloned().unwrap_or(Value::Null),
                "provider": account.get("auth_method").cloned().unwrap_or(Value::Null),
                "requires_reconnect": true
            })).collect::<Vec<Value>>()
        }),
        requires_user_reconnect: accounts.iter()
            .filter_map(|account| account.get("email_address").and_then(Value::as_str).map(str::to_string))
            .collect(),
    }
}

fn handle_get_provider_capabilities(request: GetProviderCapabilitiesRequest) -> Result<OperationResponse<ProviderCapabilities>, ErrorResponse> {
    require_effects(&request.context, &[EFFECT_PROVIDER_CAPABILITY_READ, EFFECT_MAIL_STORE_READ])?;
    for provider_id in &request.provider_ids {
        ensure_non_empty("provider_ids[]", provider_id)?;
    }
    let mut data = default_provider_capabilities();
    if !request.provider_ids.is_empty() {
        data.providers.retain(|provider| provider.get("provider").and_then(Value::as_str).is_some_and(|id| request.provider_ids.iter().any(|requested| requested == id)));
    }
    let host_effects = vec![
        host_effect(&request.context, "get_provider_capabilities", EFFECT_PROVIDER_CAPABILITY_READ, "Read host account provider policy and connector availability", json!({ "user_id": request.context.user_id, "provider_ids": request.provider_ids })),
        host_effect(&request.context, "get_provider_capabilities", EFFECT_MAIL_STORE_READ, "Read user mail preference bounds that affect account setup", json!({ "user_id": request.context.user_id })),
    ];
    Ok(OperationResponse { operation: "get_provider_capabilities", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_validate_account_setup(request: ValidateAccountSetupRequest) -> Result<OperationResponse<AccountValidationData>, ErrorResponse> {
    if request.admin_policy_snapshot.is_none() {
        require_effects(&request.context, &[EFFECT_PROVIDER_CAPABILITY_READ])?;
    }
    let normalized = normalize_account(request.account);
    let field_errors = validate_account_fields(&normalized);
    let valid = field_errors.is_empty();
    let mut redacted_normalized = normalized.clone();
    redact_secret_refs(&mut redacted_normalized);
    let data = AccountValidationData {
        valid,
        normalized_account: if valid { Some(redacted_normalized) } else { None },
        field_errors,
        warnings: Vec::new(),
        next_required_action: next_account_action(&normalized, valid),
    };
    let host_effects = if request.admin_policy_snapshot.is_none() {
        vec![host_effect(&request.context, "validate_account_setup", EFFECT_PROVIDER_CAPABILITY_READ, "Read host policy before validating user-scoped account setup", json!({ "user_id": request.context.user_id, "provider": normalized.provider }))]
    } else {
        Vec::new()
    };
    Ok(OperationResponse { operation: "validate_account_setup", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_plan_connection_test(request: PlanConnectionTestRequest) -> Result<OperationResponse<ConnectionTestPlanData>, ErrorResponse> {
    let normalized = normalize_account(request.account);
    let errors = validate_account_fields(&normalized);
    if !errors.is_empty() {
        return Err(ErrorResponse::new("InvalidInput", "Account setup must be valid before planning connection tests").with_details(json!({ "field_errors": errors })));
    }
    let receive_only = normalized.desired_status.as_deref() == Some("receive_only");
    let steps = test_steps_for(&request.test_scope, receive_only)?;
    let mut required = vec![EFFECT_PROVIDER_CAPABILITY_READ, EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_SYNC, EFFECT_IMAP_FETCH];
    if steps.iter().any(|step| step.starts_with("smtp")) {
        required.push(EFFECT_SMTP_SEND);
    }
    require_effects(&request.context, &required)?;
    let account_id = normalized.account_id.clone().unwrap_or_else(|| format!("acc_{}", hash_text(&normalized.email_address)));
    let test_id = format!("test_{}", hash_text(&format!("{}:{}", account_id, request.test_scope)));
    let data = ConnectionTestPlanData { account_id: account_id.clone(), test_id: test_id.clone(), steps: steps.clone(), requires_host_execution: true };
    let host_effects = required.into_iter().map(|effect| host_effect(&request.context, "plan_connection_test", effect, "Plan host-owned connection test step without protocol I/O in Wasm", json!({ "user_id": request.context.user_id, "account_id": account_id, "test_id": test_id, "test_scope": request.test_scope, "steps": steps }))).collect();
    Ok(OperationResponse { operation: "plan_connection_test", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_complete_account_setup(request: CompleteAccountSetupRequest) -> Result<OperationResponse<AccountSetupCompletionData>, ErrorResponse> {
    let normalized = normalize_account(request.account);
    let errors = validate_account_fields(&normalized);
    if !errors.is_empty() {
        return Err(ErrorResponse::new("InvalidInput", "Account setup is not valid").with_details(json!({ "field_errors": errors })));
    }
    let status = request.save_mode.clone().or_else(|| normalized.desired_status.clone()).unwrap_or_else(|| "active".to_string());
    if !matches!(status.as_str(), "active" | "receive_only" | "disabled" | "incomplete") {
        return Err(ErrorResponse::new("InvalidInput", "save_mode must be active, receive_only, disabled, or incomplete"));
    }
    let receive_only = status == "receive_only";
    if matches!(status.as_str(), "active" | "receive_only") {
        match &request.connection_test_result {
            Some(result) if connection_test_passed(result, receive_only) => {}
            Some(_) => return Err(ErrorResponse::new("ConnectionTestFailed", "Connection test result did not pass required incoming/outgoing checks")),
            None => return Err(ErrorResponse::new("ConnectionTestRequired", "Active account setup requires a sanitized host connection test result")),
        }
    }
    let mut required = vec![EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_STORE_PLATFORM_SECRET, EFFECT_MAIL_STORE_WRITE];
    if status == "active" || status == "receive_only" {
        required.push(EFFECT_IMAP_SYNC);
    }
    require_effects(&request.context, &required)?;
    let account = account_summary_from_setup(&normalized, request.make_default, &status);
    let data = AccountSetupCompletionData { account: account.clone(), status: status.clone(), requires_reconnect: false, requires_sync: matches!(status.as_str(), "active" | "receive_only") };
    let host_effects = required.into_iter().map(|effect| host_effect(&request.context, "complete_account_setup", effect, "Persist user-scoped mail account metadata and secret references through host services", json!({ "user_id": request.context.user_id, "account_id": account.account_id, "status": status, "make_default": request.make_default }))).collect();
    Ok(OperationResponse { operation: "complete_account_setup", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_begin_oauth_account_setup(request: BeginOAuthAccountSetupRequest) -> Result<OperationResponse<OAuthBeginData>, ErrorResponse> {
    validate_host_id("provider", &request.provider)?;
    validate_email(&request.email_address)?;
    require_effects(&request.context, &[EFFECT_PROVIDER_CAPABILITY_READ, EFFECT_OAUTH_CONNECT])?;
    let authorization_ref = format!("authorization:{}", hash_text(&format!("{}:{}", request.provider, request.email_address)));
    let next_route = request.redirect_route.clone().unwrap_or_else(|| "app://mail-client/settings/accounts".to_string());
    let data = OAuthBeginData { provider: request.provider.clone(), authorization_ref_present: true, status: "host_action_required", next_route: next_route.clone() };
    let host_effects = vec![
        host_effect(&request.context, "begin_oauth_account_setup", EFFECT_PROVIDER_CAPABILITY_READ, "Validate OAuth provider availability", json!({ "user_id": request.context.user_id, "provider": request.provider })),
        host_effect(&request.context, "begin_oauth_account_setup", EFFECT_OAUTH_CONNECT, "Start host-owned OAuth authorization", json!({ "user_id": request.context.user_id, "provider": request.provider, "email_address": request.email_address, "authorization_ref": authorization_ref, "requested_scopes": request.requested_scopes, "next_route": next_route })),
    ];
    Ok(OperationResponse { operation: "begin_oauth_account_setup", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_complete_oauth_account_setup(request: CompleteOAuthAccountSetupRequest) -> Result<OperationResponse<OAuthCompletionData>, ErrorResponse> {
    validate_host_id("provider", &request.provider)?;
    validate_secret_ref("authorization_ref", &Some(request.authorization_ref.clone()))?;
    validate_secret_ref("oauth_connection_ref", &Some(request.oauth_connection_ref.clone()))?;
    require_effects(&request.context, &[EFFECT_OAUTH_CONNECT, EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE, EFFECT_IMAP_SYNC])?;
    let account_setup = request.account.clone().unwrap_or(UserMailAccountSetup {
        account_id: None,
        revision: None,
        account_label: request.provider.clone(),
        display_name: request.provider.clone(),
        email_address: format!("{}@example.invalid", request.context.user_id),
        reply_to: None,
        provider: request.provider.clone(),
        auth_method: "oauth2".to_string(),
        incoming: None,
        outgoing: None,
        oauth: Some(OAuthAccountConfig { provider: request.provider.clone(), connection_ref: None, redirect_state_ref: None, scopes: Vec::new() }),
        sync: None,
        folder_mapping: None,
        desired_status: Some("active".to_string()),
    });
    if let Some(result) = &request.connection_test_result {
        if !connection_test_passed(result, false) {
            return Err(ErrorResponse::new("ConnectionTestFailed", "OAuth connection test result did not pass required checks"));
        }
    }
    let account = account_summary_from_setup(&account_setup, false, "active");
    let identity = MailIdentity {
        identity_id: format!("id_{}", hash_text(&account.email_address)),
        account_id: account.account_id.clone(),
        kind: "primary".to_string(),
        display_name: account.display_name.clone(),
        email_address: account.email_address.clone(),
        reply_to: None,
        signature_id: None,
        enabled: true,
        is_default_for_account: true,
        is_global_default: false,
        verification_state: "host_verified".to_string(),
    };
    let host_effects = vec![
        host_effect(&request.context, "complete_oauth_account_setup", EFFECT_OAUTH_CONNECT, "Read sanitized OAuth connection completion from host", json!({ "user_id": request.context.user_id, "provider": request.provider, "authorization_ref_present": true, "oauth_connection_ref_present": true })),
        host_effect(&request.context, "complete_oauth_account_setup", EFFECT_ACCOUNT_CREDENTIAL_WRITE, "Store OAuth connection reference for user mail account", json!({ "user_id": request.context.user_id, "account_id": account.account_id, "oauth_connection_ref_present": true })),
        host_effect(&request.context, "complete_oauth_account_setup", EFFECT_MAIL_STORE_WRITE, "Persist OAuth mail account and identity metadata", json!({ "user_id": request.context.user_id, "account_id": account.account_id })),
        host_effect(&request.context, "complete_oauth_account_setup", EFFECT_IMAP_SYNC, "Schedule initial host mail sync for OAuth account", json!({ "user_id": request.context.user_id, "account_id": account.account_id })),
    ];
    Ok(OperationResponse { operation: "complete_oauth_account_setup", status: "accepted", data: OAuthCompletionData { account, identity, requires_sync: true }, host_effects, warnings: Vec::new() })
}

fn handle_disconnect_oauth_account(request: DisconnectOAuthAccountRequest) -> Result<OperationResponse<OAuthDisconnectData>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_secret_ref("oauth_connection_ref", &request.oauth_connection_ref)?;
    require_effects(&request.context, &[EFFECT_OAUTH_DISCONNECT, EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE])?;
    let host_effects = vec![
        host_effect(&request.context, "disconnect_oauth_account", EFFECT_OAUTH_DISCONNECT, "Revoke host-owned OAuth connection", json!({ "user_id": request.context.user_id, "account_id": request.account_id, "oauth_connection_ref_present": request.oauth_connection_ref.is_some() })),
        host_effect(&request.context, "disconnect_oauth_account", EFFECT_ACCOUNT_CREDENTIAL_WRITE, "Mark OAuth credential reference revoked", json!({ "user_id": request.context.user_id, "account_id": request.account_id })),
        host_effect(&request.context, "disconnect_oauth_account", EFFECT_MAIL_STORE_WRITE, "Update account enabled state after OAuth disconnect", json!({ "user_id": request.context.user_id, "account_id": request.account_id, "disabled": request.disable_account })),
    ];
    Ok(OperationResponse { operation: "disconnect_oauth_account", status: "accepted", data: OAuthDisconnectData { account_id: request.account_id, credential_state: "revoked", disabled: request.disable_account }, host_effects, warnings: Vec::new() })
}

fn handle_remove_account(request: RemoveAccountRequest) -> Result<OperationResponse<AccountRemovalData>, ErrorResponse> {
    validate_host_id("account_id", &request.account_id)?;
    validate_optional_host_id("revision", &request.revision)?;
    let mut required = vec![EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE];
    if request.remove_local_cache || request.delete_drafts {
        required.push(EFFECT_MAIL_STORE_DELETE);
    }
    if request.cancel_scheduled_sends {
        required.push(EFFECT_CANCEL_JOB);
    }
    require_effects(&request.context, &required)?;
    let mut cleanup_planned = Vec::new();
    if request.remove_local_cache { cleanup_planned.push("local_cache"); }
    if request.cancel_scheduled_sends { cleanup_planned.push("scheduled_sends"); }
    if request.delete_drafts { cleanup_planned.push("drafts"); }
    let host_effects = required.into_iter().map(|effect| host_effect(&request.context, "remove_account", effect, "Remove or disable a user mail account through host services", json!({ "user_id": request.context.user_id, "account_id": request.account_id, "revision": request.revision, "cleanup_planned": cleanup_planned }))).collect();
    Ok(OperationResponse { operation: "remove_account", status: "accepted", data: AccountRemovalData { account_id: request.account_id, status: "removed_pending_cleanup", cleanup_planned }, host_effects, warnings: Vec::new() })
}

fn handle_list_identities(request: ListIdentitiesRequest) -> Result<OperationResponse<IdentityListData>, ErrorResponse> {
    require_effects(&request.context, &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_MAIL_STORE_READ])?;
    if let Some(account_id) = &request.account_id { ensure_non_empty("account_id", account_id)?; }
    let snapshot = request.snapshot.unwrap_or_default();
    let data = IdentityListData { identities: snapshot.identities, signatures: snapshot.signatures, needs_host_refresh: true };
    let host_effects = vec![
        host_effect(&request.context, "list_identities", EFFECT_ACCOUNT_CREDENTIAL_READ, "Read sender identity credential state without exposing secrets", json!({ "user_id": request.context.user_id, "account_id": request.account_id })),
        host_effect(&request.context, "list_identities", EFFECT_MAIL_STORE_READ, "Read sender identities and signatures", json!({ "user_id": request.context.user_id, "account_id": request.account_id })),
    ];
    Ok(OperationResponse { operation: "list_identities", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_save_identity(request: SaveIdentityRequest) -> Result<OperationResponse<IdentityMutationData>, ErrorResponse> {
    ensure_non_empty("identity.identity_id", &request.identity.identity_id)?;
    ensure_non_empty("identity.account_id", &request.identity.account_id)?;
    validate_email(&request.identity.email_address)?;
    if let Some(reply_to) = &request.identity.reply_to { if !reply_to.trim().is_empty() { validate_email(reply_to)?; } }
    require_effects(&request.context, &[EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE])?;
    let host_effects = vec![
        host_effect(&request.context, "save_identity", EFFECT_ACCOUNT_CREDENTIAL_WRITE, "Validate identity send-as capability through host account services", json!({ "user_id": request.context.user_id, "account_id": request.identity.account_id, "identity_id": request.identity.identity_id, "revision": request.revision })),
        host_effect(&request.context, "save_identity", EFFECT_MAIL_STORE_WRITE, "Persist sender identity metadata", json!({ "user_id": request.context.user_id, "identity": request.identity })),
    ];
    let verification_required = !matches!(request.identity.verification_state.as_str(), "allowed" | "host_verified" | "verified");
    Ok(OperationResponse { operation: "save_identity", status: "accepted", data: IdentityMutationData { identity: request.identity, verification_required }, host_effects, warnings: Vec::new() })
}

fn handle_delete_identity(request: DeleteIdentityRequest) -> Result<OperationResponse<IdentityDeleteData>, ErrorResponse> {
    validate_host_id("identity_id", &request.identity_id)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    require_effects(&request.context, &[EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE])?;
    let host_effects = vec![
        host_effect(&request.context, "delete_identity", EFFECT_ACCOUNT_CREDENTIAL_WRITE, "Remove send-as authorization for an identity", json!({ "user_id": request.context.user_id, "identity_id": request.identity_id, "account_id": request.account_id, "force": request.force })),
        host_effect(&request.context, "delete_identity", EFFECT_MAIL_STORE_WRITE, "Remove identity metadata and update defaults", json!({ "user_id": request.context.user_id, "identity_id": request.identity_id, "account_id": request.account_id, "force": request.force })),
    ];
    Ok(OperationResponse { operation: "delete_identity", status: "accepted", data: IdentityDeleteData { identity_id: request.identity_id, removed: true, default_reassigned_to: None }, host_effects, warnings: Vec::new() })
}

fn handle_save_signature(request: SaveSignatureRequest) -> Result<OperationResponse<SignatureMutationData>, ErrorResponse> {
    ensure_non_empty("signature.signature_id", &request.signature.signature_id)?;
    ensure_non_empty("signature.account_id", &request.signature.account_id)?;
    ensure_non_empty("signature.label", &request.signature.label)?;
    validate_compose_body("signature", &request.signature.body_html, &request.signature.body_text)?;
    require_effects(&request.context, &[EFFECT_MAIL_STORE_WRITE])?;
    let host_effects = vec![host_effect(&request.context, "save_signature", EFFECT_MAIL_STORE_WRITE, "Persist user signature metadata and content", json!({ "user_id": request.context.user_id, "signature": request.signature, "revision": request.revision }))];
    Ok(OperationResponse { operation: "save_signature", status: "accepted", data: SignatureMutationData { signature: request.signature }, host_effects, warnings: Vec::new() })
}

fn handle_delete_signature(request: DeleteSignatureRequest) -> Result<OperationResponse<SignatureDeleteData>, ErrorResponse> {
    validate_host_id("signature_id", &request.signature_id)?;
    validate_optional_host_id("account_id", &request.account_id)?;
    require_effects(&request.context, &[EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE])?;
    let host_effects = vec![
        host_effect(&request.context, "delete_signature", EFFECT_MAIL_STORE_WRITE, "Clear identity defaults that reference a deleted signature", json!({ "user_id": request.context.user_id, "signature_id": request.signature_id, "account_id": request.account_id })),
        host_effect(&request.context, "delete_signature", EFFECT_MAIL_STORE_DELETE, "Delete user signature", json!({ "user_id": request.context.user_id, "signature_id": request.signature_id, "account_id": request.account_id, "force": request.force })),
    ];
    Ok(OperationResponse { operation: "delete_signature", status: "accepted", data: SignatureDeleteData { signature_id: request.signature_id, removed: true, identity_updates: Vec::new() }, host_effects, warnings: Vec::new() })
}

fn handle_get_user_preferences(request: GetUserPreferencesRequest) -> Result<OperationResponse<UserPreferencesData>, ErrorResponse> {
    require_effects(&request.context, &[EFFECT_MAIL_STORE_READ])?;
    let data = UserPreferencesData { preferences: request.snapshot.unwrap_or_else(default_preferences), defaults_applied: request.include_defaults, needs_host_refresh: true };
    let host_effects = vec![host_effect(&request.context, "get_user_preferences", EFFECT_MAIL_STORE_READ, "Read user-scoped mail preferences", json!({ "user_id": request.context.user_id, "include_defaults": request.include_defaults }))];
    Ok(OperationResponse { operation: "get_user_preferences", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_save_user_preferences(request: SaveUserPreferencesRequest) -> Result<OperationResponse<UserPreferencesMutationData>, ErrorResponse> {
    validate_preferences(&request.preferences)?;
    require_effects(&request.context, &[EFFECT_MAIL_STORE_WRITE])?;
    let host_effects = vec![host_effect(&request.context, "save_user_preferences", EFFECT_MAIL_STORE_WRITE, "Persist user-scoped mail preferences", json!({ "user_id": request.context.user_id, "preferences": request.preferences, "revision": request.revision }))];
    Ok(OperationResponse { operation: "save_user_preferences", status: "accepted", data: UserPreferencesMutationData { preferences: request.preferences, changed_groups: vec!["reading", "compose", "notifications", "sync_defaults"] }, host_effects, warnings: Vec::new() })
}

fn handle_inspect_legacy_settings(request: InspectLegacySettingsRequest) -> Result<OperationResponse<LegacyInspectionData>, ErrorResponse> {
    let host_effects = if request.legacy_settings.is_none() {
        require_effects(&request.context, &[EFFECT_MAIL_STORE_READ])?;
        vec![host_effect(&request.context, "inspect_legacy_settings", EFFECT_MAIL_STORE_READ, "Read legacy settings for explicit user-scoped migration preview", json!({ "user_id": request.context.user_id }))]
    } else {
        Vec::new()
    };
    let data = inspect_legacy(&request.legacy_settings.unwrap_or_default());
    Ok(OperationResponse { operation: "inspect_legacy_settings", status: "accepted", data, host_effects, warnings: Vec::new() })
}

fn handle_migrate_legacy_settings(request: MigrateLegacySettingsRequest) -> Result<OperationResponse<LegacyMigrationData>, ErrorResponse> {
    let mut required = vec![EFFECT_MAIL_STORE_READ, EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE];
    if !request.dry_run {
        required.push(EFFECT_ACCOUNT_CREDENTIAL_WRITE);
    }
    require_effects(&request.context, &required)?;
    let inspection = inspect_legacy(&request.legacy_settings);
    let migrated_preferences = json!({
        "reading": {
            "thread_view": request.legacy_settings.ui_schemas.get("main").and_then(|main| main.get("properties")).and_then(|properties| properties.get("thread_view")).cloned().unwrap_or(Value::Null),
            "focused_inbox": request.legacy_settings.ui_schemas.get("main").and_then(|main| main.get("properties")).and_then(|properties| properties.get("focused_inbox")).cloned().unwrap_or(Value::Null)
        },
        "migration_note": "legacy schema-rendered settings classified for user-scoped preferences"
    });
    let obsolete_keys_removed = if request.dry_run { Vec::new() } else { vec!["ui_schemas.main", "ui_schemas.settings", "settings.accounts", "settings.general"] };
    let data = LegacyMigrationData {
        migrated_accounts: inspection.migration_preview.get("accounts").and_then(Value::as_array).cloned().unwrap_or_default(),
        migrated_preferences,
        migration_map: json!({
            "ui_schemas.main.thread_view": "preferences.reading.thread_view",
            "ui_schemas.main.focused_inbox": "preferences.reading.focused_inbox",
            "ui_schemas.main.preview_pane": "preferences.reading.preview_pane",
            "settings.accounts[]": "user_mail_account_setup_requires_reconnect",
            "settings.general.undo_send_delay_seconds": "preferences.compose.undo_send_delay_seconds"
        }),
        requires_reconnect: inspection.requires_user_reconnect,
        obsolete_keys_removed,
    };
    let host_effects = required.into_iter().map(|effect| host_effect(&request.context, "migrate_legacy_settings", effect, "Migrate legacy settings into user-scoped mail records without moving credentials into admin policy", json!({ "user_id": request.context.user_id, "dry_run": request.dry_run, "migration_options": request.migration_options }))).collect();
    Ok(OperationResponse { operation: "migrate_legacy_settings", status: "accepted", data, host_effects, warnings: Vec::new() })
}

macro_rules! export_operation {
    ($fn_name:ident, $request_type:ty, $permission:expr, [$($effect:expr),* $(,)?], $handler:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn_name(input_ptr: *const u8, input_len: usize) -> *mut u8 {
            handle_operation::<$request_type, _, _>(
                input_ptr,
                input_len,
                $permission,
                &[$($effect),*],
                $handler,
            )
        }
    };
}

export_operation!(
    list_accounts,
    ListAccountsRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ],
    handle_list_accounts
);
export_operation!(
    get_account_status,
    GetAccountStatusRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_SYNC],
    handle_get_account_status
);
export_operation!(
    list_mailboxes,
    ListMailboxesRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH],
    handle_list_mailboxes
);
export_operation!(
    read_emails,
    ReadEmailsRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH],
    handle_read_emails
);
export_operation!(
    search_emails,
    SearchEmailsRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH],
    handle_search_emails
);
export_operation!(
    get_email,
    GetEmailRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH, EFFECT_MAIL_STORE_WRITE],
    handle_get_email
);
export_operation!(
    get_thread,
    GetThreadRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH],
    handle_get_thread
);
export_operation!(
    mark_read,
    MarkReadRequest,
    Permission::Read,
    [EFFECT_MAIL_STORE_WRITE],
    handle_mark_read
);
export_operation!(
    flag_email,
    FlagEmailRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_flag_email
);
export_operation!(
    draft_email,
    DraftEmailRequest,
    Permission::Draft,
    [EFFECT_MAIL_STORE_WRITE],
    handle_draft_email
);
export_operation!(
    update_draft,
    UpdateDraftRequest,
    Permission::Draft,
    [EFFECT_MAIL_STORE_WRITE],
    handle_update_draft
);
export_operation!(
    discard_draft,
    DiscardDraftRequest,
    Permission::Draft,
    [EFFECT_MAIL_STORE_DELETE],
    handle_discard_draft
);
export_operation!(
    send_email,
    SendEmailRequest,
    Permission::Send,
    [EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND],
    handle_send_email
);
export_operation!(
    reply_email,
    ReplyEmailRequest,
    Permission::Send,
    [EFFECT_MAIL_STORE_READ, EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND],
    handle_reply_email
);
export_operation!(
    forward_email,
    ForwardEmailRequest,
    Permission::Send,
    [EFFECT_MAIL_STORE_READ, EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND],
    handle_forward_email
);
export_operation!(
    move_email,
    MoveEmailRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_move_email
);
export_operation!(
    delete_email,
    DeleteEmailRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE],
    handle_delete_email
);
export_operation!(
    archive_email,
    ArchiveEmailRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_archive_email
);
export_operation!(
    create_folder,
    CreateFolderRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_create_folder
);
export_operation!(
    rename_folder,
    RenameFolderRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_rename_folder
);
export_operation!(
    delete_folder,
    DeleteFolderRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_DELETE],
    handle_delete_folder
);
export_operation!(
    get_unread_count,
    GetUnreadCountRequest,
    Permission::Read,
    [EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH],
    handle_get_unread_count
);
export_operation!(
    create_rule,
    CreateRuleRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_WRITE],
    handle_create_rule
);
export_operation!(
    list_rules,
    ListRulesRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_READ],
    handle_list_rules
);
export_operation!(
    delete_rule,
    DeleteRuleRequest,
    Permission::Organize,
    [EFFECT_MAIL_STORE_DELETE],
    handle_delete_rule
);
export_operation!(
    schedule_send,
    ScheduleSendRequest,
    Permission::Send,
    [EFFECT_MAIL_STORE_WRITE, EFFECT_SCHEDULE_JOB],
    handle_schedule_send
);
export_operation!(
    cancel_scheduled_send,
    CancelScheduledSendRequest,
    Permission::Send,
    [EFFECT_MAIL_STORE_WRITE, EFFECT_CANCEL_JOB],
    handle_cancel_scheduled_send
);
export_operation!(
    get_provider_capabilities,
    GetProviderCapabilitiesRequest,
    Permission::Accounts,
    [],
    handle_get_provider_capabilities
);
export_operation!(
    validate_account_setup,
    ValidateAccountSetupRequest,
    Permission::Accounts,
    [],
    handle_validate_account_setup
);
export_operation!(
    plan_connection_test,
    PlanConnectionTestRequest,
    Permission::Accounts,
    [],
    handle_plan_connection_test
);
export_operation!(
    complete_account_setup,
    CompleteAccountSetupRequest,
    Permission::Accounts,
    [],
    handle_complete_account_setup
);
export_operation!(
    begin_oauth_account_setup,
    BeginOAuthAccountSetupRequest,
    Permission::Accounts,
    [],
    handle_begin_oauth_account_setup
);
export_operation!(
    complete_oauth_account_setup,
    CompleteOAuthAccountSetupRequest,
    Permission::Accounts,
    [],
    handle_complete_oauth_account_setup
);
export_operation!(
    disconnect_oauth_account,
    DisconnectOAuthAccountRequest,
    Permission::Accounts,
    [],
    handle_disconnect_oauth_account
);
export_operation!(
    remove_account,
    RemoveAccountRequest,
    Permission::Accounts,
    [],
    handle_remove_account
);
export_operation!(
    list_identities,
    ListIdentitiesRequest,
    Permission::Accounts,
    [],
    handle_list_identities
);
export_operation!(
    save_identity,
    SaveIdentityRequest,
    Permission::Accounts,
    [],
    handle_save_identity
);
export_operation!(
    delete_identity,
    DeleteIdentityRequest,
    Permission::Accounts,
    [],
    handle_delete_identity
);
export_operation!(
    save_signature,
    SaveSignatureRequest,
    Permission::Accounts,
    [],
    handle_save_signature
);
export_operation!(
    delete_signature,
    DeleteSignatureRequest,
    Permission::Accounts,
    [],
    handle_delete_signature
);
export_operation!(
    get_user_preferences,
    GetUserPreferencesRequest,
    Permission::Accounts,
    [],
    handle_get_user_preferences
);
export_operation!(
    save_user_preferences,
    SaveUserPreferencesRequest,
    Permission::Accounts,
    [],
    handle_save_user_preferences
);
export_operation!(
    inspect_legacy_settings,
    InspectLegacySettingsRequest,
    Permission::Accounts,
    [],
    handle_inspect_legacy_settings
);
export_operation!(
    migrate_legacy_settings,
    MigrateLegacySettingsRequest,
    Permission::Accounts,
    [],
    handle_migrate_legacy_settings
);

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(permission: &str, effects: &[&str]) -> RequestContext {
        RequestContext {
            user_id: "usr_test".to_string(),
            permission: permission.to_string(),
            available_effects: effects.iter().map(|effect| effect.to_string()).collect(),
            agent_id: None,
            request_id: Some("req_test".to_string()),
        }
    }

    fn call_export(export: extern "C" fn(*const u8, usize) -> *mut u8, payload: Value) -> Value {
        let body = serde_json::to_string(&payload).expect("payload serializes");
        let ptr = export(body.as_ptr(), body.len());
        assert!(!ptr.is_null());
        let response = unsafe { CString::from_raw(ptr as *mut c_char) };
        serde_json::from_str(response.to_str().expect("utf8 response")).expect("json response")
    }

    fn account_payload() -> Value {
        json!({
            "account_label": "Work",
            "display_name": "Test User",
            "email_address": "test@example.com",
            "provider": "manual_imap_smtp",
            "auth_method": "app_password",
            "incoming": {
                "protocol": "imap",
                "host": "imap.example.com",
                "port": 993,
                "security": "ssl_tls",
                "username": "test@example.com",
                "auth_method": "app_password",
                "secret_input_ref": "host-secure-field:incoming"
            },
            "outgoing": {
                "protocol": "smtp",
                "host": "smtp.example.com",
                "port": 587,
                "security": "starttls",
                "username": "test@example.com",
                "auth_method": "app_password",
                "use_incoming_secret": true
            },
            "sync": { "interval_minutes": 5 },
            "desired_status": "active"
        })
    }

    #[test]
    fn validate_account_setup_reports_field_errors_without_echoing_raw_secret_names() {
        let mut account = account_payload();
        account["email_address"] = json!("not-an-email");
        account["password"] = json!("super-secret");
        let response = call_export(
            validate_account_setup,
            json!({
                "context": context("accounts", &[EFFECT_PROVIDER_CAPABILITY_READ]),
                "account": account
            }),
        );

        assert_eq!(response["ok"]["operation"], "validate_account_setup");
        assert_eq!(response["ok"]["data"]["valid"], false);
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(!serialized.contains("super-secret"));
        assert!(!serialized.contains("\"password\""));
    }

    #[test]
    fn permission_matrix_keeps_organize_and_accounts_non_linear() {
        assert!(Permission::Organize.satisfies(Permission::Read));
        assert!(Permission::Organize.satisfies(Permission::Organize));
        assert!(!Permission::Organize.satisfies(Permission::Draft));
        assert!(!Permission::Organize.satisfies(Permission::Send));
        assert!(!Permission::Organize.satisfies(Permission::Accounts));
        assert!(!Permission::Organize.satisfies(Permission::Admin));

        assert!(Permission::Accounts.satisfies(Permission::Read));
        assert!(Permission::Accounts.satisfies(Permission::Accounts));
        assert!(!Permission::Accounts.satisfies(Permission::Send));
        assert!(!Permission::Accounts.satisfies(Permission::Organize));
        assert!(!Permission::Accounts.satisfies(Permission::Admin));
    }

    #[test]
    fn account_exports_require_accounts_permission() {
        let response = call_export(
            validate_account_setup,
            json!({
                "context": context("send", &[EFFECT_PROVIDER_CAPABILITY_READ]),
                "account": account_payload()
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn organize_permission_cannot_send_email_export() {
        let response = call_export(
            send_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "draft_id": "draft-1",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn accounts_permission_cannot_organize_email_export() {
        let response = call_export(
            flag_email,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "flagged": true
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn validate_account_setup_rejects_raw_secret_ref_and_redacts_valid_refs() {
        let mut raw_account = account_payload();
        raw_account["incoming"]["secret_input_ref"] = json!("plain-password-or-random-text");
        let raw_response = call_export(
            validate_account_setup,
            json!({
                "context": context("accounts", &[EFFECT_PROVIDER_CAPABILITY_READ]),
                "account": raw_account
            }),
        );
        assert_eq!(raw_response["ok"]["data"]["valid"], false);

        let valid_response = call_export(
            validate_account_setup,
            json!({
                "context": context("accounts", &[EFFECT_PROVIDER_CAPABILITY_READ]),
                "account": account_payload()
            }),
        );
        let serialized = serde_json::to_string(&valid_response).unwrap();
        assert_eq!(valid_response["ok"]["data"]["valid"], true);
        assert!(!serialized.contains("host-secure-field:incoming"));
        assert!(!serialized.contains("secret_input_ref"));
    }

    #[test]
    fn plan_connection_test_requires_host_effects_and_plans_protocol_steps() {
        let response = call_export(
            plan_connection_test,
            json!({
                "context": context("accounts", &[
                    EFFECT_PROVIDER_CAPABILITY_READ,
                    EFFECT_ACCOUNT_CREDENTIAL_READ,
                    EFFECT_IMAP_SYNC,
                    EFFECT_IMAP_FETCH,
                    EFFECT_SMTP_SEND
                ]),
                "account": account_payload(),
                "test_scope": "all"
            }),
        );

        assert_eq!(response["ok"]["data"]["requires_host_execution"], true);
        assert_eq!(response["ok"]["data"]["steps"].as_array().unwrap().len(), 5);
        assert_eq!(response["ok"]["host_effects"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn complete_account_setup_requires_connection_test_for_active_accounts() {
        let response = call_export(
            complete_account_setup,
            json!({
                "context": context("accounts", &[
                    EFFECT_ACCOUNT_CREDENTIAL_WRITE,
                    EFFECT_STORE_PLATFORM_SECRET,
                    EFFECT_MAIL_STORE_WRITE,
                    EFFECT_IMAP_SYNC
                ]),
                "account": account_payload(),
                "save_mode": "active"
            }),
        );

        assert_eq!(response["err"]["code"], "ConnectionTestRequired");
    }

    #[test]
    fn state_changing_host_effect_keys_include_payload_hash_with_request_id() {
        let first = call_export(
            send_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "draft_id": "draft-1",
                "account_id": "acc-1"
            }),
        );
        let second = call_export(
            send_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "draft_id": "draft-2",
                "account_id": "acc-1"
            }),
        );
        let first_key = first["ok"]["host_effects"][0]["idempotency_key"].as_str().unwrap();
        let second_key = second["ok"]["host_effects"][0]["idempotency_key"].as_str().unwrap();
        assert_ne!(first_key, second_key);

        let folder_first = call_export(
            move_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "destination_mailbox_id": "archive"
            }),
        );
        let folder_second = call_export(
            move_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "destination_mailbox_id": "projects"
            }),
        );
        assert_ne!(
            folder_first["ok"]["host_effects"][0]["idempotency_key"],
            folder_second["ok"]["host_effects"][0]["idempotency_key"]
        );
    }

    #[test]
    fn host_bound_inputs_reject_controls_and_unsupported_shapes() {
        let search_response = call_export(
            search_emails,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "query": "hello\nworld"
            }),
        );
        assert_eq!(search_response["err"]["code"], "InvalidInput");

        let rule_response = call_export(
            create_rule,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "account_id": "acc-1",
                "rule": {
                    "rule_id": "rule-1",
                    "name": "Bad rule",
                    "enabled": true,
                    "stop_processing": false,
                    "priority": 1,
                    "conditions": [{ "field": "anything", "operator": "equals", "value": "x" }],
                    "actions": [{ "action_type": "move_to", "value": "archive" }]
                }
            }),
        );
        assert_eq!(rule_response["err"]["code"], "InvalidInput");

        let mark_response = call_export(
            mark_read,
            json!({
                "context": context("read", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "read": true,
                "account_id": "acc 1"
            }),
        );
        assert_eq!(mark_response["err"]["code"], "InvalidInput");

        let flag_response = call_export(
            flag_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "flagged": true,
                "account_id": "acc\n1"
            }),
        );
        assert_eq!(flag_response["err"]["code"], "InvalidInput");

        let move_response = call_export(
            move_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "destination_mailbox_id": "archive",
                "account_id": "acc#1"
            }),
        );
        assert_eq!(move_response["err"]["code"], "InvalidInput");
    }

    #[test]
    fn email_validation_rejects_control_chars_without_echoing_input() {
        let response = call_export(
            draft_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "victim@example.com\nBcc:evil@example.com" }],
                    "subject": "Hello",
                    "body_text": "Body"
                }
            }),
        );
        assert_eq!(response["err"]["code"], "InvalidRecipient");
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(!serialized.contains("victim@example.com"));
        assert!(!serialized.contains("evil@example.com"));
    }

    #[test]
    fn save_user_preferences_validates_bounds() {
        let response = call_export(
            save_user_preferences,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_WRITE]),
                "preferences": {
                    "reading": { "page_size": 250 },
                    "compose": { "undo_send_delay_seconds": 10 },
                    "notifications": { "enabled": true },
                    "sync_defaults": { "interval_minutes": 5 }
                }
            }),
        );

        assert_eq!(response["err"]["code"], "InvalidInput");
    }

    #[test]
    fn migrate_legacy_settings_maps_old_schema_to_user_scoped_records() {
        let response = call_export(
            migrate_legacy_settings,
            json!({
                "context": context("accounts", &[
                    EFFECT_MAIL_STORE_READ,
                    EFFECT_MAIL_STORE_WRITE,
                    EFFECT_MAIL_STORE_DELETE,
                    EFFECT_ACCOUNT_CREDENTIAL_WRITE
                ]),
                "legacy_settings": {
                    "ui_schemas": { "main": { "properties": { "thread_view": { "default": true } } } },
                    "settings": { "accounts": [{ "email_address": "old@example.com" }] }
                },
                "dry_run": false
            }),
        );

        assert_eq!(response["ok"]["data"]["requires_reconnect"][0], "old@example.com");
        assert!(response["ok"]["data"]["migration_map"].get("settings.accounts[]").is_some());
    }

    #[test]
    fn existing_read_emails_workflow_is_preserved() {
        let response = call_export(
            read_emails,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "mailbox_id": "inbox",
                "pagination": { "offset": 0, "limit": 25 },
                "snapshot": { "messages": [], "total_count": 0, "unread_count": 0 }
            }),
        );

        assert_eq!(response["ok"]["operation"], "read_emails");
        assert_eq!(response["ok"]["data"]["page"]["limit"], 25);
    }

    // ─── Read Workflows ───────────────────────────────────────────────────────
    // Intent: user can open the app and see their accounts, folders, and mail.

    #[test]
    fn list_accounts_signals_host_fetch_so_account_switcher_can_populate() {
        // Result: account switcher and sidebar show the user's configured accounts.
        let response = call_export(
            list_accounts,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ])
            }),
        );

        assert_eq!(response["ok"]["operation"], "list_accounts");
        assert_eq!(response["ok"]["status"], "accepted");
        // No snapshot supplied → host refresh required so the UI shows real accounts.
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_ACCOUNT_CREDENTIAL_READ));
    }

    #[test]
    fn get_account_status_triggers_imap_sync_check_for_health_dot() {
        // Result: the toolbar health dot and nav badge reflect live account state.
        let response = call_export(
            get_account_status,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_SYNC]),
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "get_account_status");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_IMAP_SYNC));
    }

    #[test]
    fn list_mailboxes_signals_host_fetch_so_sidebar_can_show_folder_tree() {
        // Result: left sidebar shows all folders including custom ones with counts.
        let response = call_export(
            list_mailboxes,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "list_mailboxes");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(!effects.is_empty(), "host refresh effect must be planned when no snapshot supplied");
    }

    #[test]
    fn read_emails_without_snapshot_signals_host_refresh_for_inbox() {
        // Result: message list renders with a loading skeleton while host fetches real mail.
        let response = call_export(
            read_emails,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "mailbox_id": "inbox"
            }),
        );

        assert_eq!(response["ok"]["operation"], "read_emails");
        assert_eq!(response["ok"]["data"]["needs_host_refresh"], true);
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_IMAP_FETCH));
    }

    #[test]
    fn get_email_plans_read_and_mark_read_effects_so_message_opens_unread_state_clears() {
        // Result: opening a message body loads content and automatically clears the unread badge.
        let response = call_export(
            get_email,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH, EFFECT_MAIL_STORE_WRITE]),
                "email_id": "email-42",
                "account_id": "acc-1",
                "mark_read": true
            }),
        );

        assert_eq!(response["ok"]["operation"], "get_email");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let effect_names: Vec<&str> = effects.iter().map(|e| e["effect"].as_str().unwrap()).collect();
        assert!(effect_names.contains(&EFFECT_IMAP_FETCH));
        assert!(effect_names.contains(&EFFECT_MAIL_STORE_WRITE),
            "mark-read write effect must be present so unread count reconciles after open");
    }

    #[test]
    fn get_thread_returns_conversation_context_for_reading_pane() {
        // Result: reading pane shows the full conversation thread so user can follow reply chains.
        let response = call_export(
            get_thread,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "thread_id": "thread-7",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "get_thread");
        assert_eq!(response["ok"]["data"]["thread_id"], "thread-7");
        assert_eq!(response["ok"]["data"]["needs_host_refresh"], true);
    }

    #[test]
    fn mark_read_plans_bulk_write_effect_so_read_state_persists_after_bulk_action() {
        // Result: marking multiple messages read updates the sidebar unread counts.
        let response = call_export(
            mark_read,
            json!({
                "context": context("read", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1", "email-2", "email-3"],
                "read": true,
                "account_id": "acc-1",
                "mailbox_id": "inbox"
            }),
        );

        assert_eq!(response["ok"]["operation"], "mark_read");
        assert_eq!(response["ok"]["data"]["mutation"], "mark_read");
        assert_eq!(response["ok"]["data"]["resource_ids"].as_array().unwrap().len(), 3);
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn search_emails_plans_imap_fetch_so_results_populate_search_view() {
        // Result: search view shows matched messages from host with query and filters visible.
        let response = call_export(
            search_emails,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "query": "budget review",
                "pagination": { "offset": 0, "limit": 50 }
            }),
        );

        assert_eq!(response["ok"]["operation"], "search_emails");
        assert_eq!(response["ok"]["data"]["query"], "budget review");
        assert_eq!(response["ok"]["data"]["needs_host_refresh"], true);
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_IMAP_FETCH));
    }

    #[test]
    fn get_unread_count_plans_imap_fetch_so_folder_badges_show_live_counts() {
        // Result: every folder in the sidebar shows its correct unread count.
        let response = call_export(
            get_unread_count,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "get_unread_count");
        assert_eq!(response["ok"]["data"]["needs_host_refresh"], true);
    }

    // ─── Compose and Send ─────────────────────────────────────────────────────
    // Intent: user can compose, save, send, reply, and forward mail.

    #[test]
    fn draft_email_returns_suggested_draft_id_so_compose_autosave_can_update_same_draft() {
        // Result: compose pane autosaves every 10 s and updates the same draft without creating duplicates.
        let response = call_export(
            draft_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "alice@example.com", "display_name": "Alice" }],
                    "subject": "Hello",
                    "body_text": "Hi there"
                }
            }),
        );

        assert_eq!(response["ok"]["operation"], "draft_email");
        let draft_id = response["ok"]["data"]["suggested_draft_id"].as_str().unwrap();
        assert!(!draft_id.is_empty(), "compose autosave requires a non-empty suggested draft id");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn update_draft_preserves_existing_draft_id_so_autosave_does_not_create_duplicate() {
        // Result: editing a saved draft updates the same draft, not a new one.
        let response = call_export(
            update_draft,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "draft_id": "draft-existing-001",
                    "account_id": "acc-1",
                    "to": [{ "email": "alice@example.com" }],
                    "subject": "Updated subject",
                    "body_text": "Updated body"
                }
            }),
        );

        assert_eq!(response["ok"]["operation"], "update_draft");
        assert_eq!(response["ok"]["data"]["suggested_draft_id"], "draft-existing-001");
    }

    #[test]
    fn discard_draft_plans_mail_store_delete_so_compose_closes_cleanly() {
        // Result: closing compose with discard removes the draft from Drafts folder.
        let response = call_export(
            discard_draft,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_DELETE]),
                "draft_id": "draft-existing-001",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "discard_draft");
        assert_eq!(response["ok"]["data"]["mutation"], "discard_draft");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_DELETE));
    }

    #[test]
    fn send_email_returns_undo_window_so_ui_can_show_undo_send_timer() {
        // Result: after sending, the UI shows an undo banner for the configured window.
        let response = call_export(
            send_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "draft_id": "draft-ready",
                "account_id": "acc-1",
                "undo_window_seconds": 10,
                "notify": false
            }),
        );

        assert_eq!(response["ok"]["operation"], "send_email");
        let undo = response["ok"]["data"]["undo_window_seconds"].as_u64().unwrap();
        assert!(undo > 0, "undo window must be non-zero so the UI can show the undo timer");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let effect_names: Vec<&str> = effects.iter().map(|e| e["effect"].as_str().unwrap()).collect();
        assert!(effect_names.contains(&EFFECT_SMTP_SEND));
        assert!(effect_names.contains(&EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn send_email_with_notify_adds_notification_effect_for_send_confirmation_toast() {
        // Result: sending with notify=true triggers a host notification so the UI can show "Sent".
        let response = call_export(
            send_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND, EFFECT_NOTIFY_USER]),
                "draft_id": "draft-notify",
                "account_id": "acc-1",
                "notify": true
            }),
        );

        assert_eq!(response["ok"]["operation"], "send_email");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_NOTIFY_USER),
            "notify=true must include a NotifyUser effect for the send confirmation toast");
    }

    #[test]
    fn reply_email_plans_source_read_then_send_so_thread_context_is_preserved() {
        // Result: reply loads original message headers so In-Reply-To and threading stay consistent.
        let response = call_export(
            reply_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_READ, EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "email_id": "email-5",
                "account_id": "acc-1",
                "body_text": "Thanks for your message.",
                "reply_all": false,
                "send_now": true
            }),
        );

        assert_eq!(response["ok"]["operation"], "reply_email");
        assert_eq!(response["ok"]["data"]["send_now"], true);
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let effect_names: Vec<&str> = effects.iter().map(|e| e["effect"].as_str().unwrap()).collect();
        // Source must be read first to preserve thread metadata.
        assert!(effect_names.contains(&EFFECT_MAIL_STORE_READ));
        assert!(effect_names.contains(&EFFECT_SMTP_SEND));
    }

    #[test]
    fn forward_email_returns_recipient_count_so_compose_summary_is_accurate() {
        // Result: forwarding to multiple recipients shows the correct count in the compose summary.
        let response = call_export(
            forward_email,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_READ, EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "email_id": "email-3",
                "account_id": "acc-1",
                "to": [
                    { "email": "bob@example.com" },
                    { "email": "carol@example.org" }
                ],
                "note_text": "FYI",
                "send_now": false
            }),
        );

        assert_eq!(response["ok"]["operation"], "forward_email");
        assert_eq!(response["ok"]["data"]["recipient_count"], 2);
    }

    // ─── Scheduled Send ───────────────────────────────────────────────────────
    // Intent: user can schedule a message and cancel it before it dispatches.

    #[test]
    fn schedule_send_returns_job_id_so_cancel_button_can_identify_the_queued_job() {
        // Result: scheduled message appears with a job ID so the cancel action can reference it.
        let response = call_export(
            schedule_send,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SCHEDULE_JOB]),
                "draft_id": "draft-scheduled",
                "account_id": "acc-1",
                "scheduled_for": 1_800_000_000_i64
            }),
        );

        assert_eq!(response["ok"]["operation"], "schedule_send");
        let job_id = response["ok"]["data"]["job_id"].as_str().unwrap();
        assert!(job_id.starts_with("job_"), "job_id must be prefixed so it is recognisable in the UI");
        assert_eq!(response["ok"]["data"]["draft_id"], "draft-scheduled");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let effect_names: Vec<&str> = effects.iter().map(|e| e["effect"].as_str().unwrap()).collect();
        assert!(effect_names.contains(&EFFECT_SCHEDULE_JOB));
        assert!(effect_names.contains(&EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn cancel_scheduled_send_plans_cancel_job_and_clears_draft_metadata_so_draft_becomes_editable() {
        // Result: cancelling a scheduled send re-opens the draft for editing without re-sending.
        let response = call_export(
            cancel_scheduled_send,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE, EFFECT_CANCEL_JOB]),
                "job_id": "job-abc123",
                "account_id": "acc-1",
                "draft_id": "draft-scheduled"
            }),
        );

        assert_eq!(response["ok"]["operation"], "cancel_scheduled_send");
        assert_eq!(response["ok"]["data"]["mutation"], "cancel_scheduled_send");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let effect_names: Vec<&str> = effects.iter().map(|e| e["effect"].as_str().unwrap()).collect();
        assert!(effect_names.contains(&EFFECT_CANCEL_JOB));
        assert!(effect_names.contains(&EFFECT_MAIL_STORE_WRITE),
            "metadata on the draft must be cleared so the UI restores it to editable state");
    }

    // ─── Organize ─────────────────────────────────────────────────────────────
    // Intent: user can flag, archive, delete, and move messages from list and reading pane.

    #[test]
    fn flag_email_plans_mail_store_write_and_echoes_resource_ids_for_optimistic_ui_update() {
        // Result: clicking the flag icon immediately updates the row and persists the change.
        let response = call_export(
            flag_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "flagged": true,
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "flag_email");
        assert_eq!(response["ok"]["data"]["mutation"], "flag_email");
        assert_eq!(response["ok"]["data"]["resource_ids"][0], "email-1");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn archive_email_plans_archive_destination_so_message_leaves_inbox_immediately() {
        // Result: archived message disappears from inbox; folder counts reconcile on effect completion.
        let response = call_export(
            archive_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-3"],
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "archive_email");
        assert_eq!(response["ok"]["data"]["mutation"], "archive_email");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        let payload = &effects[0]["payload"];
        assert_eq!(payload["destination_mailbox_id"], "ARCHIVE",
            "archive must target the ARCHIVE mailbox so the host moves the message correctly");
    }

    #[test]
    fn delete_email_soft_delete_moves_to_trash_using_write_effect() {
        // Result: deleted message moves to Trash and is recoverable until Trash is emptied.
        let response = call_export(
            delete_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE]),
                "email_ids": ["email-2"],
                "account_id": "acc-1",
                "permanent": false
            }),
        );

        assert_eq!(response["ok"]["operation"], "delete_email");
        assert_eq!(response["ok"]["data"]["mutation"], "delete_email");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE),
            "soft delete must use MailStoreWrite so the message moves to Trash, not a hard delete");
    }

    #[test]
    fn delete_email_permanent_flag_uses_delete_effect_for_hard_removal() {
        // Result: permanent delete bypasses Trash — user must confirm destructive action in UI.
        let response = call_export(
            delete_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE]),
                "email_ids": ["email-old"],
                "account_id": "acc-1",
                "permanent": true
            }),
        );

        assert_eq!(response["ok"]["data"]["mutation"], "delete_email_permanent");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_DELETE));
    }

    #[test]
    fn move_email_plans_destination_mailbox_so_message_appears_in_target_folder() {
        // Result: moved message disappears from current folder and appears in the target folder.
        let response = call_export(
            move_email,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-5"],
                "destination_mailbox_id": "projects",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "move_email");
        assert_eq!(response["ok"]["data"]["mutation"], "move_email");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert_eq!(effects[0]["payload"]["destination_mailbox_id"], "projects");
    }

    // ─── Folder Management ────────────────────────────────────────────────────
    // Intent: user can create, rename, and delete custom folders from the sidebar.

    #[test]
    fn create_folder_returns_deterministic_mailbox_id_so_sidebar_can_navigate_to_new_folder() {
        // Result: new folder appears in the sidebar immediately with a stable ID.
        let response = call_export(
            create_folder,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "account_id": "acc-1",
                "name": "Projects"
            }),
        );

        assert_eq!(response["ok"]["operation"], "create_folder");
        let mailbox_id = response["ok"]["data"]["mailbox_id"].as_str().unwrap();
        assert!(mailbox_id.starts_with("fld_"),
            "folder id must be prefixed so the sidebar can distinguish custom folders");
        assert_eq!(response["ok"]["data"]["name"], "Projects");
    }

    #[test]
    fn rename_folder_plans_mail_store_write_and_returns_updated_name_for_sidebar_refresh() {
        // Result: folder title updates in the sidebar after rename without a full reload.
        let response = call_export(
            rename_folder,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "account_id": "acc-1",
                "mailbox_id": "fld-custom-1",
                "new_name": "Work Projects"
            }),
        );

        assert_eq!(response["ok"]["operation"], "rename_folder");
        assert_eq!(response["ok"]["data"]["name"], "Work Projects");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn delete_folder_plans_mail_store_delete_so_sidebar_removes_folder() {
        // Result: deleted folder disappears from the sidebar; contained messages are handled by host.
        let response = call_export(
            delete_folder,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_DELETE]),
                "account_id": "acc-1",
                "mailbox_id": "fld-old-project"
            }),
        );

        assert_eq!(response["ok"]["operation"], "delete_folder");
        assert_eq!(response["ok"]["data"]["mutation"], "delete_folder");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_DELETE));
    }

    // ─── Rules ────────────────────────────────────────────────────────────────
    // Intent: user can create and manage mail automation rules from Settings → Rules.

    fn valid_rule_payload() -> Value {
        json!({
            "rule_id": "rule-1",
            "name": "Move newsletters",
            "enabled": true,
            "stop_processing": false,
            "priority": 1,
            "conditions": [{ "field": "from", "operator": "contains", "value": "newsletter" }],
            "actions": [{ "action_type": "move_to", "value": "newsletters" }]
        })
    }

    #[test]
    fn create_rule_with_valid_conditions_plans_mail_store_write_so_rule_applies_to_future_mail() {
        // Result: saved rule appears in the Rules list and affects incoming messages per its conditions.
        let response = call_export(
            create_rule,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_WRITE]),
                "account_id": "acc-1",
                "rule": valid_rule_payload()
            }),
        );

        assert_eq!(response["ok"]["operation"], "create_rule");
        assert_eq!(response["ok"]["data"]["rule_id"], "rule-1");
        assert_eq!(response["ok"]["data"]["name"], "Move newsletters");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_WRITE));
    }

    #[test]
    fn list_rules_triggers_host_refresh_when_no_snapshot_so_rules_settings_shows_current_list() {
        // Result: Rules section in settings shows all saved rules after opening.
        let response = call_export(
            list_rules,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_READ]),
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "list_rules");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(!effects.is_empty(), "host fetch effect must be planned so the rules list populates");
    }

    #[test]
    fn delete_rule_plans_mail_store_delete_so_rule_is_removed_from_settings_list() {
        // Result: deleted rule disappears from the Rules settings section immediately.
        let response = call_export(
            delete_rule,
            json!({
                "context": context("organize", &[EFFECT_MAIL_STORE_DELETE]),
                "account_id": "acc-1",
                "rule_id": "rule-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "delete_rule");
        assert_eq!(response["ok"]["data"]["mutation"], "delete_rule");
        let effects = response["ok"]["host_effects"].as_array().unwrap();
        assert!(effects.iter().any(|e| e["effect"] == EFFECT_MAIL_STORE_DELETE));
    }

    // ─── Identities and Signatures ────────────────────────────────────────────
    // Intent: user can manage sender aliases and signatures from Settings → Identities.

    fn valid_identity() -> Value {
        json!({
            "identity_id": "id-1",
            "account_id": "acc-1",
            "kind": "alias",
            "display_name": "Work Account",
            "email_address": "work@example.com",
            "enabled": true,
            "is_default_for_account": false,
            "is_global_default": false,
            "verification_state": "pending"
        })
    }

    fn valid_signature() -> Value {
        json!({
            "signature_id": "sig-1",
            "account_id": "acc-1",
            "label": "Work Signature",
            "body_text": "Best regards,\nWork Team",
            "enabled": true,
            "use_for_new": true,
            "use_for_replies": false,
            "use_for_forwards": false
        })
    }

    #[test]
    fn list_identities_returns_snapshot_with_signatures_so_compose_from_selector_can_populate() {
        // Result: From selector in compose shows all configured aliases with their signatures.
        let response = call_export(
            list_identities,
            json!({
                "context": context("accounts", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_MAIL_STORE_READ])
            }),
        );

        assert_eq!(response["ok"]["operation"], "list_identities");
        assert!(response["ok"]["data"]["identities"].is_array());
        assert!(response["ok"]["data"]["signatures"].is_array());
    }

    #[test]
    fn save_identity_plans_mail_store_write_so_new_alias_appears_in_compose_from_selector() {
        // Result: saved alias is immediately selectable in the compose From dropdown.
        let response = call_export(
            save_identity,
            json!({
                "context": context("accounts", &[EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE]),
                "identity": valid_identity()
            }),
        );

        assert_eq!(response["ok"]["operation"], "save_identity");
        assert_eq!(response["ok"]["data"]["identity"]["identity_id"], "id-1");
    }

    #[test]
    fn delete_identity_removes_alias_from_compose_from_selector() {
        // Result: deleted alias no longer appears in compose From or account identity list.
        let response = call_export(
            delete_identity,
            json!({
                "context": context("accounts", &[EFFECT_ACCOUNT_CREDENTIAL_WRITE, EFFECT_MAIL_STORE_WRITE]),
                "identity_id": "id-1",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "delete_identity");
        assert_eq!(response["ok"]["data"]["identity_id"], "id-1");
    }

    #[test]
    fn save_signature_plans_mail_store_write_so_signature_appears_in_compose() {
        // Result: saved signature is inserted in compose when the matching identity or context is selected.
        let response = call_export(
            save_signature,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_WRITE]),
                "signature": valid_signature()
            }),
        );

        assert_eq!(response["ok"]["operation"], "save_signature");
        assert_eq!(response["ok"]["data"]["signature"]["signature_id"], "sig-1");
    }

    #[test]
    fn delete_signature_removes_it_from_compose_and_identity_association() {
        // Result: deleted signature no longer appears in compose and identity cards update.
        let response = call_export(
            delete_signature,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_WRITE, EFFECT_MAIL_STORE_DELETE]),
                "signature_id": "sig-1",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["ok"]["operation"], "delete_signature");
        assert_eq!(response["ok"]["data"]["signature_id"], "sig-1");
    }

    // ─── Preferences ─────────────────────────────────────────────────────────
    // Intent: user can personalize reading, compose, notification, and sync behavior.

    #[test]
    fn get_user_preferences_returns_defaults_when_no_snapshot_supplied() {
        // Result: opening Preferences settings shows sensible defaults ready to customize.
        let response = call_export(
            get_user_preferences,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_READ]),
                "include_defaults": true
            }),
        );

        assert_eq!(response["ok"]["operation"], "get_user_preferences");
        assert!(response["ok"]["data"]["preferences"]["reading"].is_object());
        assert!(response["ok"]["data"]["preferences"]["compose"].is_object());
        assert!(response["ok"]["data"]["preferences"]["notifications"].is_object());
        assert!(response["ok"]["data"]["preferences"]["sync_defaults"].is_object());
        assert_eq!(response["ok"]["data"]["defaults_applied"], true);
    }

    #[test]
    fn save_user_preferences_within_valid_bounds_acknowledges_write_so_changes_take_effect() {
        // Result: saved preferences change visible behavior — page size, pane position, undo window.
        let response = call_export(
            save_user_preferences,
            json!({
                "context": context("accounts", &[EFFECT_MAIL_STORE_WRITE]),
                "preferences": {
                    "reading": { "page_size": 50 },
                    "compose": { "undo_send_delay_seconds": 10 },
                    "notifications": { "enabled": true },
                    "sync_defaults": { "interval_minutes": 5 }
                }
            }),
        );

        assert_eq!(response["ok"]["operation"], "save_user_preferences");
        assert!(response["ok"]["data"]["preferences"].is_object());
    }

    // ─── Account Setup ────────────────────────────────────────────────────────
    // Intent: user can add and remove mail accounts including receive-only accounts.

    #[test]
    fn complete_account_setup_receive_only_skips_smtp_requirement_so_user_can_read_without_sending() {
        // Result: an account with only IMAP configured is saved as receive-only; send actions are disabled.
        let mut account = account_payload();
        account["desired_status"] = json!("receive_only");
        // Remove outgoing config to simulate SMTP failure / intentional receive-only.
        account.as_object_mut().unwrap().remove("outgoing");

        let response = call_export(
            complete_account_setup,
            json!({
                "context": context("accounts", &[
                    EFFECT_ACCOUNT_CREDENTIAL_WRITE,
                    EFFECT_STORE_PLATFORM_SECRET,
                    EFFECT_MAIL_STORE_WRITE,
                    EFFECT_IMAP_SYNC
                ]),
                "account": account,
                "save_mode": "receive_only",
                "connection_test_result": {
                    "test_id": "test-001",
                    "steps": [
                        { "step": "imap_auth", "state": "passed" },
                        { "step": "imap_mailbox_discovery", "state": "passed" }
                    ]
                }
            }),
        );

        assert_eq!(response["ok"]["operation"], "complete_account_setup");
        assert_eq!(response["ok"]["data"]["status"], "receive_only",
            "account must be saved as receive_only so compose and send are disabled for this account");
    }

    #[test]
    fn remove_account_plans_credential_and_cache_cleanup_so_nothing_is_left_behind() {
        // Result: removed account disappears from the account list with scheduled sends and cache cleared.
        let response = call_export(
            remove_account,
            json!({
                "context": context("accounts", &[
                    EFFECT_ACCOUNT_CREDENTIAL_WRITE,
                    EFFECT_MAIL_STORE_WRITE,
                    EFFECT_MAIL_STORE_DELETE,
                    EFFECT_CANCEL_JOB
                ]),
                "account_id": "acc-1",
                "remove_local_cache": true,
                "cancel_scheduled_sends": true
            }),
        );

        assert_eq!(response["ok"]["operation"], "remove_account");
        assert_eq!(response["ok"]["data"]["account_id"], "acc-1");
        let cleanup = response["ok"]["data"]["cleanup_planned"].as_array().unwrap();
        assert!(!cleanup.is_empty(), "cleanup_planned must list what will be removed");
    }

    // ─── Permission Completeness ──────────────────────────────────────────────
    // Intent: the permission model is strictly enforced — every operation rejects under-privileged callers.

    #[test]
    fn read_permission_cannot_draft_email() {
        // Result: an agent with mail:read cannot create drafts — must have at least mail:draft.
        let response = call_export(
            draft_email,
            json!({
                "context": context("read", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "bob@example.com" }],
                    "subject": "Test",
                    "body_text": "Body"
                }
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn draft_permission_cannot_send_email() {
        // Result: an agent with mail:draft cannot send — prevents accidental send from automation.
        let response = call_export(
            send_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE, EFFECT_SMTP_SEND]),
                "draft_id": "draft-1",
                "account_id": "acc-1"
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn none_permission_cannot_read_emails() {
        // Result: unauthenticated or no-permission context is completely blocked from inbox.
        let response = call_export(
            read_emails,
            json!({
                "context": context("none", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "mailbox_id": "inbox"
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    #[test]
    fn admin_permission_satisfies_all_access_levels() {
        // Result: admin context can call read, draft, send, and organize operations.
        let read_res = call_export(
            read_emails,
            json!({
                "context": context("admin", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "mailbox_id": "inbox"
            }),
        );
        assert_eq!(read_res["ok"]["operation"], "read_emails");

        let draft_res = call_export(
            draft_email,
            json!({
                "context": context("admin", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "alice@example.com" }],
                    "subject": "Test",
                    "body_text": "Body"
                }
            }),
        );
        assert_eq!(draft_res["ok"]["operation"], "draft_email");

        let archive_res = call_export(
            archive_email,
            json!({
                "context": context("admin", &[EFFECT_MAIL_STORE_WRITE]),
                "email_ids": ["email-1"],
                "account_id": "acc-1"
            }),
        );
        assert_eq!(archive_res["ok"]["operation"], "archive_email");
    }

    #[test]
    fn send_permission_cannot_organize_folders() {
        // Result: a send-only agent cannot create or delete folders — organize must be separate.
        let response = call_export(
            create_folder,
            json!({
                "context": context("send", &[EFFECT_MAIL_STORE_WRITE]),
                "account_id": "acc-1",
                "name": "NewFolder"
            }),
        );

        assert_eq!(response["err"]["code"], "PermissionDenied");
    }

    // ─── Boundary Validation ─────────────────────────────────────────────────
    // Intent: the system enforces safe data limits before forwarding to the host.

    #[test]
    fn draft_email_rejects_body_exceeding_max_length_before_writing_to_host() {
        // Result: oversized message body is rejected with a clear error before any storage occurs.
        let large_body = "x".repeat(MAX_BODY_LENGTH + 1);
        let response = call_export(
            draft_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "alice@example.com" }],
                    "subject": "Big",
                    "body_text": large_body
                }
            }),
        );

        assert_eq!(response["err"]["code"], "InvalidInput",
            "body exceeding MAX_BODY_LENGTH must be rejected before host storage");
    }

    #[test]
    fn draft_email_rejects_too_many_recipients_to_prevent_spam_from_automation() {
        // Result: drafts with more than MAX_RECIPIENTS recipients are blocked at validation time.
        let recipients: Vec<Value> = (0..=MAX_RECIPIENTS)
            .map(|i| json!({ "email": format!("user{i}@example.com") }))
            .collect();
        let response = call_export(
            draft_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": recipients,
                    "subject": "Mass send",
                    "body_text": "Body"
                }
            }),
        );

        assert_eq!(response["err"]["code"], "TooManyRecipients");
    }

    #[test]
    fn draft_email_rejects_overlong_subject_before_writing_to_host() {
        // Result: subjects violating RFC 5322 length limits are rejected with a clear validation error.
        let long_subject = "s".repeat(MAX_SUBJECT_LENGTH + 1);
        let response = call_export(
            draft_email,
            json!({
                "context": context("draft", &[EFFECT_MAIL_STORE_WRITE]),
                "draft": {
                    "account_id": "acc-1",
                    "to": [{ "email": "alice@example.com" }],
                    "subject": long_subject,
                    "body_text": "Body"
                }
            }),
        );

        assert_eq!(response["err"]["code"], "InvalidInput",
            "subject exceeding MAX_SUBJECT_LENGTH must be rejected");
    }

    #[test]
    fn read_emails_rejects_page_size_above_maximum_to_prevent_oversized_host_queries() {
        // Result: pagination is bounded so the host is never asked for more than MAX_PAGE_SIZE rows.
        let response = call_export(
            read_emails,
            json!({
                "context": context("read", &[EFFECT_ACCOUNT_CREDENTIAL_READ, EFFECT_IMAP_FETCH]),
                "mailbox_id": "inbox",
                "pagination": { "offset": 0, "limit": MAX_PAGE_SIZE + 1 }
            }),
        );

        assert_eq!(response["err"]["code"], "InvalidInput",
            "limit above MAX_PAGE_SIZE must be rejected to protect host performance");
    }
}
