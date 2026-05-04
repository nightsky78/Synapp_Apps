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
const MAX_BODY_LENGTH: usize = 512_000;
const MAX_ALLOWED_UNDO_SECONDS: u32 = 30;

const EFFECT_ACCOUNT_CREDENTIAL_READ: &str = "AccountCredentialRead";
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
}

impl Permission {
    fn parse(value: &str) -> Result<Self, ErrorResponse> {
        match value {
            "none" => Ok(Self::None),
            "read" => Ok(Self::Read),
            "draft" => Ok(Self::Draft),
            "send" => Ok(Self::Send),
            "organize" => Ok(Self::Organize),
            _ => Err(ErrorResponse::new(
                "InvalidInput",
                format!("Unknown permission level: {value}"),
            )),
        }
    }

    fn satisfies(self, required: Self) -> bool {
        match self {
            Self::Organize => true,
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

    fn idempotency_key(&self, operation: &'static str, seed: &str) -> String {
        if let Some(request_id) = &self.request_id {
            // Prepend user_id so a caller-supplied request_id can never collide
            // with another user's idempotency key on the host.
            return format!("{operation}:{}:{request_id}", self.user_id);
        }

        format!("{operation}:{}", hash_text(&format!("{}:{seed}", self.user_id)))
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

fn validate_email(email: &str) -> Result<(), ErrorResponse> {
    if email.len() > 320
        || !email.contains('@')
        || email.starts_with('@')
        || email.ends_with('@')
        || email.contains(' ')
    {
        return Err(ErrorResponse::new(
            "InvalidRecipient",
            format!("Invalid email address: {email}"),
        ));
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
    ensure_non_empty("sort.field", &sort.field)?;
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
        ensure_non_empty("email_ids[]", id)?;
    }
    Ok(())
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
    HostEffect {
        effect,
        intent,
        payload,
        idempotency_key: context.idempotency_key(operation, effect),
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
        format!("draft_{}", context.idempotency_key("draft", &seed).replace(':', "_"))
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
    ensure_non_empty("account_id", &request.account_id)?;
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
        ensure_non_empty("account_id", account_id)?;
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
    if let Some(account_id) = &request.account_id {
        ensure_non_empty("account_id", account_id)?;
    }
    let page = validate_pagination(request.pagination.clone())?;
    let sort = validate_sort(request.sort.clone())?;
    if let Some(importance) = &request.filters.importance {
        validate_importance(importance)?;
    }

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
    ensure_non_empty("query", &request.query)?;
    let page = validate_pagination(request.pagination.clone())?;
    if let Some(importance) = &request.filters.importance {
        validate_importance(importance)?;
    }
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
    ensure_non_empty("email_id", &request.email_id)?;
    if let Some(account_id) = &request.account_id {
        ensure_non_empty("account_id", account_id)?;
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
    ensure_non_empty("thread_id", &request.thread_id)?;
    if let Some(account_id) = &request.account_id {
        ensure_non_empty("account_id", account_id)?;
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
    ensure_non_empty("draft_id", &request.draft_id)?;
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
    ensure_non_empty("draft_id", &request.draft_id)?;
    ensure_non_empty("account_id", &request.account_id)?;
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
    ensure_non_empty("email_id", &request.email_id)?;
    ensure_non_empty("account_id", &request.account_id)?;
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
    ensure_non_empty("email_id", &request.email_id)?;
    ensure_non_empty("account_id", &request.account_id)?;
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
    ensure_non_empty("destination_mailbox_id", &request.destination_mailbox_id)?;
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
    ensure_non_empty("account_id", &request.account_id)?;
    ensure_non_empty("name", &request.name)?;
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
    ensure_non_empty("account_id", &request.account_id)?;
    ensure_non_empty("mailbox_id", &request.mailbox_id)?;
    ensure_non_empty("new_name", &request.new_name)?;
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
    ensure_non_empty("account_id", &request.account_id)?;
    ensure_non_empty("mailbox_id", &request.mailbox_id)?;
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
        ensure_non_empty("account_id", account_id)?;
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
    ensure_non_empty("account_id", &request.account_id)?;
    ensure_non_empty("rule.name", &request.rule.name)?;
    if request.rule.name.len() > MAX_RULE_NAME_LENGTH {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("Rule names must be at most {MAX_RULE_NAME_LENGTH} characters"),
        ));
    }
    if request.rule.conditions.is_empty() || request.rule.actions.is_empty() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "Rules require at least one condition and one action",
        ));
    }
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
    ensure_non_empty("account_id", &request.account_id)?;
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
    ensure_non_empty("account_id", &request.account_id)?;
    ensure_non_empty("rule_id", &request.rule_id)?;
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
    ensure_non_empty("draft_id", &request.draft_id)?;
    ensure_non_empty("account_id", &request.account_id)?;
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
    ensure_non_empty("job_id", &request.job_id)?;
    ensure_non_empty("account_id", &request.account_id)?;
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

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
