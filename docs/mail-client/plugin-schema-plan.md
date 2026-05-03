# Mail Client App — plugin.json Schema Synchronization Plan

> **Purpose:** Map each Rust function contract to the corresponding `plugin.json` Execution Interface and Semantic Interface definitions.
>
> **Process:** For each function, this document specifies the exact JSON Schema that must appear in `plugin.json`. Any deviation between the Rust signature and the schema is a critical bug.

---

## File Structure

The `plugin.json` will have three sections:

```json
{
  "name": "MailClient",
  "version": "1.0.0",
  "description": "AI-native email client for Synapp platform",
  "executionInterface": { ... },
  "semanticInterface": { ... },
  "capabilities": { ... }
}
```

---

## Section 1: Execution Interface

The `executionInterface` describes the Wasm module's raw exports.

### Template Structure
```json
{
  "executionInterface": {
    "wasmTarget": "wasm32-wasi",
    "wasmPath": "target/wasm32-wasi/release/mail_client.wasm",
    "exports": [
      {
        "name": "<function_name>",
        "description": "<purpose>",
        "inputSchema": { /* JSON Schema */ },
        "outputSchema": { /* JSON Schema */ }
      }
    ]
  }
}
```

---

## Function-by-Function Schema Definitions

### 1. read_emails

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "mailbox": {
      "type": "string",
      "description": "Folder identifier (INBOX, SENT, DRAFTS, TRASH, JUNK, ARCHIVE, or custom folder_id)",
      "examples": ["INBOX", "SENT", "custom_folder_1"]
    },
    "offset": {
      "type": "integer",
      "minimum": 0,
      "description": "Pagination offset (0-indexed)"
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 500,
      "description": "Max emails to return"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "mailbox", "offset", "limit", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": {
          "type": "object",
          "properties": {
            "emails": {
              "type": "array",
              "items": { "$ref": "#/definitions/Email" },
              "description": "List of emails"
            },
            "total_count": {
              "type": "integer",
              "description": "Total emails in mailbox (for pagination UI)"
            },
            "has_more": {
              "type": "boolean",
              "description": "Whether more emails exist beyond limit"
            }
          },
          "required": ["emails", "total_count", "has_more"]
        }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 2. get_email

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "email_id": {
      "type": "string",
      "description": "Unique email identifier"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "email_id", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": { "$ref": "#/definitions/Email" }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 3. draft_email

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "to": {
      "type": "array",
      "items": { "type": "string", "format": "email" },
      "minItems": 1,
      "description": "Primary recipients"
    },
    "cc": {
      "type": "array",
      "items": { "type": "string", "format": "email" },
      "description": "Carbon copy recipients (optional)"
    },
    "bcc": {
      "type": "array",
      "items": { "type": "string", "format": "email" },
      "description": "Blind carbon copy recipients (optional)"
    },
    "subject": {
      "type": "string",
      "maxLength": 1000,
      "description": "Email subject line"
    },
    "body": {
      "type": "string",
      "description": "Email body (HTML or plain text)"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "to", "subject", "body", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": { "$ref": "#/definitions/Draft" }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 4. send_email

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "draft_id": {
      "type": "string",
      "description": "Draft identifier from draft_email()"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "draft_id", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": {
          "type": "object",
          "properties": {
            "email_id": {
              "type": "string",
              "description": "ID of sent email"
            },
            "sent_at": {
              "type": "integer",
              "description": "Unix timestamp when sent"
            }
          },
          "required": ["email_id", "sent_at"]
        }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 5. list_folders

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": {
          "type": "object",
          "properties": {
            "folders": {
              "type": "array",
              "items": { "$ref": "#/definitions/Folder" }
            }
          },
          "required": ["folders"]
        }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 6. move_email

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "email_id": {
      "type": "string",
      "description": "Email to move"
    },
    "target_folder_id": {
      "type": "string",
      "description": "Destination folder identifier"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "email_id", "target_folder_id", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": {
          "type": "null",
          "description": "Success (no data returned)"
        }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 7. search_emails

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "query": {
      "type": "string",
      "description": "Search expression (free-form text or operators like from:, to:, subject:, before:, after:, is:, has:)"
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 500,
      "description": "Max results to return"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Agent permission level"
    }
  },
  "required": ["user_id", "query", "limit", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "ok": {
          "type": "object",
          "properties": {
            "emails": {
              "type": "array",
              "items": { "$ref": "#/definitions/Email" }
            },
            "matched_count": {
              "type": "integer",
              "description": "Total matches (may exceed limit)"
            },
            "has_more": {
              "type": "boolean"
            }
          },
          "required": ["emails", "matched_count", "has_more"]
        }
      },
      "required": ["ok"]
    },
    {
      "type": "object",
      "properties": {
        "err": { "$ref": "#/definitions/Error" }
      },
      "required": ["err"]
    }
  ]
}
```

---

### 8. get_agent_permissions

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "Authenticated user identifier"
    },
    "permission": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"],
      "description": "Current agent permission level (provided by host)"
    }
  },
  "required": ["user_id", "permission"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "permission_level": {
      "type": "string",
      "enum": ["none", "read", "draft", "send"]
    },
    "user_id": {
      "type": "string"
    },
    "agent_id": {
      "type": "string",
      "description": "Requesting agent identifier"
    },
    "granted_at": {
      "type": "integer",
      "description": "Unix timestamp"
    },
    "expires_at": {
      "type": "integer",
      "description": "Unix timestamp (0 = never expires)"
    }
  },
  "required": ["permission_level", "user_id", "agent_id", "granted_at", "expires_at"]
}
```

---

## Section 2: Shared Type Definitions

All types below should be defined once in a `definitions` section and referenced via `$ref`:

### Email (Shared Definition)
```json
{
  "definitions": {
    "Email": {
      "type": "object",
      "properties": {
        "id": {
          "type": "string",
          "description": "Unique email identifier"
        },
        "message_id": {
          "type": "string",
          "description": "RFC 5322 Message-ID"
        },
        "user_id": {
          "type": "string",
          "description": "Owner user ID"
        },
        "from": {
          "type": "string",
          "format": "email",
          "description": "Sender email address"
        },
        "to": {
          "type": "array",
          "items": { "type": "string", "format": "email" },
          "description": "Recipient addresses"
        },
        "cc": {
          "type": "array",
          "items": { "type": "string", "format": "email" },
          "description": "Carbon copy addresses"
        },
        "bcc": {
          "type": "array",
          "items": { "type": "string", "format": "email" },
          "description": "Blind carbon copy addresses"
        },
        "subject": {
          "type": "string",
          "description": "Email subject"
        },
        "body": {
          "type": "string",
          "description": "Email body (HTML or plain text)"
        },
        "received_at": {
          "type": "integer",
          "description": "Unix timestamp (seconds)"
        },
        "sent_at": {
          "type": "integer",
          "description": "Unix timestamp (0 if draft)"
        },
        "is_read": {
          "type": "boolean",
          "description": "Read/unread status"
        },
        "folder_id": {
          "type": "string",
          "description": "Current folder"
        },
        "thread_id": {
          "type": "string",
          "description": "Thread grouping identifier"
        },
        "has_attachments": {
          "type": "boolean"
        },
        "importance": {
          "type": "string",
          "enum": ["low", "normal", "high"]
        }
      },
      "required": [
        "id", "message_id", "user_id", "from", "to", "subject", "body",
        "received_at", "sent_at", "is_read", "folder_id", "thread_id", "has_attachments", "importance"
      ]
    }
  }
}
```

### Draft (Shared Definition)
```json
{
  "definitions": {
    "Draft": {
      "type": "object",
      "properties": {
        "draft_id": {
          "type": "string",
          "description": "Unique draft identifier"
        },
        "to": {
          "type": "array",
          "items": { "type": "string", "format": "email" }
        },
        "cc": {
          "type": "array",
          "items": { "type": "string", "format": "email" }
        },
        "bcc": {
          "type": "array",
          "items": { "type": "string", "format": "email" }
        },
        "subject": {
          "type": "string"
        },
        "body": {
          "type": "string"
        },
        "created_at": {
          "type": "integer",
          "description": "Unix timestamp"
        },
        "updated_at": {
          "type": "integer",
          "description": "Unix timestamp"
        },
        "scheduled_send_at": {
          "type": "integer",
          "description": "Unix timestamp (0 = send immediately)"
        }
      },
      "required": ["draft_id", "to", "cc", "bcc", "subject", "body", "created_at", "updated_at", "scheduled_send_at"]
    }
  }
}
```

### Folder (Shared Definition)
```json
{
  "definitions": {
    "Folder": {
      "type": "object",
      "properties": {
        "folder_id": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "type": {
          "type": "string",
          "enum": ["system", "custom"]
        },
        "unread_count": {
          "type": "integer"
        },
        "total_count": {
          "type": "integer"
        }
      },
      "required": ["folder_id", "name", "type", "unread_count", "total_count"]
    }
  }
}
```

### Error (Shared Definition)
```json
{
  "definitions": {
    "Error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "enum": [
            "PermissionDenied",
            "EmailNotFound",
            "DraftNotFound",
            "FolderNotFound",
            "InvalidRecipient",
            "InvalidInput",
            "DraftAlreadySent",
            "TooManyRecipients",
            "InternalError"
          ]
        },
        "message": {
          "type": "string",
          "description": "Human-readable error description"
        }
      },
      "required": ["code", "message"]
    }
  }
}
```

---

## Section 3: Semantic Interface

The `semanticInterface` describes how AI agents see and interact with the app as tools.

### Template
```json
{
  "semanticInterface": {
    "tools": [
      {
        "name": "<function_name>",
        "description": "<high-level description for AI>",
        "permissionRequired": "read|draft|send|none",
        "category": "query|mutation",
        "riskLevel": "low|medium|high"
      }
    ]
  }
}
```

### Tool Definitions

```json
{
  "semanticInterface": {
    "tools": [
      {
        "name": "read_emails",
        "description": "Retrieve emails from a mailbox (INBOX, SENT, DRAFTS, etc.). Returns paginated results sorted newest-first.",
        "permissionRequired": "read",
        "category": "query",
        "riskLevel": "low"
      },
      {
        "name": "get_email",
        "description": "Fetch a single email by ID, including full body and metadata. Marks email as read.",
        "permissionRequired": "read",
        "category": "query",
        "riskLevel": "low"
      },
      {
        "name": "draft_email",
        "description": "Create an email draft with recipients, subject, and body. Does not send. Returns draft_id for later sending or human review.",
        "permissionRequired": "draft",
        "category": "mutation",
        "riskLevel": "medium"
      },
      {
        "name": "send_email",
        "description": "Send a draft email to all recipients. Moves draft from DRAFTS to SENT folder. Requires explicit 'send' permission.",
        "permissionRequired": "send",
        "category": "mutation",
        "riskLevel": "high"
      },
      {
        "name": "list_folders",
        "description": "List all mailbox folders (Inbox, Sent, Drafts, etc.). Includes unread and total counts.",
        "permissionRequired": "read",
        "category": "query",
        "riskLevel": "low"
      },
      {
        "name": "move_email",
        "description": "Move an email to another folder (e.g., Archive, Trash). Soft-delete moves to Trash (purged after 30 days).",
        "permissionRequired": "draft",
        "category": "mutation",
        "riskLevel": "medium"
      },
      {
        "name": "search_emails",
        "description": "Full-text search emails. Supports operators: from:, to:, subject:, before:, after:, is:read, is:unread, has:attachment.",
        "permissionRequired": "read",
        "category": "query",
        "riskLevel": "low"
      },
      {
        "name": "get_agent_permissions",
        "description": "Introspect current agent permissions (read, draft, send). Returns permission level and expiration time.",
        "permissionRequired": "none",
        "category": "query",
        "riskLevel": "low"
      }
    ]
  }
}
```

---

## Section 4: Capabilities

Optional metadata about the app's capabilities and constraints.

```json
{
  "capabilities": {
    "maxRecipients": 100,
    "maxBodySize": 52428800,
    "maxSubjectLength": 1000,
    "folderLimit": 500,
    "draftExpirationDays": 7,
    "trashRetentionDays": 30,
    "searchOperators": ["from:", "to:", "subject:", "before:", "after:", "is:", "has:"],
    "supportedBodyFormats": ["text/plain", "text/html"],
    "attachmentSupport": false
  }
}
```

---

## Complete plugin.json Template

```json
{
  "name": "MailClient",
  "version": "1.0.0",
  "description": "AI-native email management app for Synapp platform. Supports autonomous agents with granular permission controls (read, draft, send).",
  "executionInterface": {
    "wasmTarget": "wasm32-wasi",
    "wasmPath": "target/wasm32-wasi/release/mail_client.wasm",
    "exports": [
      {
        "name": "read_emails",
        "description": "Fetch paginated emails from a mailbox.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "get_email",
        "description": "Fetch a single email by ID.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "draft_email",
        "description": "Create an email draft.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "send_email",
        "description": "Send a draft email.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "list_folders",
        "description": "List all mailbox folders.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "move_email",
        "description": "Move an email to another folder.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "search_emails",
        "description": "Full-text search emails.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      },
      {
        "name": "get_agent_permissions",
        "description": "Query current agent permission level.",
        "inputSchema": { /* see above */ },
        "outputSchema": { /* see above */ }
      }
    ]
  },
  "semanticInterface": {
    "tools": [ /* see above */ ]
  },
  "capabilities": {
    "maxRecipients": 100,
    "maxBodySize": 52428800,
    "maxSubjectLength": 1000,
    "folderLimit": 500,
    "draftExpirationDays": 7,
    "trashRetentionDays": 30,
    "searchOperators": ["from:", "to:", "subject:", "before:", "after:", "is:", "has:"],
    "supportedBodyFormats": ["text/plain", "text/html"],
    "attachmentSupport": false
  }
}
```

---

## Synchronization Checklist

Before marking `plugin.json` complete, verify:

- [ ] Each Rust `#[no_mangle] pub extern "C" fn` has a corresponding `exports` entry
- [ ] Function names in `exports` match Rust signatures exactly
- [ ] All input parameters appear in `inputSchema.properties`
- [ ] All input parameters are listed in `inputSchema.required`
- [ ] Output `type` matches the Rust return type (always JSON object with `ok` or `err`)
- [ ] All error codes in `Error` definition match those thrown in contracts.md
- [ ] All permissions in semantic interface tools are accurate (read, draft, send, none)
- [ ] No Rust function is missing from `semanticInterface.tools`
- [ ] Email, Draft, Folder, Error definitions are complete and consistent
- [ ] Plugin name, version, and description are accurate

---

## Future Maintenance

Whenever **contracts.md** is updated:

1. Identify which function(s) changed
2. Update the corresponding `exports` entry in `executionInterface`
3. Update the corresponding `tools` entry in `semanticInterface`
4. If parameters/return types changed, update shared type definitions
5. Run validation to ensure no mismatches
6. Commit both files together (contracts.md + plugin.json)

This ensures the Dual-Interface contract remains synchronized and prevents AI broker confusion or runtime errors.
