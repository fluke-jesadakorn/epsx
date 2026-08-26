// Chat Repository
// Handles topics, conversations, and messages
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::models::chat::*;
use crate::prelude::TlsPool;

pub struct ChatRepository;

impl ChatRepository {
    // ========================================================================
    // TOPICS
    // ========================================================================

    pub async fn list_topics(pool: &TlsPool) -> Result<Vec<ChatTopicDb>, String> {
        sqlx::query_as(
            "SELECT id, name, slug, description, icon, color, is_active, sort_order, created_at, updated_at \
             FROM chat_topics WHERE is_active = TRUE ORDER BY sort_order ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ========================================================================
    // CONVERSATIONS
    // ========================================================================

    pub async fn create_conversation(
        pool: &TlsPool,
        topic_id: Uuid,
        wallet: &str,
        subject: &str,
        first_message: &str,
    ) -> Result<ChatConversationDb, String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        // Insert conversation
        let created: ChatConversationDb = sqlx::query_as(
            r#"
            INSERT INTO chat_conversations (
                topic_id, wallet_address, subject, status
            ) VALUES ($1, $2, $3, 'open')
            RETURNING id, topic_id, wallet_address, subject, status, assigned_agent,
                      last_message_at, unread_user, unread_agent, created_at, updated_at,
                      closed_at, resolution
            "#,
        )
        .bind(topic_id)
        .bind(wallet)
        .bind(subject)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Insert first message
        sqlx::query(
            r#"
            INSERT INTO chat_messages (
                conversation_id, sender_type, sender_address, content, metadata
            ) VALUES ($1, 'user', $2, $3, 'null'::jsonb)
            "#,
        )
        .bind(created.id)
        .bind(wallet)
        .bind(first_message)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Update unread_agent
        sqlx::query(
            "UPDATE chat_conversations SET unread_agent = 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(created.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(created)
    }

    pub async fn list_user_conversations(
        pool: &TlsPool,
        wallet: &str,
    ) -> Result<Vec<ChatConversationDb>, String> {
        sqlx::query_as(
            "SELECT id, topic_id, wallet_address, subject, status, assigned_agent, \
                    last_message_at, unread_user, unread_agent, created_at, updated_at, \
                    closed_at, resolution \
             FROM chat_conversations \
             WHERE wallet_address = $1 \
             ORDER BY last_message_at DESC NULLS LAST",
        )
        .bind(wallet)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn list_all_conversations(
        pool: &TlsPool,
        status_filter: Option<&str>,
        topic_filter: Option<Uuid>,
        agent_filter: Option<&str>,
    ) -> Result<Vec<ChatConversationDb>, String> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, topic_id, wallet_address, subject, status, assigned_agent, \
                    last_message_at, unread_user, unread_agent, created_at, updated_at, \
                    closed_at, resolution \
             FROM chat_conversations WHERE TRUE",
        );
        if let Some(status) = status_filter {
            qb.push(" AND status = ").push_bind(status);
        }
        if let Some(topic) = topic_filter {
            qb.push(" AND topic_id = ").push_bind(topic);
        }
        if let Some(agent) = agent_filter {
            qb.push(" AND assigned_agent = ").push_bind(agent);
        }
        qb.push(" ORDER BY last_message_at DESC NULLS LAST");

        qb.build_query_as()
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_conversation(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<Option<ChatConversationDb>, String> {
        sqlx::query_as(
            "SELECT id, topic_id, wallet_address, subject, status, assigned_agent, \
                    last_message_at, unread_user, unread_agent, created_at, updated_at, \
                    closed_at, resolution \
             FROM chat_conversations WHERE id = $1",
        )
        .bind(conv_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn update_status(
        pool: &TlsPool,
        conv_id: Uuid,
        status: &str,
    ) -> Result<ChatConversationDb, String> {
        let row: ChatConversationDb = sqlx::query_as(
            r#"
            UPDATE chat_conversations
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, topic_id, wallet_address, subject, status, assigned_agent,
                      last_message_at, unread_user, unread_agent, created_at, updated_at,
                      closed_at, resolution
            "#,
        )
        .bind(status)
        .bind(conv_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row)
    }

    pub async fn assign_agent(
        pool: &TlsPool,
        conv_id: Uuid,
        agent: Option<&str>,
    ) -> Result<ChatConversationDb, String> {
        let new_status = if agent.is_some() {
            "in_progress"
        } else {
            "open"
        };

        let row: ChatConversationDb = sqlx::query_as(
            r#"
            UPDATE chat_conversations
            SET assigned_agent = $1, status = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, topic_id, wallet_address, subject, status, assigned_agent,
                      last_message_at, unread_user, unread_agent, created_at, updated_at,
                      closed_at, resolution
            "#,
        )
        .bind(agent)
        .bind(new_status)
        .bind(conv_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row)
    }

    // ========================================================================
    // MESSAGES
    // ========================================================================

    pub async fn list_messages(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<Vec<ChatMessageDb>, String> {
        sqlx::query_as(
            "SELECT id, conversation_id, sender_type, sender_address, content, metadata, \
                    is_read, read_at, created_at \
             FROM chat_messages \
             WHERE conversation_id = $1 \
             ORDER BY created_at ASC",
        )
        .bind(conv_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn send_message(
        pool: &TlsPool,
        conv_id: Uuid,
        sender_type: &str,
        sender_address: Option<&str>,
        content: &str,
    ) -> Result<ChatMessageDb, String> {
        Self::send_message_with_meta(pool, conv_id, sender_type, sender_address, content, None)
            .await
    }

    pub async fn send_message_with_meta(
        pool: &TlsPool,
        conv_id: Uuid,
        sender_type: &str,
        sender_address: Option<&str>,
        content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ChatMessageDb, String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let meta = metadata.unwrap_or(serde_json::Value::Null);

        let created: ChatMessageDb = sqlx::query_as(
            r#"
            INSERT INTO chat_messages (
                conversation_id, sender_type, sender_address, content, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id, conversation_id, sender_type, sender_address, content, metadata,
                      is_read, read_at, created_at
            "#,
        )
        .bind(conv_id)
        .bind(sender_type)
        .bind(sender_address)
        .bind(content)
        .bind(&meta)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let now = Utc::now();
        match sender_type {
            "user" => {
                sqlx::query(
                    "UPDATE chat_conversations \
                     SET last_message_at = $1, updated_at = $1, unread_agent = unread_agent + 1 \
                     WHERE id = $2",
                )
                .bind(now)
                .bind(conv_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }
            "agent" | "system" | "ai" => {
                sqlx::query(
                    "UPDATE chat_conversations \
                     SET last_message_at = $1, updated_at = $1, unread_user = unread_user + 1 \
                     WHERE id = $2",
                )
                .bind(now)
                .bind(conv_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(created)
    }

    pub async fn mark_read_by_user(pool: &TlsPool, conv_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE chat_messages SET is_read = TRUE, read_at = NOW() \
             WHERE conversation_id = $1 AND sender_type <> 'user' AND is_read = FALSE",
        )
        .bind(conv_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE chat_conversations SET unread_user = 0 WHERE id = $1")
            .bind(conv_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn mark_read_by_agent(pool: &TlsPool, conv_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE chat_messages SET is_read = TRUE, read_at = NOW() \
             WHERE conversation_id = $1 AND sender_type = 'user' AND is_read = FALSE",
        )
        .bind(conv_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE chat_conversations SET unread_agent = 0 WHERE id = $1")
            .bind(conv_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_unread_count(pool: &TlsPool, wallet: &str) -> Result<i64, String> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT COALESCE(SUM(unread_user), 0)::BIGINT FROM chat_conversations WHERE wallet_address = $1",
        )
        .bind(wallet)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.0.unwrap_or(0))
    }

    pub async fn get_last_message(pool: &TlsPool, conv_id: Uuid) -> Result<Option<String>, String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT content FROM chat_messages WHERE conversation_id = $1 \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(conv_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.map(|(c,)| c))
    }

    // ========================================================================
    // STATS (Admin)
    // ========================================================================

    pub async fn get_stats(pool: &TlsPool) -> Result<ChatStatsResponse, String> {
        let total_open: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chat_conversations WHERE status = 'open'")
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

        let total_in_progress: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chat_conversations WHERE status = 'in_progress'")
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

        let total_resolved: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chat_conversations WHERE status = 'resolved'")
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

        let total_unassigned: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM chat_conversations \
             WHERE assigned_agent IS NULL AND status <> 'closed'",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ChatStatsResponse {
            total_open: total_open.0,
            total_in_progress: total_in_progress.0,
            total_resolved: total_resolved.0,
            total_unassigned: total_unassigned.0,
        })
    }

    pub async fn get_topic(pool: &TlsPool, topic_id: Uuid) -> Result<Option<ChatTopicDb>, String> {
        sqlx::query_as(
            "SELECT id, name, slug, description, icon, color, is_active, sort_order, created_at, updated_at \
             FROM chat_topics WHERE id = $1",
        )
        .bind(topic_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
    }
}

// Re-export type alias for backward compatibility
pub type _ChatPoolArc = Arc<PgPool>;
