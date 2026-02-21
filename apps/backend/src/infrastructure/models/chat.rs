use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::schemas::primary::{chat_topics, chat_conversations, chat_messages};

// ============================================================================
// TOPIC MODELS
// ============================================================================

#[derive(Debug, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = chat_topics)]
pub struct ChatTopicDb {
    pub id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// CONVERSATION MODELS
// ============================================================================

#[derive(Debug, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = chat_conversations)]
pub struct ChatConversationDb {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub wallet_address: String,
    pub subject: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub last_message_at: DateTime<Utc>,
    pub unread_user: i32,
    pub unread_agent: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = chat_conversations)]
pub struct NewConversation {
    pub topic_id: Uuid,
    pub wallet_address: String,
    pub subject: String,
    pub status: String,
}

// ============================================================================
// MESSAGE MODELS
// ============================================================================

#[derive(Debug, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = chat_messages)]
pub struct ChatMessageDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_type: String,
    pub sender_address: Option<String>,
    pub content: String,
    pub is_read: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = chat_messages)]
pub struct NewMessage {
    pub conversation_id: Uuid,
    pub sender_type: String,
    pub sender_address: Option<String>,
    pub content: String,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateConversationRequest {
    pub topic_id: Uuid,
    pub subject: String,
    pub message: String,
    #[serde(default)]
    pub turnstile_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignAgentRequest {
    pub agent_address: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationWithTopic {
    #[serde(flatten)]
    pub conversation: ChatConversationDb,
    pub topic_name: String,
    pub topic_label: String,
    pub last_message: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatStatsResponse {
    pub total_open: i64,
    pub total_in_progress: i64,
    pub total_resolved: i64,
    pub total_unassigned: i64,
}
