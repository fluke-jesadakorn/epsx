use chrono::Utc;
use diesel::prelude::*;
use diesel::dsl::count_star;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::infrastructure::models::chat::*;
use crate::prelude::TlsPool;
use crate::schemas::primary::{chat_topics, chat_conversations, chat_messages};

pub struct ChatRepository;

impl ChatRepository {
    // ========================================================================
    // TOPICS
    // ========================================================================

    pub async fn list_topics(pool: &TlsPool) -> Result<Vec<ChatTopicDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_topics::table
            .filter(chat_topics::is_active.eq(true))
            .order(chat_topics::sort_order.asc())
            .load::<ChatTopicDb>(&mut conn)
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
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        let conv = NewConversation {
            topic_id,
            wallet_address: wallet.to_string(),
            subject: subject.to_string(),
            status: "open".to_string(),
        };

        let created: ChatConversationDb = diesel::insert_into(chat_conversations::table)
            .values(&conv)
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        // Insert first message
        let msg = NewMessage {
            conversation_id: created.id,
            sender_type: "user".to_string(),
            sender_address: Some(wallet.to_string()),
            content: first_message.to_string(),
        };

        diesel::insert_into(chat_messages::table)
            .values(&msg)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        // Update unread_agent
        diesel::update(chat_conversations::table.find(created.id))
            .set(chat_conversations::unread_agent.eq(1))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(created)
    }

    pub async fn list_user_conversations(
        pool: &TlsPool,
        wallet: &str,
    ) -> Result<Vec<ChatConversationDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_conversations::table
            .filter(chat_conversations::wallet_address.eq(wallet))
            .order(chat_conversations::last_message_at.desc())
            .load::<ChatConversationDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_all_conversations(
        pool: &TlsPool,
        status_filter: Option<&str>,
        topic_filter: Option<Uuid>,
        agent_filter: Option<&str>,
    ) -> Result<Vec<ChatConversationDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        let mut query = chat_conversations::table.into_boxed();

        if let Some(status) = status_filter {
            query = query.filter(chat_conversations::status.eq(status));
        }
        if let Some(topic) = topic_filter {
            query = query.filter(chat_conversations::topic_id.eq(topic));
        }
        if let Some(agent) = agent_filter {
            query = query.filter(chat_conversations::assigned_agent.eq(agent));
        }

        query
            .order(chat_conversations::last_message_at.desc())
            .load::<ChatConversationDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_conversation(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<Option<ChatConversationDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_conversations::table
            .find(conv_id)
            .first::<ChatConversationDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn update_status(
        pool: &TlsPool,
        conv_id: Uuid,
        status: &str,
    ) -> Result<ChatConversationDb, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        diesel::update(chat_conversations::table.find(conv_id))
            .set((
                chat_conversations::status.eq(status),
                chat_conversations::updated_at.eq(Utc::now()),
            ))
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn assign_agent(
        pool: &TlsPool,
        conv_id: Uuid,
        agent: Option<&str>,
    ) -> Result<ChatConversationDb, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        let new_status = if agent.is_some() { "in_progress" } else { "open" };

        diesel::update(chat_conversations::table.find(conv_id))
            .set((
                chat_conversations::assigned_agent.eq(agent),
                chat_conversations::status.eq(new_status),
                chat_conversations::updated_at.eq(Utc::now()),
            ))
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    // ========================================================================
    // MESSAGES
    // ========================================================================

    pub async fn list_messages(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<Vec<ChatMessageDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_messages::table
            .filter(chat_messages::conversation_id.eq(conv_id))
            .order(chat_messages::created_at.asc())
            .load::<ChatMessageDb>(&mut conn)
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
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        let msg = NewMessage {
            conversation_id: conv_id,
            sender_type: sender_type.to_string(),
            sender_address: sender_address.map(|s| s.to_string()),
            content: content.to_string(),
        };

        let created: ChatMessageDb = diesel::insert_into(chat_messages::table)
            .values(&msg)
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        // Update conversation timestamps and unread counts
        let now = Utc::now();
        match sender_type {
            "user" => {
                diesel::update(chat_conversations::table.find(conv_id))
                    .set((
                        chat_conversations::last_message_at.eq(now),
                        chat_conversations::updated_at.eq(now),
                        chat_conversations::unread_agent.eq(chat_conversations::unread_agent + 1),
                    ))
                    .execute(&mut conn)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            "agent" | "system" | "ai" => {
                diesel::update(chat_conversations::table.find(conv_id))
                    .set((
                        chat_conversations::last_message_at.eq(now),
                        chat_conversations::updated_at.eq(now),
                        chat_conversations::unread_user.eq(chat_conversations::unread_user + 1),
                    ))
                    .execute(&mut conn)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        Ok(created)
    }

    pub async fn mark_read_by_user(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<(), String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        // Mark all agent/system messages as read
        diesel::update(
            chat_messages::table
                .filter(chat_messages::conversation_id.eq(conv_id))
                .filter(chat_messages::sender_type.ne("user"))
                .filter(chat_messages::is_read.eq(false)),
        )
        .set(chat_messages::is_read.eq(true))
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

        // Reset user unread count
        diesel::update(chat_conversations::table.find(conv_id))
            .set(chat_conversations::unread_user.eq(0))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn mark_read_by_agent(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<(), String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        diesel::update(
            chat_messages::table
                .filter(chat_messages::conversation_id.eq(conv_id))
                .filter(chat_messages::sender_type.eq("user"))
                .filter(chat_messages::is_read.eq(false)),
        )
        .set(chat_messages::is_read.eq(true))
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

        diesel::update(chat_conversations::table.find(conv_id))
            .set(chat_conversations::unread_agent.eq(0))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_unread_count(
        pool: &TlsPool,
        wallet: &str,
    ) -> Result<i64, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        let total: i64 = chat_conversations::table
            .filter(chat_conversations::wallet_address.eq(wallet))
            .select(diesel::dsl::sum(chat_conversations::unread_user))
            .first::<Option<i64>>(&mut conn)
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or(0);

        Ok(total)
    }

    pub async fn get_last_message(
        pool: &TlsPool,
        conv_id: Uuid,
    ) -> Result<Option<String>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_messages::table
            .filter(chat_messages::conversation_id.eq(conv_id))
            .order(chat_messages::created_at.desc())
            .select(chat_messages::content)
            .first::<String>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }

    // ========================================================================
    // STATS (Admin)
    // ========================================================================

    pub async fn get_stats(pool: &TlsPool) -> Result<ChatStatsResponse, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;

        let total_open: i64 = chat_conversations::table
            .filter(chat_conversations::status.eq("open"))
            .select(count_star())
            .first(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        let total_in_progress: i64 = chat_conversations::table
            .filter(chat_conversations::status.eq("in_progress"))
            .select(count_star())
            .first(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        let total_resolved: i64 = chat_conversations::table
            .filter(chat_conversations::status.eq("resolved"))
            .select(count_star())
            .first(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        let total_unassigned: i64 = chat_conversations::table
            .filter(chat_conversations::assigned_agent.is_null())
            .filter(chat_conversations::status.ne("closed"))
            .select(count_star())
            .first(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(ChatStatsResponse {
            total_open,
            total_in_progress,
            total_resolved,
            total_unassigned,
        })
    }

    pub async fn get_topic(
        pool: &TlsPool,
        topic_id: Uuid,
    ) -> Result<Option<ChatTopicDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        chat_topics::table
            .find(topic_id)
            .first::<ChatTopicDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }
}
