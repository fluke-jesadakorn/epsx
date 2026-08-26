/**
 * Integration Tests for Notification System
 *
 * Tests notification handlers, SSE connections, and database operations.
 */
#[cfg(test)]
mod notification_tests {
    use crate::__test__::test_utils::*;
    use crate::infrastructure::database::diesel_connection_manager::TlsPool;
    use crate::infrastructure::database::get_diesel_pool;
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_test_notification(
        pool: &TlsPool,
        wallet_address: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO wallet_notifications
            (id, wallet_address, notification_type, title, message, priority, timestamp, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(wallet_address)
        .bind("system")
        .bind("Test Notification")
        .bind("This is a test notification")
        .bind("normal")
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(id)
    }

    async fn cleanup_test_notifications(pool: &TlsPool) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM wallet_notifications WHERE title = 'Test Notification'")
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_notification_creation_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let _test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        // Create test notification
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(pool, wallet).await?;

        #[derive(sqlx::FromRow)]
        struct NotificationExists {
            exists: Option<bool>,
        }

        let result: NotificationExists = sqlx::query_as::<_, NotificationExists>(
            "SELECT EXISTS(SELECT 1 FROM wallet_notifications WHERE id = $1) as exists",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        assert!(result.exists.unwrap_or(false));

        // Clean up
        cleanup_test_notifications(pool).await?;

        // Verify cleanup worked
        let result: NotificationExists = sqlx::query_as::<_, NotificationExists>(
            "SELECT EXISTS(SELECT 1 FROM wallet_notifications WHERE id = $1) as exists",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        assert!(!result.exists.unwrap_or(true));

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_multiple_notifications_for_same_wallet() -> Result<(), Box<dyn std::error::Error>>
    {
        let _test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Create multiple notifications
        let _id1 = setup_test_notification(pool, wallet).await?;
        let _id2 = setup_test_notification(pool, wallet).await?;
        let _id3 = setup_test_notification(pool, wallet).await?;

        #[derive(sqlx::FromRow)]
        struct CountResult {
            count: i64,
        }

        let result: CountResult = sqlx::query_as::<_, CountResult>(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE title = 'Test Notification'",
        )
        .fetch_one(pool)
        .await?;

        assert!(result.count >= 3);

        // Clean up
        cleanup_test_notifications(pool).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_notification_different_wallets() -> Result<(), Box<dyn std::error::Error>> {
        let _test_db = setup_test_database().await?;
        let pool = get_diesel_pool().await?;

        let wallet1 = "0x1234567890abcdef1234567890abcdef12345678";
        let wallet2 = "0x9876543210fedcba9876543210fedcba98765432";

        // Create notifications for different wallets
        let _id1 = setup_test_notification(pool, wallet1).await?;
        let _id2 = setup_test_notification(pool, wallet2).await?;

        #[derive(sqlx::FromRow)]
        struct WalletResult {
            _wallet_address: String,
        }

        let results: Vec<WalletResult> = sqlx::query_as::<_, WalletResult>(
            r#"
            SELECT DISTINCT wallet_address as _wallet_address
            FROM wallet_notifications
            WHERE title = 'Test Notification'
            "#,
        )
        .fetch_all(pool)
        .await?;

        assert_eq!(results.len(), 2);

        // Clean up
        cleanup_test_notifications(pool).await?;

        Ok(())
    }
}
