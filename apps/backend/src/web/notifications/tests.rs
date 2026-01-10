/**
 * Integration Tests for Notification System
 *
 * Tests notification handlers, SSE connections, and database operations.
 * Uses Diesel for database operations instead of SQLx.
 */#[cfg(test)]
mod notification_tests {
    use crate::__test__::test_utils::*;
    use crate::infrastructure::database::get_diesel_pool;
    use chrono::Utc;
    use uuid::Uuid;
    use diesel::prelude::*;
    use diesel_async::{RunQueryDsl, pooled_connection::deadpool::Pool, AsyncPgConnection};

    async fn setup_test_notification(
        pool: &Pool<AsyncPgConnection>,
        wallet_address: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();
        let mut conn = pool.get().await?;

        // Use raw SQL for insertion to match original schema
        diesel_async::RunQueryDsl::execute(diesel::sql_query(
            r#"
            INSERT INTO wallet_notifications
            (id, wallet_address, notification_type, title, message, priority, timestamp, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .bind::<diesel::sql_types::Text, _>("system")
        .bind::<diesel::sql_types::Text, _>("Test Notification")
        .bind::<diesel::sql_types::Text, _>("This is a test notification")
        .bind::<diesel::sql_types::Text, _>("normal")
        .bind::<diesel::sql_types::Timestamptz, _>(Utc::now()), &mut conn)
        .await?;

        Ok(id)
    }

    async fn cleanup_test_notifications(
        pool: &Pool<AsyncPgConnection>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = pool.get().await?;
        diesel_async::RunQueryDsl::execute(diesel::sql_query("DELETE FROM wallet_notifications WHERE title = 'Test Notification'"), &mut conn)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_notification_creation_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        // Create test notification
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet).await?;

        // Verify notification was created
        #[derive(QueryableByName)]
        struct NotificationExists {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            exists: bool,
        }

        let mut conn = pool.get().await?;
        let result: NotificationExists = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM wallet_notifications WHERE id = $1) as exists"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result(&mut conn)
        .await?;

        assert!(result.exists);

        // Clean up
        cleanup_test_notifications(&pool).await?;

        // Verify cleanup worked
        let result: NotificationExists = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM wallet_notifications WHERE id = $1) as exists"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result(&mut conn)
        .await?;

        assert!(!result.exists);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_notifications_for_same_wallet() -> Result<(), Box<dyn std::error::Error>> {
        let test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Create multiple notifications
        let _id1 = setup_test_notification(&pool, wallet).await?;
        let _id2 = setup_test_notification(&pool, wallet).await?;
        let _id3 = setup_test_notification(&pool, wallet).await?;

        // Verify all exist using a tuple
        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let mut conn = pool.get().await?;
        let result: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE title = 'Test Notification'"
        )
        .get_result(&mut conn)
        .await?;

        assert!(result.count >= 3);

        // Clean up
        cleanup_test_notifications(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_notification_different_wallets() -> Result<(), Box<dyn std::error::Error>> {
        let test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        let wallet1 = "0x1234567890abcdef1234567890abcdef12345678";
        let wallet2 = "0x9876543210fedcba9876543210fedcba98765432";

        // Create notifications for different wallets
        let _id1 = setup_test_notification(&pool, wallet1).await?;
        let _id2 = setup_test_notification(&pool, wallet2).await?;

        // Verify they exist
        #[derive(QueryableByName)]
        struct WalletResult {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
        }

        let mut conn = pool.get().await?;
        let results: Vec<WalletResult> = diesel::sql_query(
            r#"
            SELECT DISTINCT wallet_address
            FROM wallet_notifications
            WHERE title = 'Test Notification'
            "#
        )
        .load(&mut conn)
        .await?;

        assert_eq!(results.len(), 2);

        // Clean up
        cleanup_test_notifications(&pool).await?;

        Ok(())
    }
}