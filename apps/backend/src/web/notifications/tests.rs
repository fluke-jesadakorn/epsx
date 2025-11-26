/**
 * Integration Tests for Notification System
 *
 * Tests notification handlers, SSE connections, and database operations.
 * Requires a test database connection.
 */

#[cfg(test)]
mod notification_tests {
    use crate::web::notifications::{
        NotificationType, NotificationPriority, SSENotification,
        fetch_queued_notifications, mark_as_delivered, mark_as_acknowledged,
        get_notification_stats, cleanup_old_notifications,
    };
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

    async fn setup_test_notification(
        pool: &PgPool,
        wallet_address: &str,
        notification_type: &str,
        priority: &str,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO wallet_notifications
            (id, wallet_address, notification_type, title, message, priority, timestamp, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            "#,
            id,
            wallet_address,
            notification_type,
            "Test Notification",
            "This is a test notification",
            priority,
            Utc::now()
        )
        .execute(pool)
        .await?;

        Ok(id)
    }

    async fn cleanup_test_notifications(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM wallet_notifications WHERE title = 'Test Notification'")
            .execute(pool)
            .await?;
        Ok(())
    }

    // ============================================================================
    // FETCH QUEUED NOTIFICATIONS TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_fetch_queued_notifications_for_user(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        // Create test notification
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Fetch notifications
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Verify
        assert!(notifications.len() > 0, "Should fetch at least one notification");
        assert!(notifications.iter().any(|n| n.wallet_address == wallet));

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_fetch_queued_notifications_includes_broadcast(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        // Create broadcast notification
        setup_test_notification(&pool, "all", "system", "normal").await?;

        // Fetch for specific user
        let wallet = "0x9876543210abcdef9876543210abcdef98765432";
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Should include broadcast notifications
        assert!(notifications.iter().any(|n| n.wallet_address == "all"));

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_fetch_queued_notifications_excludes_soft_deleted(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Soft delete the notification
        sqlx::query!("UPDATE wallet_notifications SET deleted_at = NOW() WHERE id = $1", id)
            .execute(&pool)
            .await?;

        // Fetch notifications
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Should not include soft-deleted
        assert!(!notifications.iter().any(|n| n.id == id.to_string()));

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_fetch_queued_notifications_excludes_expired(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Set expiry to past
        let past = Utc::now() - chrono::Duration::hours(1);
        sqlx::query!("UPDATE wallet_notifications SET expires_at = $1 WHERE id = $2", past, id)
            .execute(&pool)
            .await?;

        // Fetch notifications
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Should not include expired
        assert!(!notifications.iter().any(|n| n.id == id.to_string()));

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_fetch_queued_notifications_limits_to_30_days(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Set created_at to 31 days ago
        let old_date = Utc::now() - chrono::Duration::days(31);
        sqlx::query!("UPDATE wallet_notifications SET created_at = $1 WHERE id = $2", old_date, id)
            .execute(&pool)
            .await?;

        // Fetch notifications
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Should not include old notifications
        assert!(!notifications.iter().any(|n| n.id == id.to_string()));

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_fetch_queued_notifications_ordered_by_timestamp_desc(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Create multiple notifications with different timestamps
        let id1 = setup_test_notification(&pool, wallet, "system", "normal").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let id2 = setup_test_notification(&pool, wallet, "system", "normal").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let id3 = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Fetch notifications
        let notifications = fetch_queued_notifications(&pool, wallet).await?;

        // Find our test notifications
        let test_notifs: Vec<_> = notifications.iter()
            .filter(|n| n.id == id1.to_string() || n.id == id2.to_string() || n.id == id3.to_string())
            .collect();

        // Should be ordered newest first
        if test_notifs.len() >= 2 {
            assert!(test_notifs[0].timestamp >= test_notifs[1].timestamp);
        }

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    // ============================================================================
    // MARK AS DELIVERED TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_mark_as_delivered(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Mark as delivered
        mark_as_delivered(&pool, &id.to_string()).await?;

        // Verify
        let result = sqlx::query!("SELECT delivered_at, delivery_attempts FROM wallet_notifications WHERE id = $1", id)
            .fetch_one(&pool)
            .await?;

        assert!(result.delivered_at.is_some(), "Should have delivered_at timestamp");
        assert_eq!(result.delivery_attempts, 1, "Should increment delivery_attempts");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_mark_as_delivered_invalid_uuid(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let result = mark_as_delivered(&pool, "invalid-uuid").await;
        assert!(result.is_err(), "Should return error for invalid UUID");
        Ok(())
    }

    // ============================================================================
    // MARK AS ACKNOWLEDGED TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_mark_as_acknowledged(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Mark as acknowledged
        mark_as_acknowledged(&pool, &id.to_string()).await?;

        // Verify
        let result = sqlx::query!("SELECT acknowledged_at FROM wallet_notifications WHERE id = $1", id)
            .fetch_one(&pool)
            .await?;

        assert!(result.acknowledged_at.is_some(), "Should have acknowledged_at timestamp");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    // ============================================================================
    // NOTIFICATION STATS TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_get_notification_stats(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Create test notifications
        let id1 = setup_test_notification(&pool, wallet, "system", "normal").await?;
        let id2 = setup_test_notification(&pool, wallet, "security", "high").await?;

        // Mark one as delivered
        mark_as_delivered(&pool, &id1.to_string()).await?;

        // Get stats
        let stats = get_notification_stats(&pool).await?;

        assert!(stats.total >= 2, "Should count at least our test notifications");
        assert!(stats.delivered >= 1, "Should count delivered notification");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    // ============================================================================
    // CLEANUP OLD NOTIFICATIONS TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_cleanup_soft_deleted_notifications(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Soft delete and set deleted_at to 8 days ago
        let old_delete = Utc::now() - chrono::Duration::days(8);
        sqlx::query!("UPDATE wallet_notifications SET deleted_at = $1 WHERE id = $2", old_delete, id)
            .execute(&pool)
            .await?;

        // Run cleanup
        cleanup_old_notifications(&pool, 0).await?;

        // Verify notification was deleted
        let result = sqlx::query!("SELECT id FROM wallet_notifications WHERE id = $1", id)
            .fetch_optional(&pool)
            .await?;

        assert!(result.is_none(), "Soft-deleted notification should be permanently removed");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx:test]
    async fn test_cleanup_old_read_notifications(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Mark as read and set created_at to 91 days ago
        let old_date = Utc::now() - chrono::Duration::days(91);
        sqlx::query!("UPDATE wallet_notifications SET read_at = NOW(), created_at = $1 WHERE id = $2", old_date, id)
            .execute(&pool)
            .await?;

        // Run cleanup
        cleanup_old_notifications(&pool, 0).await?;

        // Verify notification was deleted
        let result = sqlx::query!("SELECT id FROM wallet_notifications WHERE id = $1", id)
            .fetch_optional(&pool)
            .await?;

        assert!(result.is_none(), "Old read notification should be removed");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_cleanup_expired_notifications(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";
        let id = setup_test_notification(&pool, wallet, "system", "normal").await?;

        // Set expiry to past
        let past = Utc::now() - chrono::Duration::hours(1);
        sqlx::query!("UPDATE wallet_notifications SET expires_at = $1 WHERE id = $2", past, id)
            .execute(&pool)
            .await?;

        // Run cleanup
        cleanup_old_notifications(&pool, 0).await?;

        // Verify notification was deleted
        let result = sqlx::query!("SELECT id FROM wallet_notifications WHERE id = $1", id)
            .fetch_optional(&pool)
            .await?;

        assert!(result.is_none(), "Expired notification should be removed");

        cleanup_test_notifications(&pool).await?;
        Ok(())
    }

    // ============================================================================
    // NOTIFICATION TYPE AND PRIORITY PARSING TESTS
    // ============================================================================

    #[sqlx::test]
    async fn test_parse_notification_types(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Test all valid types
        let types = vec![
            ("system", "System"),
            ("security", "Security"),
            ("permission", "Permission"),
            ("wallet_management", "WalletManagement"),
            ("wallet", "Wallet"),
            ("payment", "Payment"),
            ("general", "General"),
        ];

        for (db_type, _expected) in types {
            let id = setup_test_notification(&pool, wallet, db_type, "normal").await?;
            let notifications = fetch_queued_notifications(&pool, wallet).await?;

            let found = notifications.iter().find(|n| n.id == id.to_string());
            assert!(found.is_some(), "Should fetch notification with type: {}", db_type);

            sqlx::query!("DELETE FROM wallet_notifications WHERE id = $1", id)
                .execute(&pool)
                .await?;
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_parse_priorities(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let wallet = "0x1234567890abcdef1234567890abcdef12345678";

        // Test all valid priorities
        let priorities = vec!["low", "normal", "high", "critical", "urgent"];

        for priority in priorities {
            let id = setup_test_notification(&pool, wallet, "system", priority).await?;
            let notifications = fetch_queued_notifications(&pool, wallet).await?;

            let found = notifications.iter().find(|n| n.id == id.to_string());
            assert!(found.is_some(), "Should fetch notification with priority: {}", priority);

            sqlx::query!("DELETE FROM wallet_notifications WHERE id = $1", id)
                .execute(&pool)
                .await?;
        }

        Ok(())
    }
}
