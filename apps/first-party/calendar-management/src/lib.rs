use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::ffi::{c_char, CString};

const MAX_TITLE_CHARS: usize = 512;
const MAX_BODY_CHARS: usize = 512_000;
const MAX_LOCATION_CHARS: usize = 512;
const MAX_ATTENDEES: usize = 500;
const MAX_CATEGORIES: usize = 25;
const MAX_REMINDERS: usize = 10;
const MAX_SEARCH_QUERY_CHARS: usize = 256;
const MAX_RECURRENCE_PREVIEW: usize = 50;
const DEFAULT_TIME_ZONE: &str = "UTC";

const DELETE_CALENDAR_MODES: &[&str] = &["delete", "hide", "unsubscribe"];
const EVENT_SCOPES: &[&str] = &["single", "occurrence", "this_and_following", "series"];
const INVITATION_SCOPES: &[&str] = &["occurrence", "series"];
const INVITATION_RESPONSES: &[&str] = &["accept", "tentative", "decline"];
const EVENT_DELETE_MODES: &[&str] = &["delete", "cancel"];
const NOTIFICATION_SCOPES: &[&str] =
    &["none", "changed_attendees", "all_attendees", "host_default"];
const REMINDER_ACTIONS: &[&str] = &["snooze", "dismiss"];
const RAW_SUBSCRIPTION_FIELDS: &[&str] = &[
    "url",
    "uri",
    "host",
    "hostname",
    "path",
    "token",
    "password",
    "client_secret",
    "access_token",
    "refresh_token",
    "caldav_password",
];

const EFFECT_CALENDAR_STORE_READ: &str = "CalendarStoreRead";
const EFFECT_CALENDAR_STORE_WRITE: &str = "CalendarStoreWrite";
const EFFECT_CALENDAR_STORE_DELETE: &str = "CalendarStoreDelete";
const EFFECT_CALENDAR_INVITE_SEND: &str = "CalendarInviteSend";
const EFFECT_CALENDAR_INVITE_RESPOND: &str = "CalendarInviteRespond";
const EFFECT_FREE_BUSY_LOOKUP: &str = "FreeBusyLookup";
const EFFECT_ROOM_RESOURCE_LOOKUP: &str = "RoomResourceLookup";
const EFFECT_CONTACT_LOOKUP: &str = "ContactLookup";
const EFFECT_CONFERENCE_LINK_CREATE: &str = "ConferenceLinkCreate";
const EFFECT_REMINDER_SCHEDULE: &str = "ReminderSchedule";
const EFFECT_CALENDAR_SHARE_MANAGE: &str = "CalendarShareManage";
const EFFECT_SUBSCRIPTION_SYNC: &str = "SubscriptionSync";
const EFFECT_OFFLINE_CACHE_READ: &str = "OfflineCacheRead";
const EFFECT_AUDIT_READ: &str = "AuditRead";
const EFFECT_AUDIT_WRITE: &str = "AuditWrite";
const EFFECT_NOTIFY_USER: &str = "NotifyUser";

const ALL_EFFECTS: &[&str] = &[
    EFFECT_CALENDAR_STORE_READ,
    EFFECT_CALENDAR_STORE_WRITE,
    EFFECT_CALENDAR_STORE_DELETE,
    EFFECT_CALENDAR_INVITE_SEND,
    EFFECT_CALENDAR_INVITE_RESPOND,
    EFFECT_FREE_BUSY_LOOKUP,
    EFFECT_ROOM_RESOURCE_LOOKUP,
    EFFECT_CONTACT_LOOKUP,
    EFFECT_CONFERENCE_LINK_CREATE,
    EFFECT_REMINDER_SCHEDULE,
    EFFECT_CALENDAR_SHARE_MANAGE,
    EFFECT_SUBSCRIPTION_SYNC,
    EFFECT_OFFLINE_CACHE_READ,
    EFFECT_AUDIT_READ,
    EFFECT_AUDIT_WRITE,
    EFFECT_NOTIFY_USER,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Permission {
    None,
    Read,
    Write,
    Invite,
    Respond,
    Share,
    Delegate,
    Settings,
    Audit,
    Admin,
}

impl Permission {
    fn parse(value: &str) -> Result<Self, ErrorResponse> {
        match value {
            "none" => Ok(Self::None),
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "invite" => Ok(Self::Invite),
            "respond" => Ok(Self::Respond),
            "share" => Ok(Self::Share),
            "delegate" => Ok(Self::Delegate),
            "settings" => Ok(Self::Settings),
            "audit" => Ok(Self::Audit),
            "admin" => Ok(Self::Admin),
            _ => Err(ErrorResponse::new(
                "InvalidInput",
                format!("Unknown calendar permission level: {value}"),
            )),
        }
    }

    fn satisfies(self, required: Self) -> bool {
        match self {
            Self::Admin => true,
            Self::Delegate => matches!(
                required,
                Self::None
                    | Self::Read
                    | Self::Write
                    | Self::Invite
                    | Self::Respond
                    | Self::Share
                    | Self::Delegate
            ),
            Self::Invite => matches!(required, Self::None | Self::Read | Self::Invite),
            Self::Write => matches!(required, Self::None | Self::Read | Self::Write),
            Self::Respond => matches!(required, Self::None | Self::Read | Self::Respond),
            Self::Share => matches!(required, Self::None | Self::Read | Self::Share),
            Self::Settings => matches!(required, Self::None | Self::Read | Self::Settings),
            Self::Audit => matches!(required, Self::None | Self::Read | Self::Audit),
            Self::Read => matches!(required, Self::None | Self::Read),
            Self::None => matches!(required, Self::None),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
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

#[derive(Serialize, Debug, Clone)]
struct WarningMessage {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

impl WarningMessage {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
struct HostEffect {
    effect: &'static str,
    intent: String,
    payload: Value,
    idempotency_key: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
    on_failure: &'static str,
    audit: Value,
}

#[derive(Serialize, Debug, Clone)]
struct OperationResponse {
    operation: &'static str,
    status: &'static str,
    data: Value,
    host_effects: Vec<HostEffect>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<WarningMessage>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum AppResult {
    Ok { ok: OperationResponse },
    Err { err: ErrorResponse },
}

#[derive(Deserialize, Debug, Clone)]
struct RequestContext {
    user_id: String,
    permission: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    available_effects: Vec<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    locale: Option<String>,
    #[serde(default)]
    time_zone: Option<String>,
    #[serde(default)]
    policy: Option<Value>,
    #[serde(default)]
    host_capabilities: Option<Value>,
    #[serde(default)]
    audit_correlation_id: Option<String>,
}

impl RequestContext {
    fn from_request(request: &Value) -> Result<Self, ErrorResponse> {
        let context = request.get("context").ok_or_else(|| {
            ErrorResponse::new("InvalidInput", "Missing required context object.")
        })?;
        let parsed: RequestContext = serde_json::from_value(context.clone()).map_err(|error| {
            ErrorResponse::new("InvalidInput", format!("Invalid context object: {error}"))
        })?;
        if parsed.user_id.trim().is_empty() {
            return Err(ErrorResponse::new(
                "InvalidInput",
                "context.user_id must be a non-empty host user id.",
            ));
        }
        Permission::parse(&parsed.permission)?;
        Ok(parsed)
    }

    fn permission(&self) -> Result<Permission, ErrorResponse> {
        Permission::parse(&self.permission)
    }

    fn has_effect(&self, effect: &str) -> bool {
        self.available_effects
            .iter()
            .any(|candidate| candidate == effect)
    }

    fn time_zone(&self) -> &str {
        self.time_zone.as_deref().unwrap_or(DEFAULT_TIME_ZONE)
    }

    fn idempotency_key(&self, operation: &'static str, effect: &str, seed: &Value) -> String {
        let stable = stable_json(seed);
        let hash = hash_text(&stable);
        match &self.request_id {
            Some(request_id) => {
                format!("{operation}:{}:{effect}:{hash}:{request_id}", self.user_id)
            }
            None => format!("{operation}:{}:{effect}:{hash}", self.user_id),
        }
    }
}

fn require_permission(context: &RequestContext, required: Permission) -> Result<(), ErrorResponse> {
    let actual = context.permission()?;
    if actual.satisfies(required) {
        Ok(())
    } else {
        Err(ErrorResponse::new(
            "PermissionDenied",
            "The supplied calendar context does not grant the required permission for this operation.",
        )
        .with_details(json!({ "required_permission": format_permission(required), "actual_permission": context.permission })))
    }
}

fn require_effect(context: &RequestContext, effect: &'static str) -> Result<(), ErrorResponse> {
    if context.has_effect(effect) {
        Ok(())
    } else {
        Err(ErrorResponse::new(
            "EffectUnavailable",
            format!("Host effect {effect} is required but was not advertised in context.available_effects."),
        )
        .with_details(json!({ "effect": effect })))
    }
}

fn require_effects(
    context: &RequestContext,
    effects: &[&'static str],
) -> Result<(), ErrorResponse> {
    for effect in effects {
        require_effect(context, effect)?;
    }
    Ok(())
}

fn optional_effect(
    context: &RequestContext,
    operation: &'static str,
    effect: &'static str,
    intent: &str,
    payload: Value,
    risk: &'static str,
) -> Option<HostEffect> {
    context
        .has_effect(effect)
        .then(|| build_effect(context, operation, effect, intent, payload, risk))
}

fn build_effect(
    context: &RequestContext,
    operation: &'static str,
    effect: &'static str,
    intent: &str,
    payload: Value,
    risk: &'static str,
) -> HostEffect {
    let audit = json!({
        "operation": operation,
        "risk": risk,
        "agent_id": context.agent_id,
        "correlation_id": context.audit_correlation_id,
    });
    HostEffect {
        effect,
        intent: intent.to_string(),
        idempotency_key: context.idempotency_key(operation, effect, &payload),
        payload,
        depends_on: Vec::new(),
        on_failure: "abort_remaining",
        audit,
    }
}

fn ok(operation: &'static str, data: Value, host_effects: Vec<HostEffect>) -> OperationResponse {
    OperationResponse {
        operation,
        status: "accepted",
        data,
        host_effects,
        warnings: Vec::new(),
    }
}

fn ok_with_warnings(
    operation: &'static str,
    data: Value,
    host_effects: Vec<HostEffect>,
    warnings: Vec<WarningMessage>,
) -> OperationResponse {
    OperationResponse {
        operation,
        status: "accepted",
        data,
        host_effects,
        warnings,
    }
}

fn require_field<'a>(request: &'a Value, field: &str) -> Result<&'a Value, ErrorResponse> {
    request.get(field).ok_or_else(|| {
        ErrorResponse::new(
            "InvalidInput",
            format!("Missing required field `{field}` for this calendar operation."),
        )
    })
}

fn require_non_empty_str<'a>(request: &'a Value, field: &str) -> Result<&'a str, ErrorResponse> {
    let value = require_field(request, field)?.as_str().ok_or_else(|| {
        ErrorResponse::new("InvalidInput", format!("Field `{field}` must be a string."))
    })?;
    if value.trim().is_empty() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("Field `{field}` must not be empty."),
        ));
    }
    Ok(value)
}

fn require_enum_str<'a>(
    request: &'a Value,
    field: &str,
    allowed: &[&str],
) -> Result<&'a str, ErrorResponse> {
    let value = require_non_empty_str(request, field)?;
    validate_enum_value(field, value, allowed)?;
    Ok(value)
}

fn optional_enum_str<'a>(
    request: &'a Value,
    field: &str,
    default_value: &'static str,
    allowed: &[&str],
) -> Result<&'a str, ErrorResponse> {
    match request.get(field) {
        Some(value) => {
            let value = value.as_str().ok_or_else(|| {
                ErrorResponse::new("InvalidInput", format!("Field `{field}` must be a string."))
            })?;
            if value.trim().is_empty() {
                return Err(ErrorResponse::new(
                    "InvalidInput",
                    format!("Field `{field}` must not be empty."),
                ));
            }
            validate_enum_value(field, value, allowed)?;
            Ok(value)
        }
        None => Ok(default_value),
    }
}

fn validate_enum_value(field: &str, value: &str, allowed: &[&str]) -> Result<(), ErrorResponse> {
    if allowed.iter().any(|allowed_value| *allowed_value == value) {
        Ok(())
    } else {
        Err(ErrorResponse::new(
            "InvalidInput",
            format!("Field `{field}` contains unsupported value `{value}`."),
        )
        .with_details(json!({ "field": field, "allowed_values": allowed })))
    }
}

fn require_array<'a>(request: &'a Value, field: &str) -> Result<&'a Vec<Value>, ErrorResponse> {
    require_field(request, field)?.as_array().ok_or_else(|| {
        ErrorResponse::new("InvalidInput", format!("Field `{field}` must be an array."))
    })
}

fn get_bool(request: &Value, field: &str, default_value: bool) -> bool {
    request
        .get(field)
        .and_then(Value::as_bool)
        .unwrap_or(default_value)
}

fn confirmation_confirmed(request: &Value) -> bool {
    request
        .get("confirmation")
        .and_then(|confirmation| confirmation.get("confirmed"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn require_confirmation(request: &Value, message: &str) -> Result<(), ErrorResponse> {
    if confirmation_confirmed(request) {
        Ok(())
    } else {
        Err(
            ErrorResponse::new("DestructiveConfirmationRequired", message).with_details(json!({
                "required_confirmation": true
            })),
        )
    }
}

fn validate_host_ref(value: &str, field_path: &str) -> Result<(), ErrorResponse> {
    if value.trim().is_empty() || value.chars().count() > 128 || value.contains("://") {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field_path} must be a non-empty host-managed reference, not a raw URL."),
        ));
    }
    if !value.chars().all(|candidate| {
        candidate.is_ascii_alphanumeric() || matches!(candidate, '.' | '_' | ':' | '/' | '@' | '-')
    }) {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field_path} contains characters outside the host reference contract."),
        ));
    }
    Ok(())
}

fn validate_subscription_value(value: &Value, field_path: &str) -> Result<(), ErrorResponse> {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if RAW_SUBSCRIPTION_FIELDS
                    .iter()
                    .any(|field| field.eq_ignore_ascii_case(key))
                {
                    return Err(ErrorResponse::new(
                        "InvalidInput",
                        format!("subscription contains forbidden raw provider field `{key}`."),
                    ));
                }
                validate_subscription_value(nested, &format!("{field_path}.{key}"))?;
            }
        }
        Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                validate_subscription_value(nested, &format!("{field_path}[{index}]"))?;
            }
        }
        Value::String(text) if text.contains("://") => {
            return Err(ErrorResponse::new(
                "InvalidInput",
                format!("{field_path} must not contain a raw URL."),
            ));
        }
        _ => {}
    }
    Ok(())
}

fn sanitize_subscription_request(request: &Value) -> Result<Value, ErrorResponse> {
    let subscription = require_field(request, "subscription")?;
    let object = subscription.as_object().ok_or_else(|| {
        ErrorResponse::new(
            "InvalidInput",
            "subscription must be an object with host-managed references.",
        )
    })?;
    validate_subscription_value(subscription, "subscription")?;
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "subscription_ref" | "url_ref" | "display_name" | "color"
        ) {
            return Err(ErrorResponse::new(
                "InvalidInput",
                format!("subscription contains unsupported field `{key}`."),
            ));
        }
    }
    let subscription_ref = object.get("subscription_ref").and_then(Value::as_str);
    let url_ref = object.get("url_ref").and_then(Value::as_str);
    if subscription_ref.is_none() && url_ref.is_none() {
        return Err(ErrorResponse::new("InvalidInput", "subscription.url_ref or subscription.subscription_ref is required; raw provider credentials are not accepted."));
    }
    if let Some(value) = subscription_ref {
        validate_host_ref(value, "subscription.subscription_ref")?;
    }
    if let Some(value) = url_ref {
        validate_host_ref(value, "subscription.url_ref")?;
    }
    let mut sanitized = Map::new();
    if let Some(value) = subscription_ref {
        sanitized.insert("subscription_ref".to_string(), json!(value));
    }
    if let Some(value) = url_ref {
        sanitized.insert("url_ref".to_string(), json!(value));
    }
    if let Some(display_name) = object.get("display_name") {
        if display_name
            .as_str()
            .map_or(true, |text| text.chars().count() > MAX_TITLE_CHARS)
        {
            return Err(ErrorResponse::new(
                "InvalidInput",
                "subscription.display_name must be a string of 512 characters or fewer.",
            ));
        }
        sanitized.insert("display_name".to_string(), display_name.clone());
    }
    if let Some(color) = object.get("color") {
        if color.as_str().map_or(true, |text| {
            text.chars().count() > 64 || text.contains("://")
        }) {
            return Err(ErrorResponse::new(
                "InvalidInput",
                "subscription.color must be a host-safe color string.",
            ));
        }
        sanitized.insert("color".to_string(), color.clone());
    }
    Ok(Value::Object(sanitized))
}

fn validate_time_range(
    range: &Value,
    field_path: &str,
) -> Result<(DateTimeParts, DateTimeParts), ErrorResponse> {
    let start = range.get("start").and_then(Value::as_str).ok_or_else(|| {
        ErrorResponse::new("InvalidInput", format!("{field_path}.start is required."))
    })?;
    let end = range.get("end").and_then(Value::as_str).ok_or_else(|| {
        ErrorResponse::new("InvalidInput", format!("{field_path}.end is required."))
    })?;
    if range
        .get("time_zone")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field_path}.time_zone is required for calendar time calculations."),
        ));
    }
    let start_parts = parse_datetime(start).ok_or_else(|| {
        ErrorResponse::new(
            "InvalidInput",
            format!("{field_path}.start must be ISO 8601 date or date-time."),
        )
    })?;
    let end_parts = parse_datetime(end).ok_or_else(|| {
        ErrorResponse::new(
            "InvalidInput",
            format!("{field_path}.end must be ISO 8601 date or date-time."),
        )
    })?;
    if end_parts.to_minutes() <= start_parts.to_minutes() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            format!("{field_path}.end must be after {field_path}.start."),
        ));
    }
    Ok((start_parts, end_parts))
}

fn validate_event_draft_value(
    draft: &Value,
) -> (bool, Value, Vec<Value>, Vec<WarningMessage>, Vec<Value>) {
    let mut normalized = draft.clone();
    let mut field_errors = Vec::new();
    let mut warnings = Vec::new();
    let mut required_actions = Vec::new();

    if normalized
        .get("calendar_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        push_field_error(
            &mut field_errors,
            "draft.calendar_id",
            "required",
            "Choose a calendar before saving.",
        );
    }
    if normalized
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        normalized["kind"] = json!("appointment");
    }
    if normalized
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .count()
        > MAX_TITLE_CHARS
    {
        push_field_error(
            &mut field_errors,
            "draft.title",
            "too_long",
            "Event title must be 512 characters or fewer.",
        );
    }
    if normalized
        .get("location")
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .count()
        > MAX_LOCATION_CHARS
    {
        push_field_error(
            &mut field_errors,
            "draft.location",
            "too_long",
            "Location must be 512 characters or fewer.",
        );
    }
    let body_size = normalized
        .get("body_text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .len()
        + normalized
            .get("body_html")
            .and_then(Value::as_str)
            .unwrap_or("")
            .len();
    if body_size > MAX_BODY_CHARS {
        push_field_error(
            &mut field_errors,
            "draft.body",
            "too_large",
            "Event notes must be 512000 bytes or fewer.",
        );
    }
    if normalized
        .get("time_zone")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        push_field_error(
            &mut field_errors,
            "draft.time_zone",
            "required",
            "Timed events require an IANA time zone id.",
        );
    }
    if normalized
        .get("availability")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        normalized["availability"] = json!("busy");
    }
    if normalized
        .get("privacy")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        normalized["privacy"] = json!("normal");
    }
    match validate_time_range(&normalized, "draft") {
        Ok(_) => {}
        Err(error) => push_field_error(
            &mut field_errors,
            "draft.start_end",
            &error.code,
            &error.message,
        ),
    }
    let attendees = normalized
        .get("attendees")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if attendees.len() > MAX_ATTENDEES {
        push_field_error(
            &mut field_errors,
            "draft.attendees",
            "too_many",
            "Events can include at most 500 attendees.",
        );
    }
    let is_meeting = normalized.get("kind").and_then(Value::as_str) == Some("meeting");
    if is_meeting && !attendees.iter().any(valid_attendee) {
        required_actions.push(json!({
            "action": "add_attendee",
            "message": "Meeting drafts require at least one valid attendee before invite send."
        }));
    }
    if normalized
        .get("categories")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
        > MAX_CATEGORIES
    {
        push_field_error(
            &mut field_errors,
            "draft.categories",
            "too_many",
            "Events can include at most 25 categories.",
        );
    }
    if normalized
        .get("reminders")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
        > MAX_REMINDERS
    {
        push_field_error(
            &mut field_errors,
            "draft.reminders",
            "too_many",
            "Events can include at most 10 reminders.",
        );
    }
    if let Some(reminders) = normalized.get("reminders").and_then(Value::as_array) {
        for (index, reminder) in reminders.iter().enumerate() {
            let offset = reminder
                .get("offset_minutes")
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            if !(0..=40_320).contains(&offset) {
                push_field_error(
                    &mut field_errors,
                    &format!("draft.reminders[{index}].offset_minutes"),
                    "out_of_range",
                    "Reminder offset must be between 0 and 40320 minutes.",
                );
            }
        }
    }
    if let Some(recurrence) = normalized.get("recurrence") {
        let event_time = json!({
            "start": normalized.get("start").cloned().unwrap_or(Value::Null),
            "end": normalized.get("end").cloned().unwrap_or(Value::Null),
            "time_zone": normalized.get("time_zone").cloned().unwrap_or(json!(DEFAULT_TIME_ZONE)),
            "all_day": normalized.get("all_day").cloned().unwrap_or(json!(false))
        });
        let (valid, _rule, errors, rec_warnings) =
            validate_recurrence_rule(&event_time, recurrence, None);
        if !valid {
            field_errors.extend(errors);
        }
        warnings.extend(rec_warnings);
    }

    let valid = field_errors.is_empty() && required_actions.is_empty();
    (valid, normalized, field_errors, warnings, required_actions)
}

fn push_field_error(field_errors: &mut Vec<Value>, field_path: &str, code: &str, message: &str) {
    field_errors.push(json!({
        "field_path": field_path,
        "code": code,
        "message": message
    }));
}

fn valid_attendee(attendee: &Value) -> bool {
    attendee
        .get("email")
        .and_then(Value::as_str)
        .map_or(false, |email| email.contains('@'))
        || attendee.get("resolved_ref").is_some()
}

fn normalize_attendee(attendee: &Value) -> Value {
    let mut normalized = attendee.clone();
    if let Some(email) = normalized.get("email").and_then(Value::as_str) {
        normalized["email"] = json!(email.trim().to_ascii_lowercase());
    }
    if normalized.get("kind").and_then(Value::as_str).is_none() {
        normalized["kind"] = json!("required");
    }
    if normalized
        .get("response_status")
        .and_then(Value::as_str)
        .is_none()
    {
        normalized["response_status"] = json!("none");
    }
    normalized
}

fn event_from_draft(draft: &Value, context: &RequestContext, operation: &'static str) -> Value {
    let mut event = draft.clone();
    if event
        .get("event_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        event["event_id"] = json!(format!("evt_{}", hash_text(&stable_json(draft))));
    }
    if event.get("revision").and_then(Value::as_str).is_none() {
        event["revision"] = json!("planned");
    }
    if event.get("status").and_then(Value::as_str).is_none() {
        event["status"] = json!("confirmed");
    }
    if event.get("created_at").and_then(Value::as_str).is_none() {
        event["created_at"] = json!("1970-01-01T00:00:00Z");
    }
    if event.get("updated_at").and_then(Value::as_str).is_none() {
        event["updated_at"] = json!("1970-01-01T00:00:00Z");
    }
    if event.get("created_by_agent_id").is_none() {
        event["created_by_agent_id"] = json!(context.agent_id);
    }
    if event.get("last_modified_by_agent_id").is_none() {
        event["last_modified_by_agent_id"] = json!(context.agent_id);
    }
    event["planning_state"] = json!({ "operation": operation, "requires_host_execution": true });
    event
}

fn project_event(event: &Value, projection: &str) -> Value {
    match projection {
        "availability_only" => json!({
            "event_id": event.get("event_id").cloned().unwrap_or(Value::Null),
            "calendar_id": event.get("calendar_id").cloned().unwrap_or(Value::Null),
            "start": event.get("start").cloned().unwrap_or(Value::Null),
            "end": event.get("end").cloned().unwrap_or(Value::Null),
            "all_day": event.get("all_day").cloned().unwrap_or(json!(false)),
            "time_zone": event.get("time_zone").cloned().unwrap_or(json!(DEFAULT_TIME_ZONE)),
            "availability": event.get("availability").cloned().unwrap_or(json!("busy")),
            "privacy": event.get("privacy").cloned().unwrap_or(json!("private"))
        }),
        "limited" => json!({
            "event_id": event.get("event_id").cloned().unwrap_or(Value::Null),
            "calendar_id": event.get("calendar_id").cloned().unwrap_or(Value::Null),
            "kind": event.get("kind").cloned().unwrap_or(json!("appointment")),
            "status": event.get("status").cloned().unwrap_or(json!("confirmed")),
            "title": "Busy",
            "start": event.get("start").cloned().unwrap_or(Value::Null),
            "end": event.get("end").cloned().unwrap_or(Value::Null),
            "all_day": event.get("all_day").cloned().unwrap_or(json!(false)),
            "time_zone": event.get("time_zone").cloned().unwrap_or(json!(DEFAULT_TIME_ZONE)),
            "availability": event.get("availability").cloned().unwrap_or(json!("busy")),
            "privacy": event.get("privacy").cloned().unwrap_or(json!("private")),
            "recurrence": event.get("recurrence").is_some()
        }),
        _ => event.clone(),
    }
}

fn read_snapshot_array(request: &Value, field: &str) -> Vec<Value> {
    request
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn read_snapshot_object(request: &Value, field: &str) -> Value {
    request.get(field).cloned().unwrap_or_else(|| json!({}))
}

fn base_preferences(context: &RequestContext) -> Value {
    json!({
        "default_view": "work_week",
        "density": "comfortable",
        "default_calendar_id": "primary",
        "time_zone": context.time_zone(),
        "work_days": ["MO", "TU", "WE", "TH", "FR"],
        "work_hours": { "start": "09:00", "end": "17:00", "time_zone": context.time_zone() },
        "default_reminders": [{ "method": "host_notification", "offset_minutes": 15, "state": "planned" }],
        "default_online_meeting": false,
        "overlay_mode": "overlay",
        "category_sort": "name"
    })
}

fn default_calendar(context: &RequestContext) -> Value {
    json!({
        "calendar_id": "primary",
        "owner_id": context.user_id,
        "display_name": "Calendar",
        "kind": "primary",
        "role": "owner",
        "color": "#2563eb",
        "visible": true,
        "display_order": 0,
        "default_reminders": [{ "method": "host_notification", "offset_minutes": 15, "state": "planned" }],
        "time_zone": context.time_zone(),
        "capabilities": {
            "can_create": true,
            "can_update": true,
            "can_delete": false,
            "can_share": true,
            "can_delegate": true,
            "can_copy_from": true,
            "can_move_from": true,
            "can_move_to": true,
            "can_subscribe": true,
            "can_writeback_subscription": false,
            "can_view_private_full": true
        },
        "sync_state": { "state": "unknown" }
    })
}

fn handle_get_calendar_capabilities(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let available_effects: Vec<Value> = ALL_EFFECTS
        .iter()
        .map(|effect| {
            json!({
                "effect": effect,
                "available": context.has_effect(effect)
            })
        })
        .collect();
    Ok(ok(
        "get_calendar_capabilities",
        json!({
            "capabilities": {
                "read": true,
                "write": context.permission()?.satisfies(Permission::Write),
                "invite": context.permission()?.satisfies(Permission::Invite),
                "respond": context.permission()?.satisfies(Permission::Respond),
                "share": context.permission()?.satisfies(Permission::Share),
                "delegate": context.permission()?.satisfies(Permission::Delegate),
                "settings": context.permission()?.satisfies(Permission::Settings),
                "audit": context.permission()?.satisfies(Permission::Audit),
                "declared": context.capabilities
            },
            "locale": context.locale,
            "host_capabilities": context.host_capabilities,
            "policy": context.policy.unwrap_or_else(|| json!({ "recurrence_preview_limit": MAX_RECURRENCE_PREVIEW, "no_end_recurrence": false })),
            "host_effects": available_effects,
            "default_view_state": {
                "selected_calendar_ids": request.get("calendar_ids").cloned().unwrap_or_else(|| json!(["primary"])),
                "hidden_calendar_ids": [],
                "display_order": ["primary"],
                "mode": "overlay",
                "view": "work_week"
            },
            "warnings": []
        }),
        Vec::new(),
    ))
}

fn handle_list_calendars(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let include_hidden = get_bool(request, "include_hidden", false);
    let mut calendars = read_snapshot_array(request, "snapshot");
    if calendars.is_empty() {
        calendars.push(default_calendar(&context));
    }
    if !include_hidden {
        calendars.retain(|calendar| {
            calendar
                .get("visible")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        });
    }
    Ok(ok(
        "list_calendars",
        json!({
            "calendars": calendars,
            "visibility_state": { "include_hidden": include_hidden },
            "sync_state": { "state": if get_bool(request, "refresh", false) { "syncing" } else { "current" } },
            "needs_host_refresh": get_bool(request, "refresh", false)
        }),
        Vec::new(),
    ))
}

fn handle_save_calendar(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let calendar = require_field(request, "calendar")?.clone();
    if calendar
        .get("display_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "calendar.display_name is required.",
        ));
    }
    let created = get_bool(request, "create", false);
    let effect = build_effect(
        &context,
        "save_calendar",
        EFFECT_CALENDAR_STORE_WRITE,
        "Create or update calendar metadata",
        json!({
            "calendar": calendar,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null),
            "create": created
        }),
        "high",
    );
    Ok(ok(
        "save_calendar",
        json!({ "calendar": calendar, "created": created, "updated": !created }),
        vec![effect],
    ))
}

fn handle_delete_calendar(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    let calendar_id = require_non_empty_str(request, "calendar_id")?;
    let mode = require_enum_str(request, "mode", DELETE_CALENDAR_MODES)?;
    if mode == "delete" {
        require_confirmation(
            request,
            "Deleting a calendar requires explicit confirmation.",
        )?;
        require_effect(&context, EFFECT_CALENDAR_STORE_DELETE)?;
    } else {
        require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    }
    let effect_name = if mode == "delete" {
        EFFECT_CALENDAR_STORE_DELETE
    } else {
        EFFECT_CALENDAR_STORE_WRITE
    };
    let effect = build_effect(
        &context,
        "delete_calendar",
        effect_name,
        "Delete, hide, or unsubscribe a calendar through host services",
        json!({
            "calendar_id": calendar_id,
            "mode": mode,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null)
        }),
        "critical",
    );
    Ok(ok(
        "delete_calendar",
        json!({
            "calendar_id": calendar_id,
            "deletion_mode": mode,
            "event_outcome": if mode == "delete" { "host_deletes_or_rehomes_events" } else { "events_preserved" },
            "requires_confirmation": mode == "delete"
        }),
        vec![effect],
    ))
}

fn handle_set_calendar_view_state(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Settings)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let view_state = require_field(request, "view_state")?.clone();
    let effect = build_effect(
        &context,
        "set_calendar_view_state",
        EFFECT_CALENDAR_STORE_WRITE,
        "Persist calendar view preferences",
        json!({ "view_state": view_state }),
        "medium",
    );
    Ok(ok(
        "set_calendar_view_state",
        json!({ "view_state": view_state, "normalized": true }),
        vec![effect],
    ))
}

fn handle_list_events(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let range = require_field(request, "range")?;
    validate_time_range(range, "range")?;
    let calendar_ids = require_array(request, "calendar_ids")?;
    if calendar_ids.is_empty() || calendar_ids.len() > 100 {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "calendar_ids must include 1 to 100 calendars.",
        ));
    }
    let projection = request
        .get("projection")
        .and_then(Value::as_str)
        .unwrap_or("full");
    let events: Vec<Value> = read_snapshot_array(request, "snapshot")
        .into_iter()
        .map(|event| project_event(&event, projection))
        .collect();
    Ok(ok(
        "list_events",
        json!({
            "events": events,
            "range": range,
            "projection_applied": projection,
            "sync_state": { "state": if get_bool(request, "refresh", false) { "syncing" } else { "current" } },
            "warnings": []
        }),
        Vec::new(),
    ))
}

fn handle_get_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let event_id = require_non_empty_str(request, "event_id")?;
    let projection = request
        .get("projection")
        .and_then(Value::as_str)
        .unwrap_or("full");
    let event = read_snapshot_array(request, "snapshot")
        .into_iter()
        .find(|event| event.get("event_id").and_then(Value::as_str) == Some(event_id))
        .unwrap_or_else(|| {
            json!({
                "event_id": event_id,
                "calendar_id": request.get("calendar_id").cloned().unwrap_or(json!("primary")),
                "kind": "appointment",
                "status": "confirmed",
                "title": "Host event placeholder",
                "start": "1970-01-01T09:00:00Z",
                "end": "1970-01-01T09:30:00Z",
                "all_day": false,
                "time_zone": context.time_zone(),
                "availability": "busy",
                "privacy": "normal",
                "categories": [],
                "reminders": [],
                "attendees": [],
                "revision": "unknown",
                "created_at": "1970-01-01T00:00:00Z",
                "updated_at": "1970-01-01T00:00:00Z"
            })
        });
    Ok(ok(
        "get_event",
        json!({
            "event": project_event(&event, projection),
            "projection_applied": projection,
            "allowed_actions": ["view", "edit", "copy", "move", "delete"],
            "warnings": []
        }),
        Vec::new(),
    ))
}

fn handle_search_events(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let query = require_non_empty_str(request, "query")?;
    if query.chars().count() > MAX_SEARCH_QUERY_CHARS {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "Search query must be 256 characters or fewer.",
        ));
    }
    let lower_query = query.to_ascii_lowercase();
    let projection = request
        .get("projection")
        .and_then(Value::as_str)
        .unwrap_or("full");
    let results: Vec<Value> = read_snapshot_array(request, "snapshot")
        .into_iter()
        .filter(|event| {
            stable_json(event)
                .to_ascii_lowercase()
                .contains(&lower_query)
        })
        .map(|event| project_event(&event, projection))
        .collect();
    Ok(ok(
        "search_events",
        json!({
            "results": results,
            "facets": {},
            "pagination": request.get("pagination").cloned().unwrap_or_else(|| json!({ "limit": 50 })),
            "projection_applied": projection
        }),
        Vec::new(),
    ))
}

fn handle_validate_event_draft(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    let draft = require_field(request, "draft")?;
    let (valid, normalized_draft, field_errors, warnings, required_actions) =
        validate_event_draft_value(draft);
    Ok(ok_with_warnings(
        "validate_event_draft",
        json!({
            "valid": valid,
            "normalized_draft": normalized_draft,
            "field_errors": field_errors,
            "warnings": warnings,
            "required_actions": required_actions
        }),
        Vec::new(),
        warnings,
    ))
}

fn handle_create_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let draft = require_field(request, "draft")?;
    let (valid, normalized_draft, field_errors, warnings, required_actions) =
        validate_event_draft_value(draft);
    if !valid {
        return Err(
            ErrorResponse::new("InvalidInput", "Event draft is not valid for creation.")
                .with_details(json!({
                    "field_errors": field_errors,
                    "required_actions": required_actions
                })),
        );
    }
    let mut effects = vec![build_effect(
        &context,
        "create_event",
        EFFECT_CALENDAR_STORE_WRITE,
        "Create calendar event in host store",
        json!({
            "draft": normalized_draft,
            "client_operation_id": request.get("client_operation_id").cloned().unwrap_or(Value::Null)
        }),
        "high",
    )];
    let send_invites = get_bool(request, "send_invites", false);
    let is_meeting = normalized_draft.get("kind").and_then(Value::as_str) == Some("meeting");
    if send_invites || is_meeting {
        require_permission(&context, Permission::Invite)?;
        require_effect(&context, EFFECT_CALENDAR_INVITE_SEND)?;
        effects.push(build_effect(
            &context,
            "create_event",
            EFFECT_CALENDAR_INVITE_SEND,
            "Send meeting invitations through host",
            json!({ "draft": normalized_draft }),
            "critical",
        ));
    }
    if normalized_draft
        .pointer("/online_meeting/requested")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        require_effect(&context, EFFECT_CONFERENCE_LINK_CREATE)?;
        effects.push(build_effect(
            &context,
            "create_event",
            EFFECT_CONFERENCE_LINK_CREATE,
            "Create host-owned online meeting link",
            json!({ "event_ref": "planned" }),
            "high",
        ));
    }
    if normalized_draft
        .get("reminders")
        .and_then(Value::as_array)
        .map_or(false, |items| !items.is_empty())
    {
        require_effect(&context, EFFECT_REMINDER_SCHEDULE)?;
        effects.push(build_effect(
            &context,
            "create_event",
            EFFECT_REMINDER_SCHEDULE,
            "Schedule host reminders",
            json!({ "reminders": normalized_draft.get("reminders").cloned().unwrap_or(json!([])) }),
            "medium",
        ));
    }
    if let Some(audit) = optional_effect(
        &context,
        "create_event",
        EFFECT_AUDIT_WRITE,
        "Write host audit projection for calendar creation",
        json!({ "event_preview": event_from_draft(&normalized_draft, &context, "create_event") }),
        "high",
    ) {
        effects.push(audit);
    }
    Ok(ok_with_warnings(
        "create_event",
        json!({
            "event_preview": event_from_draft(&normalized_draft, &context, "create_event"),
            "persistence_status": "planned",
            "notification_state": if send_invites || is_meeting { "planned" } else { "none" },
            "requires_host_execution": true
        }),
        effects,
        warnings,
    ))
}

fn handle_update_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let event_id = require_non_empty_str(request, "event_id")?;
    let patch = require_field(request, "patch")?.clone();
    let scope = optional_enum_str(request, "scope", "single", EVENT_SCOPES)?;
    let notification_scope = optional_enum_str(
        request,
        "notification_scope",
        "host_default",
        NOTIFICATION_SCOPES,
    )?;
    let mut effects = vec![build_effect(
        &context,
        "update_event",
        EFFECT_CALENDAR_STORE_WRITE,
        "Update calendar event in host store",
        json!({
            "event_id": event_id,
            "occurrence_id": request.get("occurrence_id").cloned().unwrap_or(Value::Null),
            "scope": scope,
            "patch": patch,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null)
        }),
        "high",
    )];
    if notification_scope != "none"
        && (patch.get("attendees").is_some() || patch.get("title").is_some())
    {
        require_permission(&context, Permission::Invite)?;
        require_effect(&context, EFFECT_CALENDAR_INVITE_SEND)?;
        effects.push(build_effect(
            &context,
            "update_event",
            EFFECT_CALENDAR_INVITE_SEND,
            "Send calendar update notifications through host",
            json!({ "event_id": event_id, "notification_scope": notification_scope }),
            "critical",
        ));
    }
    Ok(ok(
        "update_event",
        json!({
            "event_preview": { "event_id": event_id, "patch": patch, "planning_state": "updated_by_host" },
            "attendee_delta": { "requires_host_diff": true },
            "notification_state": { "scope": notification_scope, "state": if effects.len() > 1 { "planned" } else { "none" } },
            "conflict_state": { "requires_host_revision_check": request.get("revision").is_some() },
            "requires_host_execution": true
        }),
        effects,
    ))
}

fn handle_delete_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_DELETE)?;
    require_confirmation(
        request,
        "Deleting or cancelling an event requires explicit confirmation.",
    )?;
    let target = require_field(request, "target")?.clone();
    let mode = require_enum_str(request, "mode", EVENT_DELETE_MODES)?;
    let scope = require_enum_str(request, "scope", EVENT_SCOPES)?;
    let mut effects = vec![build_effect(
        &context,
        "delete_event",
        EFFECT_CALENDAR_STORE_DELETE,
        "Delete or cancel calendar event through host store",
        json!({
            "target": target,
            "scope": scope,
            "mode": mode,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null)
        }),
        "critical",
    )];
    if mode == "cancel" {
        require_permission(&context, Permission::Invite)?;
        require_effect(&context, EFFECT_CALENDAR_INVITE_SEND)?;
        effects.push(build_effect(&context, "delete_event", EFFECT_CALENDAR_INVITE_SEND, "Send meeting cancellation through host", json!({ "target": target, "message": request.get("message").cloned().unwrap_or(Value::Null) }), "critical"));
    }
    Ok(ok(
        "delete_event",
        json!({
            "target": target,
            "scope": scope,
            "deletion_state": { "state": "planned", "mode": mode },
            "notification_state": { "state": if mode == "cancel" { "planned" } else { "none" } },
            "requires_host_execution": true
        }),
        effects,
    ))
}

fn handle_copy_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_STORE_READ, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    let event_id = require_non_empty_str(request, "event_id")?;
    let destination_calendar_id = require_non_empty_str(request, "destination_calendar_id")?;
    let read_effect = build_effect(
        &context,
        "copy_event",
        EFFECT_CALENDAR_STORE_READ,
        "Read source event for host-owned copy",
        json!({ "event_id": event_id }),
        "high",
    );
    let write_effect = build_effect(
        &context,
        "copy_event",
        EFFECT_CALENDAR_STORE_WRITE,
        "Write copied event to destination calendar",
        json!({
            "event_id": event_id,
            "destination_calendar_id": destination_calendar_id,
            "copy_options": request.get("copy_options").cloned().unwrap_or_else(|| json!({}))
        }),
        "high",
    );
    Ok(ok(
        "copy_event",
        json!({
            "source_event_id": event_id,
            "new_event_preview": { "event_id": format!("copy_{}", hash_text(event_id)), "calendar_id": destination_calendar_id, "source_event_id": event_id },
            "requires_host_execution": true
        }),
        vec![read_effect, write_effect],
    ))
}

fn handle_move_event(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let event_id = require_non_empty_str(request, "event_id")?;
    let destination_calendar_id = require_non_empty_str(request, "destination_calendar_id")?;
    let notification_scope = optional_enum_str(
        request,
        "notification_scope",
        "host_default",
        NOTIFICATION_SCOPES,
    )?;
    let effect = build_effect(
        &context,
        "move_event",
        EFFECT_CALENDAR_STORE_WRITE,
        "Move event to another host-managed calendar",
        json!({
            "event_id": event_id,
            "destination_calendar_id": destination_calendar_id,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null),
            "notification_scope": notification_scope
        }),
        "high",
    );
    Ok(ok(
        "move_event",
        json!({
            "event_preview": { "event_id": event_id, "calendar_id": destination_calendar_id, "planning_state": "move_planned" },
            "source_calendar_id": request.get("source_calendar_id").cloned().unwrap_or(Value::Null),
            "destination_calendar_id": destination_calendar_id,
            "notification_state": { "state": "host_default" }
        }),
        vec![effect],
    ))
}

fn handle_validate_recurrence(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    let event_time = require_field(request, "event_time")?;
    let recurrence = require_field(request, "recurrence")?;
    let (valid, normalized_recurrence, field_errors, warnings) =
        validate_recurrence_rule(event_time, recurrence, context.policy.as_ref());
    Ok(ok_with_warnings(
        "validate_recurrence",
        json!({
            "valid": valid,
            "normalized_recurrence": normalized_recurrence,
            "field_errors": field_errors,
            "warnings": warnings
        }),
        Vec::new(),
        warnings,
    ))
}

fn handle_preview_recurrence(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    let event_time = require_field(request, "event_time")?;
    let recurrence = require_field(request, "recurrence")?;
    let limit = request
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(10)
        .min(MAX_RECURRENCE_PREVIEW as u64) as usize;
    let (valid, _normalized, field_errors, mut warnings) =
        validate_recurrence_rule(event_time, recurrence, context.policy.as_ref());
    if !valid {
        return Err(ErrorResponse::new(
            "RecurrenceUnsupported",
            "Recurrence cannot be previewed until validation errors are fixed.",
        )
        .with_details(json!({ "field_errors": field_errors })));
    }
    let (occurrences, truncated) = expand_recurrence(event_time, recurrence, limit)?;
    if truncated {
        warnings.push(WarningMessage::new(
            "truncated",
            "Preview was capped by the recurrence preview limit.",
        ));
    }
    Ok(ok_with_warnings(
        "preview_recurrence",
        json!({
            "occurrences": occurrences,
            "truncated": truncated,
            "warnings": warnings
        }),
        Vec::new(),
        warnings,
    ))
}

fn handle_plan_free_busy_lookup(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_FREE_BUSY_LOOKUP)?;
    let attendees = require_array(request, "attendees")?;
    if attendees.is_empty() || attendees.len() > MAX_ATTENDEES {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "attendees must include 1 to 500 entries.",
        ));
    }
    let range = require_field(request, "range")?;
    validate_time_range(range, "range")?;
    let lookup_request = json!({
        "attendees": attendees.iter().map(normalize_attendee).collect::<Vec<_>>(),
        "range": range,
        "granularity_minutes": request.get("granularity_minutes").cloned().unwrap_or(json!(30))
    });
    let effect = build_effect(
        &context,
        "plan_free_busy_lookup",
        EFFECT_FREE_BUSY_LOOKUP,
        "Lookup free/busy availability through host services",
        lookup_request.clone(),
        "high",
    );
    Ok(ok(
        "plan_free_busy_lookup",
        json!({ "lookup_request": lookup_request, "requires_host_execution": true }),
        vec![effect],
    ))
}

fn handle_suggest_meeting_times(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    let draft = require_field(request, "meeting_draft")?;
    let snapshots = require_array(request, "free_busy_snapshot")?;
    if snapshots.is_empty() {
        return Err(ErrorResponse::new(
            "FreeBusyUnavailable",
            "Provide at least one free/busy snapshot before scoring meeting times.",
        ));
    }
    let suggestions = score_meeting_times(draft, snapshots, request.get("rooms_snapshot"));
    Ok(ok(
        "suggest_meeting_times",
        json!({
            "suggestions": suggestions,
            "conflicts": [],
            "warnings": []
        }),
        Vec::new(),
    ))
}

fn handle_search_rooms(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_ROOM_RESOURCE_LOOKUP)?;
    let range = require_field(request, "range")?;
    validate_time_range(range, "range")?;
    let room_lookup_request = json!({
        "range": range,
        "query": request.get("query").cloned().unwrap_or(Value::Null),
        "capacity": request.get("capacity").cloned().unwrap_or(Value::Null),
        "features": request.get("features").cloned().unwrap_or_else(|| json!([])),
        "location_hint": request.get("location_hint").cloned().unwrap_or(Value::Null)
    });
    let effect = build_effect(
        &context,
        "search_rooms",
        EFFECT_ROOM_RESOURCE_LOOKUP,
        "Search host-managed rooms and resources",
        room_lookup_request.clone(),
        "high",
    );
    Ok(ok(
        "search_rooms",
        json!({ "room_lookup_request": room_lookup_request, "requires_host_execution": true }),
        vec![effect],
    ))
}

fn handle_resolve_attendees(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    let attendees = require_array(request, "attendees")?;
    if attendees.len() > MAX_ATTENDEES {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "At most 500 attendees can be resolved.",
        ));
    }
    let normalized: Vec<Value> = attendees.iter().map(normalize_attendee).collect();
    let unresolved: Vec<Value> = normalized
        .iter()
        .filter(|attendee| !valid_attendee(attendee))
        .cloned()
        .collect();
    let allow_manual_email = get_bool(request, "allow_manual_email", false);
    if !unresolved.is_empty() && !allow_manual_email && !context.has_effect(EFFECT_CONTACT_LOOKUP) {
        return Err(ErrorResponse::new(
            "AttendeeResolutionFailed",
            "Unresolved attendees require ContactLookup or manual email fallback.",
        ));
    }
    let effects = if !unresolved.is_empty() && context.has_effect(EFFECT_CONTACT_LOOKUP) {
        vec![build_effect(
            &context,
            "resolve_attendees",
            EFFECT_CONTACT_LOOKUP,
            "Resolve attendees through host contacts",
            json!({ "attendees": unresolved }),
            "high",
        )]
    } else {
        Vec::new()
    };
    Ok(ok(
        "resolve_attendees",
        json!({
            "normalized_attendees": normalized,
            "unresolved": unresolved,
            "requires_host_execution": !effects.is_empty()
        }),
        effects,
    ))
}

fn handle_list_invitations(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    read_list_operation(
        request,
        "list_invitations",
        "invitations",
        EFFECT_CALENDAR_STORE_READ,
        Permission::Read,
    )
}

fn handle_get_invitation(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    let invitation_id = require_non_empty_str(request, "invitation_id")?;
    Ok(ok(
        "get_invitation",
        json!({
            "invitation": read_snapshot_object(request, "snapshot"),
            "invitation_id": invitation_id,
            "allowed_responses": ["accept", "tentative", "decline", "propose_new_time"],
            "conflicts": [],
            "warnings": []
        }),
        Vec::new(),
    ))
}

fn handle_respond_to_invitation(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Respond)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_INVITE_RESPOND, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    let invitation_id = require_non_empty_str(request, "invitation_id")?;
    let response = require_enum_str(request, "response", INVITATION_RESPONSES)?;
    let scope = optional_enum_str(request, "scope", "series", INVITATION_SCOPES)?;
    let effects = vec![
        build_effect(
            &context,
            "respond_to_invitation",
            EFFECT_CALENDAR_INVITE_RESPOND,
            "Send RSVP response through host",
            json!({ "invitation_id": invitation_id, "response": response, "scope": scope, "send_response": get_bool(request, "send_response", true) }),
            "high",
        ),
        build_effect(
            &context,
            "respond_to_invitation",
            EFFECT_CALENDAR_STORE_WRITE,
            "Update local calendar response projection",
            json!({ "invitation_id": invitation_id, "response": response, "scope": scope }),
            "high",
        ),
    ];
    Ok(ok(
        "respond_to_invitation",
        json!({
            "invitation_id": invitation_id,
            "response_state": { "state": "planned", "response": response },
            "calendar_update_state": { "state": "planned" },
            "requires_host_execution": true
        }),
        effects,
    ))
}

fn handle_propose_new_time(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Respond)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_INVITE_RESPOND, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    let invitation_id = require_non_empty_str(request, "invitation_id")?;
    let scope = optional_enum_str(request, "scope", "series", INVITATION_SCOPES)?;
    let proposed_time = require_field(request, "proposed_time")?;
    validate_time_range(proposed_time, "proposed_time")?;
    let effects = vec![
        build_effect(
            &context,
            "propose_new_time",
            EFFECT_CALENDAR_INVITE_RESPOND,
            "Send proposed meeting time through host",
            json!({ "invitation_id": invitation_id, "proposed_time": proposed_time, "scope": scope }),
            "high",
        ),
        build_effect(
            &context,
            "propose_new_time",
            EFFECT_CALENDAR_STORE_WRITE,
            "Save proposed time response projection",
            json!({ "invitation_id": invitation_id, "proposed_time": proposed_time, "scope": scope }),
            "high",
        ),
    ];
    Ok(ok(
        "propose_new_time",
        json!({
            "proposal_state": { "state": "planned", "invitation_id": invitation_id },
            "proposed_time": proposed_time,
            "requires_host_execution": true
        }),
        effects,
    ))
}

fn handle_list_categories(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    Ok(ok(
        "list_categories",
        json!({ "categories": read_snapshot_array(request, "snapshot") }),
        Vec::new(),
    ))
}

fn handle_save_category(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Settings)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let category = require_field(request, "category")?.clone();
    if category
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "category.name is required.",
        ));
    }
    let created = category
        .get("category_id")
        .and_then(Value::as_str)
        .is_none();
    let effect = build_effect(
        &context,
        "save_category",
        EFFECT_CALENDAR_STORE_WRITE,
        "Save host-managed calendar category",
        json!({ "category": category, "revision": request.get("revision").cloned().unwrap_or(Value::Null) }),
        "medium",
    );
    Ok(ok(
        "save_category",
        json!({ "category": category, "created": created, "updated": !created }),
        vec![effect],
    ))
}

fn handle_delete_category(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Settings)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    require_confirmation(
        request,
        "Deleting a category requires explicit confirmation.",
    )?;
    let category_id = require_non_empty_str(request, "category_id")?;
    let effect = build_effect(
        &context,
        "delete_category",
        EFFECT_CALENDAR_STORE_WRITE,
        "Remove or reassign calendar category references",
        json!({
            "category_id": category_id,
            "replacement_category_id": request.get("replacement_category_id").cloned().unwrap_or(Value::Null)
        }),
        "medium",
    );
    Ok(ok(
        "delete_category",
        json!({
            "category_id": category_id,
            "event_update_plan": { "replacement_category_id": request.get("replacement_category_id").cloned().unwrap_or(Value::Null) },
            "requires_host_execution": true
        }),
        vec![effect],
    ))
}

fn handle_bulk_update_events(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    require_field(request, "selection")?;
    require_field(request, "operation")?;
    require_confirmation(request, "Bulk event updates require explicit confirmation.")?;
    let estimated = request
        .pointer("/selection/estimated_count")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            request
                .pointer("/selection/event_ids")
                .and_then(Value::as_array)
                .map_or(0, |ids| ids.len() as u64)
        });
    let effect = build_effect(
        &context,
        "bulk_update_events",
        EFFECT_CALENDAR_STORE_WRITE,
        "Apply confirmed bulk calendar update through host",
        json!({
            "selection": request.get("selection").cloned().unwrap_or(Value::Null),
            "operation": request.get("operation").cloned().unwrap_or(Value::Null)
        }),
        "critical",
    );
    Ok(ok(
        "bulk_update_events",
        json!({ "affected_count": estimated, "operation_state": { "state": "planned" }, "requires_host_execution": true }),
        vec![effect],
    ))
}

fn handle_get_sharing_state(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Share)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_SHARE_MANAGE, EFFECT_CALENDAR_STORE_READ],
    )?;
    require_non_empty_str(request, "calendar_id")?;
    Ok(ok(
        "get_sharing_state",
        json!({
            "shares": request.get("snapshot").and_then(|s| s.get("shares")).cloned().unwrap_or_else(|| json!([])),
            "delegates": request.get("snapshot").and_then(|s| s.get("delegates")).cloned().unwrap_or_else(|| json!([])),
            "allowed_actions": ["grant", "revoke", "change_access", "delegate"]
        }),
        Vec::new(),
    ))
}

fn handle_update_sharing(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Share)?;
    require_effects(
        &context,
        &[
            EFFECT_CALENDAR_SHARE_MANAGE,
            EFFECT_CONTACT_LOOKUP,
            EFFECT_AUDIT_WRITE,
        ],
    )?;
    require_confirmation(request, "Sharing changes require explicit confirmation.")?;
    let calendar_id = require_non_empty_str(request, "calendar_id")?;
    let changes = require_field(request, "changes")?.clone();
    let effects = vec![
        build_effect(
            &context,
            "update_sharing",
            EFFECT_CONTACT_LOOKUP,
            "Resolve sharing principals through host contacts",
            json!({ "changes": changes }),
            "high",
        ),
        build_effect(
            &context,
            "update_sharing",
            EFFECT_CALENDAR_SHARE_MANAGE,
            "Apply host-owned calendar sharing changes",
            json!({ "calendar_id": calendar_id, "changes": changes }),
            "high",
        ),
        build_effect(
            &context,
            "update_sharing",
            EFFECT_AUDIT_WRITE,
            "Record sharing change audit projection",
            json!({ "calendar_id": calendar_id }),
            "high",
        ),
    ];
    Ok(ok(
        "update_sharing",
        json!({
            "sharing_state": { "calendar_id": calendar_id, "state": "planned" },
            "provisioning_state": { "state": "planned" },
            "requires_host_execution": true
        }),
        effects,
    ))
}

fn handle_accept_shared_calendar(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Share)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_SHARE_MANAGE, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    let share_invitation_id = require_non_empty_str(request, "share_invitation_id")?;
    let calendar = json!({
        "calendar_id": format!("shared_{}", hash_text(share_invitation_id)),
        "owner_id": "host_shared_owner",
        "display_name": request.pointer("/display_options/display_name").and_then(Value::as_str).unwrap_or("Shared Calendar"),
        "kind": "shared",
        "role": "viewer",
        "color": request.pointer("/display_options/color").cloned().unwrap_or(json!("#059669")),
        "visible": true,
        "capabilities": {},
        "time_zone": context.time_zone(),
        "sync_state": { "state": "unknown" }
    });
    let effects = vec![
        build_effect(
            &context,
            "accept_shared_calendar",
            EFFECT_CALENDAR_SHARE_MANAGE,
            "Accept host-owned shared calendar invitation",
            json!({ "share_invitation_id": share_invitation_id }),
            "high",
        ),
        build_effect(
            &context,
            "accept_shared_calendar",
            EFFECT_CALENDAR_STORE_WRITE,
            "Provision shared calendar projection",
            json!({ "calendar": calendar }),
            "high",
        ),
    ];
    Ok(ok(
        "accept_shared_calendar",
        json!({ "calendar": calendar, "provisioning_state": { "state": "planned" } }),
        effects,
    ))
}

fn handle_subscribe_calendar(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Share)?;
    require_effects(
        &context,
        &[EFFECT_SUBSCRIPTION_SYNC, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    require_confirmation(
        request,
        "Subscribing to an external calendar requires explicit confirmation.",
    )?;
    let subscription = sanitize_subscription_request(request)?;
    let calendar_preview = json!({
        "calendar_id": format!("sub_{}", hash_text(&stable_json(&subscription))),
        "owner_id": context.user_id,
        "display_name": subscription.get("display_name").cloned().unwrap_or(json!("Subscribed Calendar")),
        "kind": "subscription",
        "role": "viewer",
        "color": subscription.get("color").cloned().unwrap_or(json!("#7c3aed")),
        "visible": true,
        "capabilities": { "can_writeback_subscription": false },
        "time_zone": context.time_zone(),
        "sync_state": { "state": "syncing" }
    });
    let effects = vec![
        build_effect(
            &context,
            "subscribe_calendar",
            EFFECT_SUBSCRIPTION_SYNC,
            "Validate and sync Internet Calendar through host",
            json!({ "subscription": subscription }),
            "high",
        ),
        build_effect(
            &context,
            "subscribe_calendar",
            EFFECT_CALENDAR_STORE_WRITE,
            "Store subscription calendar projection",
            json!({ "calendar": calendar_preview }),
            "high",
        ),
    ];
    Ok(ok(
        "subscribe_calendar",
        json!({ "subscription_state": { "state": "planned" }, "calendar_preview": calendar_preview }),
        effects,
    ))
}

fn handle_get_reminder_state(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effects(
        &context,
        &[EFFECT_CALENDAR_STORE_READ, EFFECT_REMINDER_SCHEDULE],
    )?;
    if request.get("event_id").is_none() && request.get("calendar_id").is_none() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "event_id or calendar_id is required.",
        ));
    }
    Ok(ok(
        "get_reminder_state",
        json!({
            "reminders": read_snapshot_array(request, "snapshot"),
            "delivery_state": { "state": "host_managed" }
        }),
        Vec::new(),
    ))
}

fn handle_snooze_or_dismiss_reminder(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Write)?;
    require_effects(
        &context,
        &[EFFECT_REMINDER_SCHEDULE, EFFECT_CALENDAR_STORE_WRITE],
    )?;
    let reminder_id = require_non_empty_str(request, "reminder_id")?;
    let action = require_enum_str(request, "action", REMINDER_ACTIONS)?;
    if action == "snooze" && request.get("snooze_until").is_none() {
        return Err(ErrorResponse::new(
            "InvalidInput",
            "snooze_until is required when action is snooze.",
        ));
    }
    let reminder_state = json!({
        "reminder_id": reminder_id,
        "method": "host_notification",
        "offset_minutes": 0,
        "state": if action == "dismiss" { "dismissed" } else { "snoozed" },
        "snooze_until": request.get("snooze_until").cloned().unwrap_or(Value::Null)
    });
    let effects = vec![
        build_effect(
            &context,
            "snooze_or_dismiss_reminder",
            EFFECT_REMINDER_SCHEDULE,
            "Snooze or dismiss host reminder",
            reminder_state.clone(),
            "medium",
        ),
        build_effect(
            &context,
            "snooze_or_dismiss_reminder",
            EFFECT_CALENDAR_STORE_WRITE,
            "Persist reminder state projection",
            reminder_state.clone(),
            "medium",
        ),
    ];
    Ok(ok(
        "snooze_or_dismiss_reminder",
        json!({ "reminder_state": reminder_state, "requires_host_execution": true }),
        effects,
    ))
}

fn handle_get_user_preferences(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Settings)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_READ)?;
    Ok(ok(
        "get_user_preferences",
        json!({
            "preferences": request.get("snapshot").cloned().unwrap_or_else(|| base_preferences(&context)),
            "defaults": if get_bool(request, "include_defaults", false) { base_preferences(&context) } else { json!({}) },
            "needs_host_refresh": request.get("snapshot").is_none()
        }),
        Vec::new(),
    ))
}

fn handle_save_user_preferences(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Settings)?;
    require_effect(&context, EFFECT_CALENDAR_STORE_WRITE)?;
    let preferences = require_field(request, "preferences")?.clone();
    let effect = build_effect(
        &context,
        "save_user_preferences",
        EFFECT_CALENDAR_STORE_WRITE,
        "Persist calendar preferences",
        json!({
            "preferences": preferences,
            "revision": request.get("revision").cloned().unwrap_or(Value::Null)
        }),
        "medium",
    );
    Ok(ok(
        "save_user_preferences",
        json!({ "preferences": preferences, "updated": true }),
        vec![effect],
    ))
}

fn handle_get_offline_state(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Read)?;
    require_effect(&context, EFFECT_OFFLINE_CACHE_READ)?;
    Ok(ok(
        "get_offline_state",
        json!({
            "offline_state": { "state": "host_managed", "calendar_ids": request.get("calendar_ids").cloned().unwrap_or_else(|| json!([])) },
            "last_sync_at": Value::Null,
            "mutation_policy": "blocked"
        }),
        Vec::new(),
    ))
}

fn handle_get_activity_log(request: &Value) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, Permission::Audit)?;
    require_effect(&context, EFFECT_AUDIT_READ)?;
    Ok(ok(
        "get_activity_log",
        json!({
            "activities": read_snapshot_array(request, "snapshot"),
            "pagination": request.get("pagination").cloned().unwrap_or_else(|| json!({ "limit": 50 }))
        }),
        Vec::new(),
    ))
}

fn read_list_operation(
    request: &Value,
    operation: &'static str,
    data_field: &'static str,
    effect: &'static str,
    permission: Permission,
) -> Result<OperationResponse, ErrorResponse> {
    let context = RequestContext::from_request(request)?;
    require_permission(&context, permission)?;
    require_effect(&context, effect)?;
    let mut data = Map::new();
    data.insert(
        data_field.to_string(),
        json!(read_snapshot_array(request, "snapshot")),
    );
    data.insert(
        "pagination".to_string(),
        request
            .get("pagination")
            .cloned()
            .unwrap_or_else(|| json!({ "limit": 50 })),
    );
    data.insert(
        "sync_state".to_string(),
        json!({ "state": if get_bool(request, "refresh", false) { "syncing" } else { "current" } }),
    );
    Ok(ok(operation, Value::Object(data), Vec::new()))
}

fn validate_recurrence_rule(
    event_time: &Value,
    recurrence: &Value,
    policy: Option<&Value>,
) -> (bool, Value, Vec<Value>, Vec<WarningMessage>) {
    let mut normalized = recurrence.clone();
    let mut field_errors = Vec::new();
    let mut warnings = Vec::new();
    if let Err(error) = validate_time_range(event_time, "event_time") {
        push_field_error(&mut field_errors, "event_time", &error.code, &error.message);
    }
    let frequency = recurrence
        .get("frequency")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(frequency, "daily" | "weekly" | "monthly" | "yearly") {
        push_field_error(
            &mut field_errors,
            "recurrence.frequency",
            "unsupported",
            "Frequency must be daily, weekly, monthly, or yearly.",
        );
    }
    let interval = recurrence
        .get("interval")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    if !(1..=99).contains(&interval) {
        push_field_error(
            &mut field_errors,
            "recurrence.interval",
            "out_of_range",
            "Recurrence interval must be 1 to 99.",
        );
    }
    if normalized.get("interval").is_none() {
        normalized["interval"] = json!(1);
    }
    if recurrence
        .get("time_zone")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        push_field_error(
            &mut field_errors,
            "recurrence.time_zone",
            "required",
            "Recurring events require an IANA time zone id.",
        );
    }
    let end = recurrence.get("end").unwrap_or(&Value::Null);
    let mode = end.get("mode").and_then(Value::as_str).unwrap_or("");
    match mode {
        "never" => {
            let allow_never = policy
                .and_then(|policy| policy.get("allow_no_end_recurrence"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !allow_never {
                warnings.push(WarningMessage::new(
                    "policy_no_end",
                    "No-end recurrence requires host policy approval; preview remains capped.",
                ));
            }
        }
        "after_count" => {
            let count = end.get("count").and_then(Value::as_u64).unwrap_or(0);
            if !(1..=999).contains(&count) {
                push_field_error(
                    &mut field_errors,
                    "recurrence.end.count",
                    "out_of_range",
                    "Recurrence count must be 1 to 999.",
                );
            }
        }
        "on_date" => {
            if end
                .get("until")
                .and_then(Value::as_str)
                .and_then(parse_datetime)
                .is_none()
            {
                push_field_error(
                    &mut field_errors,
                    "recurrence.end.until",
                    "invalid",
                    "End date must be an ISO date or date-time.",
                );
            }
        }
        _ => push_field_error(
            &mut field_errors,
            "recurrence.end.mode",
            "required",
            "Recurrence end mode is required.",
        ),
    }
    if frequency == "monthly"
        && recurrence.get("day_of_month").is_some()
        && recurrence.get("month_end_behavior").is_none()
    {
        push_field_error(
            &mut field_errors,
            "recurrence.month_end_behavior",
            "required",
            "Monthly day-of-month recurrence requires month_end_behavior.",
        );
    }
    (field_errors.is_empty(), normalized, field_errors, warnings)
}

fn expand_recurrence(
    event_time: &Value,
    recurrence: &Value,
    limit: usize,
) -> Result<(Vec<Value>, bool), ErrorResponse> {
    let (start, end) = validate_time_range(event_time, "event_time")?;
    let duration = end.to_minutes() - start.to_minutes();
    let frequency = recurrence
        .get("frequency")
        .and_then(Value::as_str)
        .unwrap_or("daily");
    let interval = recurrence
        .get("interval")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .max(1);
    let count_limit = recurrence
        .pointer("/end/count")
        .and_then(Value::as_u64)
        .unwrap_or(limit as u64)
        .min(limit as u64) as usize;
    let until = recurrence
        .pointer("/end/until")
        .and_then(Value::as_str)
        .and_then(parse_datetime);
    let mut occurrences = Vec::new();
    let mut current = start;
    let horizon_minutes = start.to_minutes() + 5 * 366 * 24 * 60;
    let mut iterations = 0usize;
    while occurrences.len() < count_limit
        && current.to_minutes() <= horizon_minutes
        && iterations < 20_000
    {
        if let Some(until) = until {
            if current.to_minutes() > until.to_minutes() {
                break;
            }
        }
        let current_end = current.add_minutes(duration);
        occurrences.push(json!({
            "occurrence_id": format!("occ_{}", occurrences.len() + 1),
            "start": current.format_datetime(),
            "end": current_end.format_datetime(),
            "time_zone": recurrence.get("time_zone").cloned().unwrap_or_else(|| event_time.get("time_zone").cloned().unwrap_or(json!(DEFAULT_TIME_ZONE))),
            "is_exception": false
        }));
        current = match frequency {
            "weekly" => current.add_days(7 * interval),
            "monthly" => current.add_months(interval as i32),
            "yearly" => current.add_months((12 * interval) as i32),
            _ => current.add_days(interval),
        };
        iterations += 1;
    }
    let truncated = recurrence
        .pointer("/end/count")
        .and_then(Value::as_u64)
        .map_or(false, |count| count as usize > occurrences.len())
        || recurrence.pointer("/end/mode").and_then(Value::as_str) == Some("never");
    Ok((occurrences, truncated))
}

fn score_meeting_times(
    draft: &Value,
    snapshots: &[Value],
    rooms_snapshot: Option<&Value>,
) -> Vec<Value> {
    let start = draft
        .get("start")
        .and_then(Value::as_str)
        .and_then(parse_datetime)
        .unwrap_or(DateTimeParts {
            year: 1970,
            month: 1,
            day: 1,
            hour: 9,
            minute: 0,
            second: 0,
        });
    let end = draft
        .get("end")
        .and_then(Value::as_str)
        .and_then(parse_datetime)
        .unwrap_or(start.add_minutes(30));
    let duration = (end.to_minutes() - start.to_minutes()).max(15);
    let time_zone = draft
        .get("time_zone")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_TIME_ZONE);
    let mut suggestions = Vec::new();
    for index in 0..16 {
        let slot_start = start.add_minutes(index * 30);
        let slot_end = slot_start.add_minutes(duration);
        let mut conflicts = Vec::new();
        for snapshot in snapshots {
            let subject_id = snapshot
                .get("subject_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            for block in snapshot
                .get("blocks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let availability = block
                    .get("availability")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                if availability != "free" && range_overlaps(slot_start, slot_end, block) {
                    conflicts.push(json!({ "subject_id": subject_id, "availability": availability, "block": block }));
                }
            }
        }
        let within_work_hours = (8..18).contains(&slot_start.hour);
        let room_available = rooms_snapshot.map_or(true, |rooms| {
            rooms.as_array().map_or(true, |items| !items.is_empty())
        });
        let score = 100_i64 - (conflicts.len() as i64 * 20)
            + if within_work_hours { 10 } else { -10 }
            + if room_available { 5 } else { -15 };
        suggestions.push(json!({
            "start": slot_start.format_datetime(),
            "end": slot_end.format_datetime(),
            "time_zone": time_zone,
            "score": score.max(0),
            "required_status": if conflicts.is_empty() { "free" } else { "conflicted" },
            "optional_status": "not_evaluated",
            "room_status": if room_available { "available_or_not_requested" } else { "unavailable" },
            "within_work_hours": within_work_hours,
            "conflicts": conflicts,
            "reasons": if conflicts.is_empty() { vec![json!("required_attendees_free"), json!("least_conflicts")] } else { vec![json!("least_conflicts")] },
            "warnings": []
        }));
    }
    suggestions.sort_by(|left, right| {
        right
            .get("score")
            .and_then(Value::as_i64)
            .cmp(&left.get("score").and_then(Value::as_i64))
    });
    suggestions.truncate(5);
    suggestions
}

fn range_overlaps(slot_start: DateTimeParts, slot_end: DateTimeParts, block: &Value) -> bool {
    let block_start = block
        .get("start")
        .and_then(Value::as_str)
        .and_then(parse_datetime);
    let block_end = block
        .get("end")
        .and_then(Value::as_str)
        .and_then(parse_datetime);
    match (block_start, block_end) {
        (Some(block_start), Some(block_end)) => {
            slot_start.to_minutes() < block_end.to_minutes()
                && block_start.to_minutes() < slot_end.to_minutes()
        }
        _ => false,
    }
}

fn dispatch_operation(operation: &'static str, input: &[u8]) -> AppResult {
    let parsed: Result<Value, _> = serde_json::from_slice(input);
    let request = match parsed {
        Ok(value) => value,
        Err(error) => {
            return AppResult::Err {
                err: ErrorResponse::new(
                    "InvalidInput",
                    format!("Input must be a UTF-8 JSON object: {error}"),
                ),
            }
        }
    };
    let response = match operation {
        "get_calendar_capabilities" => handle_get_calendar_capabilities(&request),
        "list_calendars" => handle_list_calendars(&request),
        "save_calendar" => handle_save_calendar(&request),
        "delete_calendar" => handle_delete_calendar(&request),
        "set_calendar_view_state" => handle_set_calendar_view_state(&request),
        "list_events" => handle_list_events(&request),
        "get_event" => handle_get_event(&request),
        "search_events" => handle_search_events(&request),
        "validate_event_draft" => handle_validate_event_draft(&request),
        "create_event" => handle_create_event(&request),
        "update_event" => handle_update_event(&request),
        "delete_event" => handle_delete_event(&request),
        "copy_event" => handle_copy_event(&request),
        "move_event" => handle_move_event(&request),
        "validate_recurrence" => handle_validate_recurrence(&request),
        "preview_recurrence" => handle_preview_recurrence(&request),
        "plan_free_busy_lookup" => handle_plan_free_busy_lookup(&request),
        "suggest_meeting_times" => handle_suggest_meeting_times(&request),
        "search_rooms" => handle_search_rooms(&request),
        "resolve_attendees" => handle_resolve_attendees(&request),
        "list_invitations" => handle_list_invitations(&request),
        "get_invitation" => handle_get_invitation(&request),
        "respond_to_invitation" => handle_respond_to_invitation(&request),
        "propose_new_time" => handle_propose_new_time(&request),
        "list_categories" => handle_list_categories(&request),
        "save_category" => handle_save_category(&request),
        "delete_category" => handle_delete_category(&request),
        "bulk_update_events" => handle_bulk_update_events(&request),
        "get_sharing_state" => handle_get_sharing_state(&request),
        "update_sharing" => handle_update_sharing(&request),
        "accept_shared_calendar" => handle_accept_shared_calendar(&request),
        "subscribe_calendar" => handle_subscribe_calendar(&request),
        "get_reminder_state" => handle_get_reminder_state(&request),
        "snooze_or_dismiss_reminder" => handle_snooze_or_dismiss_reminder(&request),
        "get_user_preferences" => handle_get_user_preferences(&request),
        "save_user_preferences" => handle_save_user_preferences(&request),
        "get_offline_state" => handle_get_offline_state(&request),
        "get_activity_log" => handle_get_activity_log(&request),
        _ => Err(ErrorResponse::new(
            "InvalidInput",
            "Unknown calendar operation.",
        )),
    };
    match response {
        Ok(ok_response) => AppResult::Ok { ok: ok_response },
        Err(error) => AppResult::Err { err: error },
    }
}

fn write_result(result: AppResult) -> *mut u8 {
    let body = serde_json::to_string(&result).unwrap_or_else(|error| {
        serde_json::to_string(&AppResult::Err {
            err: ErrorResponse::new(
                "InternalError",
                format!("Failed to serialize response: {error}"),
            ),
        })
        .unwrap_or_else(|_| {
            "{\"err\":{\"code\":\"InternalError\",\"message\":\"serialization failed\"}}"
                .to_string()
        })
    });
    CString::new(body)
        .expect("JSON serializer never writes interior nul bytes")
        .into_raw() as *mut u8
}

unsafe fn read_input<'a>(input_ptr: *const u8, input_len: usize) -> &'a [u8] {
    if input_ptr.is_null() || input_len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(input_ptr, input_len)
    }
}

macro_rules! export_operation {
    ($name:ident) => {
        #[no_mangle]
        pub extern "C" fn $name(input_ptr: *const u8, input_len: usize) -> *mut u8 {
            let input = unsafe { read_input(input_ptr, input_len) };
            write_result(dispatch_operation(stringify!($name), input))
        }
    };
}

export_operation!(get_calendar_capabilities);
export_operation!(list_calendars);
export_operation!(save_calendar);
export_operation!(delete_calendar);
export_operation!(set_calendar_view_state);
export_operation!(list_events);
export_operation!(get_event);
export_operation!(search_events);
export_operation!(validate_event_draft);
export_operation!(create_event);
export_operation!(update_event);
export_operation!(delete_event);
export_operation!(copy_event);
export_operation!(move_event);
export_operation!(validate_recurrence);
export_operation!(preview_recurrence);
export_operation!(plan_free_busy_lookup);
export_operation!(suggest_meeting_times);
export_operation!(search_rooms);
export_operation!(resolve_attendees);
export_operation!(list_invitations);
export_operation!(get_invitation);
export_operation!(respond_to_invitation);
export_operation!(propose_new_time);
export_operation!(list_categories);
export_operation!(save_category);
export_operation!(delete_category);
export_operation!(bulk_update_events);
export_operation!(get_sharing_state);
export_operation!(update_sharing);
export_operation!(accept_shared_calendar);
export_operation!(subscribe_calendar);
export_operation!(get_reminder_state);
export_operation!(snooze_or_dismiss_reminder);
export_operation!(get_user_preferences);
export_operation!(save_user_preferences);
export_operation!(get_offline_state);
export_operation!(get_activity_log);

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

fn format_permission(permission: Permission) -> &'static str {
    match permission {
        Permission::None => "none",
        Permission::Read => "read",
        Permission::Write => "write",
        Permission::Invite => "invite",
        Permission::Respond => "respond",
        Permission::Share => "share",
        Permission::Delegate => "delegate",
        Permission::Settings => "settings",
        Permission::Audit => "audit",
        Permission::Admin => "admin",
    }
}

fn stable_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut pairs: Vec<_> = map.iter().collect();
            pairs.sort_by(|left, right| left.0.cmp(right.0));
            let inner = pairs
                .into_iter()
                .map(|(key, value)| format!("\"{}\":{}", key, stable_json(value)))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{inner}}}")
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(stable_json).collect::<Vec<_>>().join(",")
        ),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    }
}

fn hash_text(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateTimeParts {
    year: i32,
    month: u32,
    day: u32,
    hour: i64,
    minute: i64,
    second: i64,
}

impl DateTimeParts {
    fn to_minutes(self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 24 * 60 + self.hour * 60 + self.minute
    }

    fn add_minutes(self, minutes: i64) -> Self {
        let total = self.to_minutes() + minutes;
        let days = total.div_euclid(24 * 60);
        let minute_of_day = total.rem_euclid(24 * 60);
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            hour: minute_of_day / 60,
            minute: minute_of_day % 60,
            second: self.second,
        }
    }

    fn add_days(self, days: i64) -> Self {
        self.add_minutes(days * 24 * 60)
    }

    fn add_months(self, months: i32) -> Self {
        let total_months = self.year * 12 + self.month as i32 - 1 + months;
        let year = total_months.div_euclid(12);
        let month = total_months.rem_euclid(12) as u32 + 1;
        let day = self.day.min(days_in_month(year, month));
        Self {
            year,
            month,
            day,
            ..self
        }
    }

    fn format_datetime(self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

fn parse_datetime(value: &str) -> Option<DateTimeParts> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 {
        return None;
    }
    let year = value.get(0..4)?.parse::<i32>().ok()?;
    let month = value.get(5..7)?.parse::<u32>().ok()?;
    let day = value.get(8..10)?.parse::<u32>().ok()?;
    if value.len() == 10 {
        return valid_date(year, month, day).then_some(DateTimeParts {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
        });
    }
    let hour = value.get(11..13)?.parse::<i64>().ok()?;
    let minute = value.get(14..16)?.parse::<i64>().ok()?;
    let second = value
        .get(17..19)
        .and_then(|part| part.parse::<i64>().ok())
        .unwrap_or(0);
    if !valid_date(year, month, day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=60).contains(&second)
    {
        return None;
    }
    Some(DateTimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

fn valid_date(year: i32, month: u32, day: u32) -> bool {
    (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) as i64
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i32::from(month <= 2);
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPORTS: &[&str] = &[
        "get_calendar_capabilities",
        "list_calendars",
        "save_calendar",
        "delete_calendar",
        "set_calendar_view_state",
        "list_events",
        "get_event",
        "search_events",
        "validate_event_draft",
        "create_event",
        "update_event",
        "delete_event",
        "copy_event",
        "move_event",
        "validate_recurrence",
        "preview_recurrence",
        "plan_free_busy_lookup",
        "suggest_meeting_times",
        "search_rooms",
        "resolve_attendees",
        "list_invitations",
        "get_invitation",
        "respond_to_invitation",
        "propose_new_time",
        "list_categories",
        "save_category",
        "delete_category",
        "bulk_update_events",
        "get_sharing_state",
        "update_sharing",
        "accept_shared_calendar",
        "subscribe_calendar",
        "get_reminder_state",
        "snooze_or_dismiss_reminder",
        "get_user_preferences",
        "save_user_preferences",
        "get_offline_state",
        "get_activity_log",
    ];

    fn context(permission: &str) -> Value {
        json!({
            "user_id": "usr_123",
            "permission": permission,
            "capabilities": ["calendar:read", "calendar:write", "calendar:invite", "calendar:respond", "calendar:share", "calendar:delegate", "calendar:settings", "calendar:audit"],
            "available_effects": ALL_EFFECTS,
            "request_id": "req_456",
            "locale": "en-US",
            "time_zone": "Europe/Berlin",
            "policy": { "allow_no_end_recurrence": true }
        })
    }

    fn base_event() -> Value {
        json!({
            "event_id": "evt_1",
            "calendar_id": "cal_1",
            "kind": "meeting",
            "status": "confirmed",
            "title": "Planning",
            "start": "2026-05-08T09:00:00Z",
            "end": "2026-05-08T10:00:00Z",
            "all_day": false,
            "time_zone": "Europe/Berlin",
            "availability": "busy",
            "privacy": "normal",
            "categories": [],
            "reminders": [],
            "attendees": [{ "email": "ada@example.com", "kind": "required", "response_status": "none" }],
            "revision": "rev_1",
            "created_at": "2026-05-01T00:00:00Z",
            "updated_at": "2026-05-01T00:00:00Z"
        })
    }

    fn base_range() -> Value {
        json!({ "start": "2026-05-08T09:00:00Z", "end": "2026-05-08T17:00:00Z", "time_zone": "Europe/Berlin" })
    }

    fn recurrence() -> Value {
        json!({
            "frequency": "weekly",
            "interval": 1,
            "days_of_week": ["FR"],
            "end": { "mode": "after_count", "count": 4 },
            "time_zone": "Europe/Berlin"
        })
    }

    fn success_request(operation: &str) -> Value {
        let mut request = json!({ "context": context("admin") });
        match operation {
            "get_calendar_capabilities" => request["calendar_ids"] = json!(["cal_1"]),
            "list_calendars" => {
                request["snapshot"] = json!([default_calendar(
                    &RequestContext::from_request(&request).unwrap()
                )])
            }
            "save_calendar" => {
                request["calendar"] =
                    default_calendar(&RequestContext::from_request(&request).unwrap())
            }
            "delete_calendar" => {
                request["calendar_id"] = json!("cal_1");
                request["mode"] = json!("delete");
                request["confirmation"] = json!({ "confirmed": true });
            }
            "set_calendar_view_state" => {
                request["view_state"] = json!({ "selected_calendar_ids": ["cal_1"], "hidden_calendar_ids": [], "display_order": ["cal_1"], "mode": "overlay", "view": "week" })
            }
            "list_events" => {
                request["range"] = base_range();
                request["calendar_ids"] = json!(["cal_1"]);
                request["snapshot"] = json!([base_event()]);
            }
            "get_event" => {
                request["event_id"] = json!("evt_1");
                request["snapshot"] = json!([base_event()]);
            }
            "search_events" => {
                request["query"] = json!("Planning");
                request["snapshot"] = json!([base_event()]);
            }
            "validate_event_draft" => request["draft"] = base_event(),
            "create_event" => {
                request["draft"] = base_event();
                request["send_invites"] = json!(true);
            }
            "update_event" => {
                request["event_id"] = json!("evt_1");
                request["patch"] = json!({ "title": "Updated" });
                request["scope"] = json!("single");
                request["notification_scope"] = json!("none");
            }
            "delete_event" => {
                request["target"] = json!({ "event_id": "evt_1" });
                request["scope"] = json!("single");
                request["mode"] = json!("delete");
                request["confirmation"] = json!({ "confirmed": true });
            }
            "copy_event" => {
                request["event_id"] = json!("evt_1");
                request["destination_calendar_id"] = json!("cal_2");
            }
            "move_event" => {
                request["event_id"] = json!("evt_1");
                request["destination_calendar_id"] = json!("cal_2");
            }
            "validate_recurrence" => {
                request["event_time"] = base_range();
                request["recurrence"] = recurrence();
            }
            "preview_recurrence" => {
                request["event_time"] = base_range();
                request["recurrence"] = recurrence();
                request["limit"] = json!(4);
            }
            "plan_free_busy_lookup" => {
                request["attendees"] = json!([{ "email": "ada@example.com", "kind": "required" }]);
                request["range"] = base_range();
            }
            "suggest_meeting_times" => {
                request["meeting_draft"] = base_event();
                request["free_busy_snapshot"] = json!([{ "subject_id": "ada", "subject_kind": "attendee", "range": base_range(), "time_zone": "Europe/Berlin", "blocks": [] }]);
            }
            "search_rooms" => request["range"] = base_range(),
            "resolve_attendees" => {
                request["attendees"] = json!([{ "email": "Ada@Example.COM", "kind": "required" }])
            }
            "list_invitations" => request["snapshot"] = json!([]),
            "get_invitation" => request["invitation_id"] = json!("inv_1"),
            "respond_to_invitation" => {
                request["invitation_id"] = json!("inv_1");
                request["response"] = json!("accept");
            }
            "propose_new_time" => {
                request["invitation_id"] = json!("inv_1");
                request["proposed_time"] = base_range();
            }
            "list_categories" => request["snapshot"] = json!([]),
            "save_category" => request["category"] = json!({ "name": "Focus", "color": "#2563eb" }),
            "delete_category" => {
                request["category_id"] = json!("cat_1");
                request["confirmation"] = json!({ "confirmed": true });
            }
            "bulk_update_events" => {
                request["selection"] = json!({ "event_ids": ["evt_1"] });
                request["operation"] = json!({ "type": "category_add", "category_id": "cat_1" });
                request["confirmation"] = json!({ "confirmed": true });
            }
            "get_sharing_state" => request["calendar_id"] = json!("cal_1"),
            "update_sharing" => {
                request["calendar_id"] = json!("cal_1");
                request["changes"] = json!([{ "principal": { "email": "ada@example.com" }, "access_level": "view" }]);
                request["confirmation"] = json!({ "confirmed": true });
            }
            "accept_shared_calendar" => request["share_invitation_id"] = json!("share_1"),
            "subscribe_calendar" => {
                request["subscription"] = json!({ "subscription_ref": "host_sub_1", "display_name": "Team", "color": "#2563eb" });
                request["confirmation"] = json!({ "confirmed": true });
            }
            "get_reminder_state" => request["event_id"] = json!("evt_1"),
            "snooze_or_dismiss_reminder" => {
                request["reminder_id"] = json!("rem_1");
                request["action"] = json!("dismiss");
            }
            "get_user_preferences" => request["include_defaults"] = json!(true),
            "save_user_preferences" => {
                request["preferences"] =
                    base_preferences(&RequestContext::from_request(&request).unwrap())
            }
            "get_offline_state" => request["calendar_ids"] = json!(["cal_1"]),
            "get_activity_log" => request["snapshot"] = json!([]),
            _ => {}
        }
        request
    }

    fn call(operation: &'static str, request: Value) -> Value {
        let input = serde_json::to_vec(&request).unwrap();
        match dispatch_operation(operation, &input) {
            AppResult::Ok { ok } => json!({ "ok": ok }),
            AppResult::Err { err } => json!({ "err": err }),
        }
    }

    fn assert_err_code(response: Value, expected_code: &str) {
        assert_eq!(
            response.pointer("/err/code").and_then(Value::as_str),
            Some(expected_code),
            "unexpected response: {response}"
        );
    }

    #[test]
    fn every_export_accepts_valid_contract_input() {
        for operation in EXPORTS {
            let response = call(operation, success_request(operation));
            assert!(
                response.get("ok").is_some(),
                "{operation} failed with {response}"
            );
            assert_eq!(
                response.pointer("/ok/operation").and_then(Value::as_str),
                Some(*operation)
            );
        }
    }

    #[test]
    fn every_export_rejects_missing_context() {
        for operation in EXPORTS {
            let response = call(operation, json!({}));
            assert_eq!(
                response.pointer("/err/code").and_then(Value::as_str),
                Some("InvalidInput"),
                "{operation} should reject missing context"
            );
        }
    }

    #[test]
    fn every_export_rejects_missing_permission() {
        for operation in EXPORTS {
            let mut request = success_request(operation);
            request["context"] = context("none");
            let response = call(operation, request);
            assert_eq!(
                response.pointer("/err/code").and_then(Value::as_str),
                Some("PermissionDenied"),
                "{operation} should reject permission none: {response}"
            );
        }
    }

    #[test]
    fn recurrence_validation_and_preview_are_deterministic() {
        let response = call("preview_recurrence", success_request("preview_recurrence"));
        let occurrences = response
            .pointer("/ok/data/occurrences")
            .and_then(Value::as_array)
            .unwrap();
        assert_eq!(occurrences.len(), 4);
        assert_eq!(
            occurrences[0].get("start").and_then(Value::as_str),
            Some("2026-05-08T09:00:00Z")
        );
        assert_eq!(
            occurrences[1].get("start").and_then(Value::as_str),
            Some("2026-05-15T09:00:00Z")
        );
    }

    #[test]
    fn event_validation_reports_missing_meeting_attendee() {
        let mut request = success_request("validate_event_draft");
        request["draft"]["attendees"] = json!([]);
        let response = call("validate_event_draft", request);
        assert_eq!(
            response.pointer("/ok/data/valid").and_then(Value::as_bool),
            Some(false)
        );
        assert!(!response
            .pointer("/ok/data/required_actions")
            .and_then(Value::as_array)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn suggested_times_avoid_busy_blocks() {
        let mut request = success_request("suggest_meeting_times");
        request["free_busy_snapshot"] = json!([{
            "subject_id": "ada",
            "subject_kind": "attendee",
            "range": base_range(),
            "time_zone": "Europe/Berlin",
            "blocks": [{ "start": "2026-05-08T09:00:00Z", "end": "2026-05-08T10:00:00Z", "availability": "busy" }]
        }]);
        let response = call("suggest_meeting_times", request);
        let first = response
            .pointer("/ok/data/suggestions/0/start")
            .and_then(Value::as_str)
            .unwrap();
        assert_ne!(first, "2026-05-08T09:00:00Z");
    }

    #[test]
    fn limited_event_projection_strips_private_content() {
        let mut event = base_event();
        event["title"] = json!("Private acquisition review");
        event["location"] = json!("Board room");
        event["body_text"] = json!("Sensitive agenda");
        event["body_html"] = json!("<p>Sensitive agenda</p>");
        event["attendees"] = json!([{ "email": "ceo@example.com", "kind": "required" }]);
        event["organizer"] = json!({ "email": "founder@example.com" });
        event["categories"] = json!(["cat_secret"]);

        let mut request = success_request("list_events");
        request["snapshot"] = json!([event]);
        request["projection"] = json!("limited");

        let response = call("list_events", request);
        let projected = response.pointer("/ok/data/events/0").unwrap();
        assert_eq!(projected.get("title").and_then(Value::as_str), Some("Busy"));
        assert!(projected.get("location").is_none(), "limited projection must hide location: {projected}");
        assert!(projected.get("body_text").is_none(), "limited projection must hide notes: {projected}");
        assert!(projected.get("body_html").is_none(), "limited projection must hide HTML body: {projected}");
        assert!(projected.get("attendees").is_none(), "limited projection must hide attendees: {projected}");
        assert!(projected.get("organizer").is_none(), "limited projection must hide organizer: {projected}");
        assert!(projected.get("categories").is_none(), "limited projection must hide categories: {projected}");
    }

    #[test]
    fn host_effect_plans_include_stable_idempotency_keys() {
        let response = call("create_event", success_request("create_event"));
        let effects = response
            .pointer("/ok/host_effects")
            .and_then(Value::as_array)
            .unwrap();
        assert!(effects
            .iter()
            .any(|effect| effect.get("effect").and_then(Value::as_str)
                == Some(EFFECT_CALENDAR_STORE_WRITE)));
        assert!(effects.iter().all(|effect| effect
            .get("idempotency_key")
            .and_then(Value::as_str)
            .unwrap()
            .contains("req_456")));
    }

    #[test]
    fn basic_event_save_requires_write_permission_and_returns_preview() {
        let mut read_only_request = success_request("create_event");
        read_only_request["context"] = context("read");
        read_only_request["draft"]["kind"] = json!("appointment");
        read_only_request["draft"]["attendees"] = json!([]);
        read_only_request["draft"]["reminders"] = json!([]);
        read_only_request["send_invites"] = json!(false);

        let read_only_response = call("create_event", read_only_request);
        assert_err_code(read_only_response, "PermissionDenied");

        let mut write_request = success_request("create_event");
        write_request["context"] = context("write");
        write_request["draft"]["kind"] = json!("appointment");
        write_request["draft"]["title"] = json!("Planning review");
        write_request["draft"]["attendees"] = json!([]);
        write_request["draft"]["reminders"] = json!([]);
        write_request["send_invites"] = json!(false);

        let write_response = call("create_event", write_request);
        assert_eq!(
            write_response
                .pointer("/ok/data/event_preview/title")
                .and_then(Value::as_str),
            Some("Planning review"),
            "write-capable save should return an event preview: {write_response}"
        );
        assert_eq!(
            write_response
                .pointer("/ok/data/persistence_status")
                .and_then(Value::as_str),
            Some("planned")
        );
    }

    #[test]
    fn unavailable_required_effect_is_reported() {
        let mut request = success_request("plan_free_busy_lookup");
        request["context"]["available_effects"] = json!([EFFECT_CALENDAR_STORE_READ]);
        let response = call("plan_free_busy_lookup", request);
        assert_eq!(
            response.pointer("/err/code").and_then(Value::as_str),
            Some("EffectUnavailable")
        );
    }

    #[test]
    fn invite_permission_is_additive_and_does_not_grant_privileged_operations() {
        let invite = Permission::Invite;
        assert!(invite.satisfies(Permission::Invite));
        assert!(invite.satisfies(Permission::Read));
        assert!(!invite.satisfies(Permission::Write));
        assert!(!invite.satisfies(Permission::Settings));
        assert!(!invite.satisfies(Permission::Share));
        assert!(!invite.satisfies(Permission::Respond));
        assert!(!invite.satisfies(Permission::Audit));

        for operation in [
            "save_calendar",
            "set_calendar_view_state",
            "get_sharing_state",
            "respond_to_invitation",
            "get_activity_log",
        ] {
            let mut request = success_request(operation);
            request["context"] = context("invite");
            assert_err_code(call(operation, request), "PermissionDenied");
        }
    }

    #[test]
    fn subscribe_calendar_only_forwards_sanitized_host_refs() {
        let response = call("subscribe_calendar", success_request("subscribe_calendar"));
        let subscription = response
            .pointer("/ok/host_effects/0/payload/subscription")
            .unwrap();
        assert_eq!(
            subscription.get("subscription_ref").and_then(Value::as_str),
            Some("host_sub_1")
        );
        assert!(subscription.get("url").is_none());
        assert!(subscription.get("token").is_none());
    }

    #[test]
    fn subscribe_calendar_rejects_raw_provider_fields_and_urls() {
        for field in RAW_SUBSCRIPTION_FIELDS {
            let mut request = success_request("subscribe_calendar");
            let mut subscription = Map::new();
            subscription.insert("subscription_ref".to_string(), json!("host_sub_1"));
            subscription.insert((*field).to_string(), json!("raw-secret"));
            request["subscription"] = Value::Object(subscription);
            assert_err_code(call("subscribe_calendar", request), "InvalidInput");
        }

        let mut request = success_request("subscribe_calendar");
        request["subscription"] = json!({ "url_ref": "https://calendar.example.test/feed.ics" });
        assert_err_code(call("subscribe_calendar", request), "InvalidInput");

        let mut request = success_request("subscribe_calendar");
        request["subscription"] = json!({ "subscription_ref": "host_sub_1", "display_name": "https://calendar.example.test/feed.ics" });
        assert_err_code(call("subscribe_calendar", request), "InvalidInput");
    }

    #[test]
    fn unknown_planner_enum_values_are_rejected_before_planning_effects() {
        let cases: [(&str, &str, Value); 9] = [
            ("delete_calendar", "mode", json!("archive")),
            ("update_event", "scope", json!("future")),
            ("update_event", "notification_scope", json!("external")),
            ("delete_event", "mode", json!("purge")),
            ("delete_event", "scope", json!("future")),
            ("respond_to_invitation", "response", json!("maybe")),
            ("respond_to_invitation", "scope", json!("all_future")),
            ("propose_new_time", "scope", json!("all_future")),
            ("snooze_or_dismiss_reminder", "action", json!("delay")),
        ];

        for (operation, field, value) in cases {
            let mut request = success_request(operation);
            request[field] = value;
            assert_err_code(call(operation, request), "InvalidInput");
        }

        let mut request = success_request("move_event");
        request["notification_scope"] = json!("external");
        assert_err_code(call("move_event", request), "InvalidInput");
    }
}
