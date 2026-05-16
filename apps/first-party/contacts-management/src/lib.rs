#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const APP_ID: &str = "contacts-management";

pub const ACTION_CONTACTS_LIST: &str = "contacts.list";
pub const ACTION_CONTACTS_CREATE: &str = "contacts.create";
pub const ACTION_CONTACTS_UPDATE: &str = "contacts.update";
pub const ACTION_CONTACTS_DELETE: &str = "contacts.delete";
pub const ACTION_CONTACTS_SYNC_DIRECTORY: &str = "contacts.sync-directory";

const COLLECTION_CONTACTS: &str = "contacts";
const MAX_CONTACT_NAME_LENGTH: usize = 200;
const MAX_TEXT_LENGTH: usize = 2_000;
const MAX_TAGS: usize = 20;
const DEFAULT_LIST_LIMIT: i32 = 50;
const MAX_LIST_LIMIT: i32 = 500;
const BLOCKED_SYNC_REASON: &str = "platform user enumeration contract unavailable";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostEffect {
    PutDocument {
        collection: String,
        doc_id: String,
        data: Value,
    },
    QueryDocuments {
        collection: String,
        filters: BTreeMap<String, String>,
        limit: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppActionResult {
    pub ok: bool,
    pub payload: Value,
    pub effects: Vec<HostEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallerCtx {
    pub actor_id: String,
    pub actor_type: String,
    pub principal_id: String,
}

pub fn tool_registration_payload() -> Value {
    json!({
        "app_id": APP_ID,
        "tools": [
            {
                "action_name": ACTION_CONTACTS_LIST,
                "description": "List contacts visible to the current principal.",
                "json_schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "group_id": { "type": ["string", "null"], "maxLength": 128 },
                        "query": { "type": ["string", "null"], "maxLength": 256 },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 500 }
                    }
                },
                "required_scope": "contacts:read"
            },
            {
                "action_name": ACTION_CONTACTS_CREATE,
                "description": "Create a new caller-scoped contact.",
                "json_schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "contact": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "display_name": { "type": "string", "minLength": 1, "maxLength": 200 },
                                "email": { "type": ["string", "null"], "maxLength": 320 },
                                "phone": { "type": ["string", "null"], "maxLength": 64 },
                                "company": { "type": ["string", "null"], "maxLength": 200 },
                                "title": { "type": ["string", "null"], "maxLength": 200 },
                                "group_id": { "type": ["string", "null"], "maxLength": 128 },
                                "notes": { "type": ["string", "null"], "maxLength": 2000 },
                                "favorite": { "type": "boolean" },
                                "tags": { "type": "array", "items": { "type": "string", "maxLength": 64 }, "maxItems": 20 }
                            },
                            "required": ["display_name"]
                        }
                    },
                    "required": ["contact"]
                },
                "required_scope": "contacts:write"
            },
            {
                "action_name": ACTION_CONTACTS_UPDATE,
                "description": "Update an existing caller-scoped contact.",
                "json_schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "contact_id": { "type": "string", "minLength": 1, "maxLength": 128 },
                        "contact": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "display_name": { "type": "string", "minLength": 1, "maxLength": 200 },
                                "email": { "type": ["string", "null"], "maxLength": 320 },
                                "phone": { "type": ["string", "null"], "maxLength": 64 },
                                "company": { "type": ["string", "null"], "maxLength": 200 },
                                "title": { "type": ["string", "null"], "maxLength": 200 },
                                "group_id": { "type": ["string", "null"], "maxLength": 128 },
                                "notes": { "type": ["string", "null"], "maxLength": 2000 },
                                "favorite": { "type": "boolean" },
                                "tags": { "type": "array", "items": { "type": "string", "maxLength": 64 }, "maxItems": 20 }
                            },
                            "required": ["display_name"]
                        },
                        "revision": { "type": ["string", "null"], "maxLength": 128 }
                    },
                    "required": ["contact_id", "contact"]
                },
                "required_scope": "contacts:write"
            },
            {
                "action_name": ACTION_CONTACTS_DELETE,
                "description": "Soft-delete a caller-scoped contact.",
                "json_schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "contact_id": { "type": "string", "minLength": 1, "maxLength": 128 }
                    },
                    "required": ["contact_id"]
                },
                "required_scope": "contacts:write"
            },
            {
                "action_name": ACTION_CONTACTS_SYNC_DIRECTORY,
                "description": "Plan syncing the caller's organization directory; returns a blocked state when the platform enumeration contract is unavailable.",
                "json_schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "directory_scope": { "type": ["string", "null"], "maxLength": 128 },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 500 }
                    }
                },
                "required_scope": "contacts:read"
            }
        ]
    })
}

pub fn settings_schema_payload() -> Value {
    json!({
        "app_id": APP_ID,
        "title": "Contacts Management",
        "description": "Settings for the contacts workspace and directory sync behavior.",
        "fields": [
            {
                "key": "default_view",
                "label": "Default View",
                "field_type": "string",
                "required": true,
                "default": "people",
                "help": "Initial workspace view for the contacts app."
            },
            {
                "key": "compact_density",
                "label": "Compact Density",
                "field_type": "boolean",
                "required": true,
                "default": false,
                "help": "Render tighter rows in the contact list."
            },
            {
                "key": "directory_sync_enabled",
                "label": "Directory Sync Enabled",
                "field_type": "boolean",
                "required": true,
                "default": false,
                "help": "Enable directory sync planning once the platform exposes a user enumeration contract."
            }
        ]
    })
}

pub fn execute_action(action_name: &str, arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    match action_name {
        ACTION_CONTACTS_LIST => action_list(arguments, caller),
        ACTION_CONTACTS_CREATE => action_create(arguments, caller),
        ACTION_CONTACTS_UPDATE => action_update(arguments, caller),
        ACTION_CONTACTS_DELETE => action_delete(arguments, caller),
        ACTION_CONTACTS_SYNC_DIRECTORY => action_sync_directory(arguments, caller),
        _ => AppActionResult {
            ok: false,
            payload: json!({
                "error": format!("Unknown action: {action_name}"),
            }),
            effects: vec![],
        },
    }
}

fn action_list(arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    let limit = match parse_limit(arguments.get("limit")) {
        Ok(value) => value,
        Err(error) => return error,
    };

    let mut filters = BTreeMap::new();
    filters.insert("deleted".to_string(), "false".to_string());

    match optional_string(arguments.get("group_id"), 128, "group_id") {
        Ok(Some(value)) => {
            filters.insert("group_id".to_string(), value);
        }
        Ok(None) => {}
        Err(error) => return error,
    }

    match optional_string(arguments.get("query"), 256, "query") {
        Ok(Some(value)) => {
            filters.insert("query".to_string(), value);
        }
        Ok(None) => {}
        Err(error) => return error,
    }

    AppActionResult {
        ok: true,
        payload: json!({
            "status": "queued",
            "principal_id": caller.principal_id,
            "limit": limit,
        }),
        effects: vec![HostEffect::QueryDocuments {
            collection: COLLECTION_CONTACTS.to_string(),
            filters,
            limit,
        }],
    }
}

fn action_create(arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    let contact = match arguments.get("contact") {
        Some(value) if value.is_object() => value,
        _ => {
            return validation_error("contact is required and must be an object");
        }
    };

    let data = match build_contact_document(contact, &caller.principal_id, false, None) {
        Ok(value) => value,
        Err(error) => return error,
    };

    AppActionResult {
        ok: true,
        payload: json!({
            "status": "queued",
            "principal_id": caller.principal_id,
        }),
        effects: vec![HostEffect::PutDocument {
            collection: COLLECTION_CONTACTS.to_string(),
            doc_id: String::new(),
            data,
        }],
    }
}

fn action_update(arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    let contact_id = match required_id(arguments.get("contact_id"), "contact_id") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let contact = match arguments.get("contact") {
        Some(value) if value.is_object() => value,
        _ => {
            return validation_error("contact is required and must be an object");
        }
    };

    let revision = match optional_string(arguments.get("revision"), 128, "revision") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let data = match build_contact_document(contact, &caller.principal_id, false, revision) {
        Ok(value) => value,
        Err(error) => return error,
    };

    AppActionResult {
        ok: true,
        payload: json!({
            "status": "queued",
            "principal_id": caller.principal_id,
            "contact_id": contact_id,
        }),
        effects: vec![HostEffect::PutDocument {
            collection: COLLECTION_CONTACTS.to_string(),
            doc_id: contact_id,
            data,
        }],
    }
}

fn action_delete(arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    let contact_id = match required_id(arguments.get("contact_id"), "contact_id") {
        Ok(value) => value,
        Err(error) => return error,
    };

    AppActionResult {
        ok: true,
        payload: json!({
            "status": "queued",
            "principal_id": caller.principal_id,
            "contact_id": contact_id,
            "soft_delete": true,
        }),
        effects: vec![HostEffect::PutDocument {
            collection: COLLECTION_CONTACTS.to_string(),
            doc_id: contact_id,
            data: json!({
                "deleted": true,
                "deleted_reason": "user_requested",
                "principal_id": caller.principal_id,
            }),
        }],
    }
}

fn action_sync_directory(arguments: &Value, caller: &CallerCtx) -> AppActionResult {
    if let Some(value) = arguments.get("directory_scope") {
        if !value.is_null() {
            if let Err(error) = optional_string(Some(value), 128, "directory_scope") {
                return error;
            }
        }
    }

    if let Some(limit) = arguments.get("limit") {
        if !limit.is_null() {
            if let Err(error) = parse_limit(Some(limit)) {
                return error;
            }
        }
    }

    AppActionResult {
        ok: false,
        payload: json!({
            "status": "blocked",
            "reason": BLOCKED_SYNC_REASON,
            "principal_id": caller.principal_id,
            "missing_contract": "user_enumeration",
        }),
        effects: vec![],
    }
}

fn build_contact_document(
    contact: &Value,
    principal_id: &str,
    deleted: bool,
    revision: Option<String>,
) -> Result<Value, AppActionResult> {
    let display_name = required_text(
        contact.get("display_name"),
        "display_name",
        MAX_CONTACT_NAME_LENGTH,
    )?;

    let email = optional_string(contact.get("email"), 320, "email")?;
    let phone = optional_string(contact.get("phone"), 64, "phone")?;
    let company = optional_string(contact.get("company"), 200, "company")?;
    let title = optional_string(contact.get("title"), 200, "title")?;
    let group_id = optional_string(contact.get("group_id"), 128, "group_id")?;
    let notes = optional_string(contact.get("notes"), MAX_TEXT_LENGTH, "notes")?;

    let favorite = contact.get("favorite").and_then(Value::as_bool).unwrap_or(false);
    let tags = parse_tags(contact.get("tags"))?;

    let mut data = Map::new();
    data.insert("display_name".to_string(), json!(display_name));
    data.insert("favorite".to_string(), json!(favorite));
    data.insert("deleted".to_string(), json!(deleted));
    data.insert("principal_id".to_string(), json!(principal_id));
    data.insert("source".to_string(), json!("manual"));
    data.insert("tags".to_string(), json!(tags));

    if let Some(value) = email {
        data.insert("email".to_string(), json!(value));
    }
    if let Some(value) = phone {
        data.insert("phone".to_string(), json!(value));
    }
    if let Some(value) = company {
        data.insert("company".to_string(), json!(value));
    }
    if let Some(value) = title {
        data.insert("title".to_string(), json!(value));
    }
    if let Some(value) = group_id {
        data.insert("group_id".to_string(), json!(value));
    }
    if let Some(value) = notes {
        data.insert("notes".to_string(), json!(value));
    }
    if let Some(value) = revision {
        data.insert("expected_revision".to_string(), json!(value));
    }

    Ok(Value::Object(data))
}

fn parse_tags(value: Option<&Value>) -> Result<Vec<String>, AppActionResult> {
    let Some(value) = value else {
        return Ok(vec![]);
    };

    let Some(array) = value.as_array() else {
        return Err(validation_error("tags must be an array of strings"));
    };

    if array.len() > MAX_TAGS {
        return Err(validation_error("tags exceeds the maximum allowed size"));
    }

    let mut tags = Vec::with_capacity(array.len());
    for entry in array {
        let Some(text) = entry.as_str() else {
            return Err(validation_error("tags must be an array of strings"));
        };
        if text.trim().is_empty() {
            return Err(validation_error("tags entries must be non-empty strings"));
        }
        tags.push(text.trim().to_string());
    }

    Ok(tags)
}

fn parse_limit(value: Option<&Value>) -> Result<i32, AppActionResult> {
    match value {
        Some(raw) => {
            let Some(limit) = raw.as_i64() else {
                return Err(validation_error("limit must be an integer between 1 and 500"));
            };
            if !(1..=MAX_LIST_LIMIT as i64).contains(&limit) {
                return Err(validation_error("limit must be between 1 and 500"));
            }
            Ok(limit as i32)
        }
        None => Ok(DEFAULT_LIST_LIMIT),
    }
}

fn required_id(value: Option<&Value>, field: &str) -> Result<String, AppActionResult> {
    match optional_string(value, 128, field) {
        Ok(Some(text)) => Ok(text),
        Ok(None) => Err(validation_error(&format!("{field} is required"))),
        Err(error) => Err(error),
    }
}

fn required_text(value: Option<&Value>, field: &str, max_length: usize) -> Result<String, AppActionResult> {
    let Some(text) = value.and_then(Value::as_str) else {
        return Err(validation_error(&format!("{field} is required and must be a string")));
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(validation_error(&format!("{field} must be non-empty")));
    }
    if trimmed.len() > max_length {
        return Err(validation_error(&format!("{field} exceeds the maximum length of {max_length}")));
    }

    Ok(trimmed.to_string())
}

fn optional_string(value: Option<&Value>, max_length: usize, field: &str) -> Result<Option<String>, AppActionResult> {
    let Some(value) = value else {
        return Ok(None);
    };

    if value.is_null() {
        return Ok(None);
    }

    let Some(text) = value.as_str() else {
        return Err(validation_error(&format!("{field} must be a string or null")));
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > max_length {
        return Err(validation_error(&format!("{field} exceeds the maximum length of {max_length}")));
    }

    Ok(Some(trimmed.to_string()))
}

fn validation_error(message: &str) -> AppActionResult {
    AppActionResult {
        ok: false,
        payload: json!({
            "error": message,
        }),
        effects: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_caller() -> CallerCtx {
        CallerCtx {
            actor_id: "agent-abc".to_string(),
            actor_type: "agent".to_string(),
            principal_id: "user-123".to_string(),
        }
    }

    #[test]
    fn tool_manifest_is_well_formed() {
        let manifest = tool_registration_payload();
        assert_eq!(manifest["app_id"], APP_ID);

        let tools = manifest["tools"].as_array().expect("tools must be an array");
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().all(|tool| tool["action_name"].is_string()));
    }

    #[test]
    fn settings_schema_is_well_formed() {
        let schema = settings_schema_payload();
        assert_eq!(schema["app_id"], APP_ID);
        assert_eq!(schema["fields"].as_array().expect("fields must be an array").len(), 3);
    }

    #[test]
    fn list_uses_principal_namespace_and_query_effect() {
        let result = execute_action(
            ACTION_CONTACTS_LIST,
            &json!({"group_id": "coworkers", "query": "alba", "limit": 12}),
            &make_caller(),
        );

        assert!(result.ok);
        assert_eq!(result.payload["principal_id"], "user-123");
        assert_eq!(result.effects.len(), 1);

        match &result.effects[0] {
            HostEffect::QueryDocuments { collection, filters, limit } => {
                assert_eq!(collection, COLLECTION_CONTACTS);
                assert_eq!(filters.get("deleted").map(String::as_str), Some("false"));
                assert_eq!(filters.get("group_id").map(String::as_str), Some("coworkers"));
                assert_eq!(filters.get("query").map(String::as_str), Some("alba"));
                assert_eq!(*limit, 12);
            }
            other => panic!("expected QueryDocuments, got {other:?}"),
        }
    }

    #[test]
    fn create_emits_put_document_with_principal_scope() {
        let result = execute_action(
            ACTION_CONTACTS_CREATE,
            &json!({
                "contact": {
                    "display_name": "Alba Ortiz",
                    "email": "alba@example.com",
                    "phone": "+49 30 555 0101",
                    "company": "Northwind",
                    "title": "Design Systems Lead",
                    "group_id": "coworkers",
                    "notes": "Prefers concise follow-ups",
                    "favorite": true,
                    "tags": ["design", "priority"]
                }
            }),
            &make_caller(),
        );

        assert!(result.ok);
        assert_eq!(result.payload["principal_id"], "user-123");
        assert_eq!(result.effects.len(), 1);

        match &result.effects[0] {
            HostEffect::PutDocument { collection, doc_id, data } => {
                assert_eq!(collection, COLLECTION_CONTACTS);
                assert!(doc_id.is_empty(), "create must let the host generate the doc id");
                assert_eq!(data["display_name"], "Alba Ortiz");
                assert_eq!(data["principal_id"], "user-123");
                assert_eq!(data["deleted"], false);
                assert_eq!(data["favorite"], true);
                assert_eq!(data["tags"].as_array().map(|tags| tags.len()), Some(2));
            }
            other => panic!("expected PutDocument, got {other:?}"),
        }
    }

    #[test]
    fn update_requires_contact_id_and_preserves_principal_scope() {
        let result = execute_action(
            ACTION_CONTACTS_UPDATE,
            &json!({
                "contact_id": "contact-42",
                "contact": {
                    "display_name": "Alba Ortiz",
                    "email": "alba.ortiz@example.com",
                    "tags": []
                },
                "revision": "rev-9"
            }),
            &make_caller(),
        );

        assert!(result.ok);
        assert_eq!(result.payload["contact_id"], "contact-42");

        match &result.effects[0] {
            HostEffect::PutDocument { doc_id, data, .. } => {
                assert_eq!(doc_id, "contact-42");
                assert_eq!(data["expected_revision"], "rev-9");
                assert_eq!(data["principal_id"], "user-123");
            }
            other => panic!("expected PutDocument, got {other:?}"),
        }
    }

    #[test]
    fn delete_emits_soft_delete_document() {
        let result = execute_action(
            ACTION_CONTACTS_DELETE,
            &json!({"contact_id": "contact-42"}),
            &make_caller(),
        );

        assert!(result.ok);
        match &result.effects[0] {
            HostEffect::PutDocument { doc_id, data, .. } => {
                assert_eq!(doc_id, "contact-42");
                assert_eq!(data["deleted"], true);
                assert_eq!(data["principal_id"], "user-123");
            }
            other => panic!("expected PutDocument, got {other:?}"),
        }
    }

    #[test]
    fn sync_directory_is_explicitly_blocked_without_effects() {
        let result = execute_action(
            ACTION_CONTACTS_SYNC_DIRECTORY,
            &json!({"directory_scope": "org"}),
            &make_caller(),
        );

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert_eq!(result.payload["status"], "blocked");
        assert!(result.payload["reason"].as_str().unwrap_or_default().contains("unavailable"));
    }

    #[test]
    fn invalid_input_returns_error_without_effects() {
        let result = execute_action(
            ACTION_CONTACTS_CREATE,
            &json!({"contact": {"display_name": ""}}),
            &make_caller(),
        );

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert!(result.payload["error"].as_str().unwrap_or_default().contains("display_name"));
    }

    #[test]
    fn list_rejects_non_integer_limit() {
        let result = execute_action(
            ACTION_CONTACTS_LIST,
            &json!({"limit": "25"}),
            &make_caller(),
        );

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert!(result.payload["error"].as_str().unwrap_or_default().contains("limit"));
    }

    #[test]
    fn sync_directory_rejects_non_integer_limit() {
        let result = execute_action(
            ACTION_CONTACTS_SYNC_DIRECTORY,
            &json!({"directory_scope": "org", "limit": "50"}),
            &make_caller(),
        );

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert!(result.payload["error"].as_str().unwrap_or_default().contains("limit"));
    }

    #[test]
    fn unknown_action_returns_error_without_effects() {
        let result = execute_action("contacts.unknown", &json!({}), &make_caller());

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert!(result.payload["error"].as_str().unwrap_or_default().contains("Unknown action"));
    }
}