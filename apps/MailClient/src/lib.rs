use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Represents an email with full metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Email {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub received_at: i64,
    pub sent_at: i64,
    pub is_read: bool,
    pub folder_id: String,
    pub thread_id: String,
    pub has_attachments: bool,
    pub importance: String,
}

/// Represents a draft email (unsent)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Draft {
    pub draft_id: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub scheduled_send_at: i64,
}

/// Represents a folder/mailbox
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub folder_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub folder_type: String,
    pub unread_count: u32,
    pub total_count: u32,
}

/// Permission levels for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    None,
    Read,
    Draft,
    Send,
}

impl Permission {
    fn from_str(s: &str) -> Self {
        match s {
            "read" => Permission::Read,
            "draft" => Permission::Draft,
            "send" => Permission::Send,
            _ => Permission::None,
        }
    }

    /// Check if this permission satisfies the required level
    fn satisfies(&self, required: Permission) -> bool {
        match (self, required) {
            (Permission::None, Permission::None) => true,
            (Permission::Read, Permission::None) | (Permission::Read, Permission::Read) => true,
            (Permission::Draft, Permission::None)
            | (Permission::Draft, Permission::Read)
            | (Permission::Draft, Permission::Draft) => true,
            (Permission::Send, _) => true,
            _ => false,
        }
    }
}

/// Error types for mail client operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Result wrapper for OK/Error responses
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum AppResult<T: Serialize> {
    Ok(OkResponse<T>),
    Err(ErrResponse),
}

#[derive(Serialize, Debug)]
pub struct OkResponse<T: Serialize> {
    pub ok: T,
}

#[derive(Serialize, Debug)]
pub struct ErrResponse {
    pub err: ErrorResponse,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Parse UTF-8 string from C pointer and length
fn parse_string_from_raw(ptr: *const u8, len: usize) -> Result<String, ErrorResponse> {
    if ptr.is_null() || len == 0 {
        return Err(ErrorResponse {
            code: "InvalidInput".to_string(),
            message: "Empty or null input parameter".to_string(),
        });
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8(bytes.to_vec()).map_err(|_| ErrorResponse {
        code: "InvalidInput".to_string(),
        message: "Invalid UTF-8 encoding".to_string(),
    })
}

/// Parse JSON value from raw string parameter
fn parse_json_from_raw(ptr: *const u8, len: usize) -> Result<Value, ErrorResponse> {
    let json_str = parse_string_from_raw(ptr, len)?;
    serde_json::from_str(&json_str).map_err(|_| ErrorResponse {
        code: "InvalidInput".to_string(),
        message: format!("Invalid JSON format: {}", json_str),
    })
}

/// Validate email address format (basic check)
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
}

/// Allocate and return JSON response as C string
fn return_json_response<T: Serialize>(response: &T) -> *mut u8 {
    let json = serde_json::to_string(response).unwrap_or_else(|_| {
        json!({"err": {"code": "InternalError", "message": "Failed to serialize response"}})
            .to_string()
    });
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

// ============================================================================
// PERMISSION VALIDATION
// ============================================================================

/// Check if agent has required permission, return error if not
fn check_permission(
    user_perm: Permission,
    required: Permission,
) -> Result<(), ErrorResponse> {
    if user_perm.satisfies(required) {
        Ok(())
    } else {
        Err(ErrorResponse {
            code: "PermissionDenied".to_string(),
            message: format!("Agent lacks required permission: {:?}", required),
        })
    }
}

// ============================================================================
// EXPORTED WASM FUNCTIONS
// ============================================================================

/// Function: read_emails
/// Purpose: Fetch paginated list of emails from a mailbox
/// Permission Required: read
#[no_mangle]
pub extern "C" fn read_emails(
    user_id: *const u8,
    user_id_len: usize,
    mailbox: *const u8,
    mailbox_len: usize,
    offset: u32,
    limit: u32,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let mailbox = match parse_string_from_raw(mailbox, mailbox_len) {
        Ok(m) => m,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Read) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Validate input bounds
    let limit = std::cmp::min(limit, 500);
    if limit == 0 {
        let err = ErrorResponse {
            code: "InvalidInput".to_string(),
            message: "Limit must be between 1 and 500".to_string(),
        };
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
    }

    // Platform Effect: QueryDocuments(user_id, collection="emails", mailbox=mailbox, offset=offset, limit=limit)
    // For this stateless implementation, we return effect metadata indicating what platform should query
    let response_data = json!({
        "emails": [],
        "total_count": 0,
        "has_more": false,
        "effects": [
            {
                "type": "QueryDocuments",
                "params": {
                    "user_id": user_id,
                    "collection": "emails",
                    "mailbox": mailbox,
                    "offset": offset,
                    "limit": limit
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: get_email
/// Purpose: Fetch a single email by ID with full content
/// Permission Required: read
#[no_mangle]
pub extern "C" fn get_email(
    user_id: *const u8,
    user_id_len: usize,
    email_id: *const u8,
    email_id_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let email_id = match parse_string_from_raw(email_id, email_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Read) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Platform Effect: QueryDocument(user_id, collection="emails", document_id=email_id)
    // Returns Email or error if not found
    let response_data = json!({
        "effects": [
            {
                "type": "QueryDocument",
                "params": {
                    "user_id": user_id,
                    "collection": "emails",
                    "document_id": email_id
                }
            },
            {
                "type": "UpdateDocument",
                "params": {
                    "user_id": user_id,
                    "collection": "emails",
                    "document_id": email_id,
                    "updates": { "is_read": true }
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: draft_email
/// Purpose: Create a new email draft
/// Permission Required: draft
#[no_mangle]
pub extern "C" fn draft_email(
    user_id: *const u8,
    user_id_len: usize,
    to: *const u8,
    to_len: usize,
    cc: *const u8,
    cc_len: usize,
    bcc: *const u8,
    bcc_len: usize,
    subject: *const u8,
    subject_len: usize,
    body: *const u8,
    body_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let to_json = match parse_json_from_raw(to, to_len) {
        Ok(j) => j,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let cc_json = match parse_json_from_raw(cc, cc_len) {
        Ok(j) => j,
        Err(_) => Value::Array(vec![]),
    };

    let bcc_json = match parse_json_from_raw(bcc, bcc_len) {
        Ok(j) => j,
        Err(_) => Value::Array(vec![]),
    };

    let subject = match parse_string_from_raw(subject, subject_len) {
        Ok(s) => s,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let body = match parse_string_from_raw(body, body_len) {
        Ok(b) => b,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Draft) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Validate recipients
    let empty_vec = vec![];
    let to_array = to_json.as_array().unwrap_or(&empty_vec);
    if to_array.is_empty() {
        let err = ErrorResponse {
            code: "InvalidInput".to_string(),
            message: "At least one recipient (to) is required".to_string(),
        };
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
    }

    // Validate email addresses
    let mut all_recipients = 0;
    for recipient in to_array {
        if let Some(email) = recipient.as_str() {
            if !is_valid_email(email) {
                let err = ErrorResponse {
                    code: "InvalidRecipient".to_string(),
                    message: format!("Invalid email format: '{}'", email),
                };
                return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
            }
            all_recipients += 1;
        }
    }

    if let Some(cc_arr) = cc_json.as_array() {
        for recipient in cc_arr {
            if let Some(email) = recipient.as_str() {
                if !is_valid_email(email) {
                    let err = ErrorResponse {
                        code: "InvalidRecipient".to_string(),
                        message: format!("Invalid email format: '{}'", email),
                    };
                    return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
                }
                all_recipients += 1;
            }
        }
    }

    if let Some(bcc_arr) = bcc_json.as_array() {
        for recipient in bcc_arr {
            if let Some(email) = recipient.as_str() {
                if !is_valid_email(email) {
                    let err = ErrorResponse {
                        code: "InvalidRecipient".to_string(),
                        message: format!("Invalid email format: '{}'", email),
                    };
                    return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
                }
                all_recipients += 1;
            }
        }
    }

    if all_recipients > 100 {
        let err = ErrorResponse {
            code: "TooManyRecipients".to_string(),
            message: format!(
                "Too many recipients: {} (max 100)",
                all_recipients
            ),
        };
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
    }

    // Platform Effect: CreateDocument(user_id, collection="drafts", document_data)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let response_data = json!({
        "draft_id": format!("draft_{}", now),
        "to": to_array,
        "cc": cc_json.as_array().unwrap_or(&vec![]),
        "bcc": bcc_json.as_array().unwrap_or(&vec![]),
        "subject": subject,
        "body": body,
        "created_at": now,
        "updated_at": now,
        "scheduled_send_at": 0,
        "effects": [
            {
                "type": "CreateDocument",
                "params": {
                    "user_id": user_id,
                    "collection": "drafts",
                    "data": {
                        "draft_id": format!("draft_{}", now),
                        "to": to_array,
                        "cc": cc_json.as_array().unwrap_or(&vec![]),
                        "bcc": bcc_json.as_array().unwrap_or(&vec![]),
                        "subject": subject,
                        "body": body,
                        "created_at": now,
                        "updated_at": now,
                        "scheduled_send_at": 0
                    }
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: send_email
/// Purpose: Send a draft email
/// Permission Required: send
#[no_mangle]
pub extern "C" fn send_email(
    user_id: *const u8,
    user_id_len: usize,
    draft_id: *const u8,
    draft_id_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let draft_id = match parse_string_from_raw(draft_id, draft_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Send) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Platform Effects:
    // 1. QueryDocument(collection="drafts", document_id=draft_id) - validate draft exists
    // 2. SendEmail(draft_id, recipients...) - platform SMTP service
    // 3. MoveDocument(draft_id, from="drafts", to="sent") - archive draft
    // 4. CreateDocument(collection="emails", data={...}) - store sent email

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let response_data = json!({
        "email_id": format!("email_{}", draft_id),
        "sent_at": now,
        "effects": [
            {
                "type": "QueryDocument",
                "params": {
                    "user_id": user_id,
                    "collection": "drafts",
                    "document_id": draft_id
                }
            },
            {
                "type": "SendEmail",
                "params": {
                    "draft_id": draft_id,
                    "user_id": user_id
                }
            },
            {
                "type": "MoveDocument",
                "params": {
                    "user_id": user_id,
                    "from_collection": "drafts",
                    "to_collection": "emails",
                    "document_id": draft_id,
                    "updates": { "sent_at": now, "folder_id": "sent" }
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: list_folders
/// Purpose: Enumerate all folders for the user
/// Permission Required: read
#[no_mangle]
pub extern "C" fn list_folders(
    user_id: *const u8,
    user_id_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Read) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Platform Effect: QueryFolders(user_id)
    let response_data = json!({
        "folders": [
            {
                "folder_id": "inbox",
                "name": "Inbox",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            },
            {
                "folder_id": "drafts",
                "name": "Drafts",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            },
            {
                "folder_id": "sent",
                "name": "Sent",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            },
            {
                "folder_id": "trash",
                "name": "Trash",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            },
            {
                "folder_id": "junk",
                "name": "Junk",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            },
            {
                "folder_id": "archive",
                "name": "Archive",
                "type": "system",
                "unread_count": 0,
                "total_count": 0
            }
        ],
        "effects": [
            {
                "type": "QueryFolders",
                "params": {
                    "user_id": user_id
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: move_email
/// Purpose: Move an email from one folder to another
/// Permission Required: draft (for DRAFTS) or send (for others)
#[no_mangle]
pub extern "C" fn move_email(
    user_id: *const u8,
    user_id_len: usize,
    email_id: *const u8,
    email_id_len: usize,
    target_folder_id: *const u8,
    target_folder_id_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let email_id = match parse_string_from_raw(email_id, email_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let target_folder_id = match parse_string_from_raw(target_folder_id, target_folder_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Determine required permission based on target folder
    let required_perm = if target_folder_id == "drafts" {
        Permission::Draft
    } else {
        Permission::Send
    };

    if let Err(e) = check_permission(perm, required_perm) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Platform Effect: UpdateDocument(user_id, collection="emails", document_id=email_id, updates={folder_id: target_folder_id})
    let _response_data = json!({
        "effects": [
            {
                "type": "UpdateDocument",
                "params": {
                    "user_id": user_id,
                    "collection": "emails",
                    "document_id": email_id,
                    "updates": {
                        "folder_id": target_folder_id
                    }
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: json!(null),
    });
    return_json_response(&result)
}

/// Function: search_emails
/// Purpose: Full-text search across user's emails
/// Permission Required: read
#[no_mangle]
pub extern "C" fn search_emails(
    user_id: *const u8,
    user_id_len: usize,
    query: *const u8,
    query_len: usize,
    limit: u32,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let query = match parse_string_from_raw(query, query_len) {
        Ok(q) => q,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let perm_str = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };
    let perm = Permission::from_str(&perm_str);

    // Check permission
    if let Err(e) = check_permission(perm, Permission::Read) {
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e }));
    }

    // Validate limit
    let limit = std::cmp::min(limit, 500);
    if limit == 0 {
        let err = ErrorResponse {
            code: "InvalidInput".to_string(),
            message: "Limit must be between 1 and 500".to_string(),
        };
        return return_json_response(&AppResult::<Value>::Err(ErrResponse { err }));
    }

    // Platform Effect: SearchDocuments(user_id, collection="emails", query, limit)
    let response_data = json!({
        "emails": [],
        "matched_count": 0,
        "has_more": false,
        "effects": [
            {
                "type": "SearchDocuments",
                "params": {
                    "user_id": user_id,
                    "collection": "emails",
                    "query": query,
                    "limit": limit
                }
            }
        ]
    });

    let result = AppResult::Ok(OkResponse {
        ok: response_data,
    });
    return_json_response(&result)
}

/// Function: get_agent_permissions
/// Purpose: Query the current agent's permission level
/// Permission Required: None (always callable)
#[no_mangle]
pub extern "C" fn get_agent_permissions(
    user_id: *const u8,
    user_id_len: usize,
    permission: *const u8,
    permission_len: usize,
) -> *mut u8 {
    // Parse inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(id) => id,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let permission = match parse_string_from_raw(permission, permission_len) {
        Ok(p) => p,
        Err(e) => return return_json_response(&AppResult::<Value>::Err(ErrResponse { err: e })),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Return the permission set (no platform effect required, just introspection)
    let response_data = json!({
        "permission_level": permission,
        "user_id": user_id,
        "agent_id": "agent_requesting",
        "granted_at": now,
        "expires_at": 0
    });

    return_json_response(&response_data)
}
