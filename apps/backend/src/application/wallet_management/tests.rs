// Comprehensive tests for Wallet Management Application Layer
// Tests query handlers, command handlers, and application services

#[cfg(test)]
mod query_handler_tests {
    use super::super::*;
    use crate::domain::wallet_management::{
        WalletUser, WalletAddress, Permission,
        WalletUserRepositoryPort,
        repository_ports::{WalletUserSearchPort, WalletUserSearchCriteria, WalletUserSearchResult},
        SessionRepositoryPort, Session, SessionId, SessionSearchCriteria, SessionSearchResult, SessionStatistics,
    };
    use crate::prelude::AppResult;
    use crate::domain::wallet_management::value_objects::PermissionType;
    use crate::domain::shared_kernel::value_objects::UserId;
    use crate::application::shared::{PaginationParams, QueryHandler};
    use crate::core::errors::AppError;
    use std::sync::Arc;
    use std::collections::HashSet;
    use async_trait::async_trait;

    // ============================================================================
    // MOCK IMPLEMENTATIONS
    // ============================================================================

    /// Mock WalletUserRepositoryPort for testing
    pub struct MockWalletUserRepository {
        wallets: Vec<WalletUser>,
    }

    impl MockWalletUserRepository {
        pub fn new() -> Self {
            Self {
                wallets: Vec::new(),
            }
        }

        pub fn with_wallet(mut self, wallet: WalletUser) -> Self {
            self.wallets.push(wallet);
            self
        }
    }

    #[async_trait]
    impl WalletUserSearchPort for MockWalletUserRepository {
        async fn find_by_criteria(
            &self,
            _criteria: &WalletUserSearchCriteria,
            _limit: u32,
            _offset: u32,
        ) -> AppResult<WalletUserSearchResult> {
            Ok(WalletUserSearchResult::new(
                self.wallets.clone(),
                self.wallets.len() as u64,
                0,
                10,
            ))
        }

        async fn count_by_criteria(&self, _criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
            Ok(self.wallets.len() as u64)
        }

        async fn find_by_permission(&self, _permission: &Permission) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn find_by_permission_type(&self, _permission_type: &PermissionType) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn find_by_permission_group(&self, _permission_group: &str) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn find_by_nft_ownership(&self, _contract_address: &str, _token_ids: Option<&[u64]>, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn find_by_token_balance(&self, _contract_address: &str, _min_balance: &str, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn find_by_dao_membership(&self, _dao_contract: &str, _min_voting_power: &str, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn validate_web3_permissions(&self, _wallet_address: &WalletAddress, _permissions: &[Permission]) -> AppResult<Vec<bool>> {
            Ok(Vec::new())
        }

        async fn cache_web3_validation(&self, _wallet_address: &WalletAddress, _permission: &Permission, _is_valid: bool, _cache_duration_seconds: u64) -> AppResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl WalletUserRepositoryPort for MockWalletUserRepository {
        async fn find_by_wallet(&self, wallet_addr: &WalletAddress) -> AppResult<Option<WalletUser>> {
            Ok(self.wallets.iter()
                .find(|w| w.wallet_address() == wallet_addr)
                .cloned())
        }

        async fn find_by_wallets(&self, wallet_addresses: &[WalletAddress]) -> AppResult<Vec<WalletUser>> {
            Ok(self.wallets.iter()
                .filter(|w| wallet_addresses.contains(w.wallet_address()))
                .cloned()
                .collect())
        }

        async fn save(&self, _wallet: &WalletUser) -> AppResult<()> {
            Ok(())
        }

        async fn delete(&self, _wallet_addr: &WalletAddress) -> AppResult<()> {
            Ok(())
        }

        async fn find_eligible_for_web3_permissions(&self, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
            Ok(Vec::new())
        }

        async fn save_batch(&self, _users: &[WalletUser]) -> AppResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> AppResult<()> {
            Ok(())
        }

        async fn cleanup_expired_permissions(&self) -> AppResult<u32> {
            Ok(0)
        }
    }

    /// Mock SessionRepositoryPort for testing
    pub struct MockSessionRepository {
        sessions: Vec<Session>,
    }

    impl MockSessionRepository {
        pub fn new() -> Self {
            Self {
                sessions: Vec::new(),
            }
        }

        pub fn with_session(mut self, session: Session) -> Self {
            self.sessions.push(session);
            self
        }
    }

    #[async_trait]
    impl SessionRepositoryPort for MockSessionRepository {
        async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, AppError> {
            Ok(self.sessions.iter()
                .find(|s| s.id() == id)
                .cloned())
        }

        async fn find_by_wallet_id(&self, wallet_addr: &UserId) -> Result<Vec<Session>, AppError> {
            Ok(self.sessions.iter()
                .filter(|s| s.user_id().to_string() == wallet_addr.to_string())
                .cloned()
                .collect())
        }

        async fn find_active_by_wallet_id(&self, wallet_addr: &UserId) -> Result<Vec<Session>, AppError> {
            Ok(self.sessions.iter()
                .filter(|s| s.user_id().to_string() == wallet_addr.to_string() && s.is_valid())
                .cloned()
                .collect())
        }

        async fn find_by_access_token(&self, _token: &str) -> Result<Option<Session>, AppError> {
            Ok(None)
        }

        async fn find_by_refresh_token(&self, _token: &str) -> Result<Option<Session>, AppError> {
            Ok(None)
        }

        async fn save(&self, _session: &Session) -> Result<(), AppError> {
            Ok(())
        }

        async fn delete(&self, _id: &SessionId) -> Result<(), AppError> {
            Ok(())
        }

        async fn invalidate_all_for_wallet(&self, _wallet_addr: &UserId) -> Result<u32, AppError> {
            Ok(0)
        }

        async fn find_expired_sessions(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<Vec<Session>, AppError> {
            Ok(Vec::new())
        }

        async fn cleanup_expired(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<u32, AppError> {
            Ok(0)
        }

        async fn find_by_criteria(
            &self,
            _criteria: &SessionSearchCriteria,
            _limit: u32,
            _offset: u32,
        ) -> Result<SessionSearchResult, AppError> {
            Ok(SessionSearchResult::new(Vec::new(), 0, 0, 10))
        }

        async fn count_by_criteria(&self, _criteria: &SessionSearchCriteria) -> Result<u64, AppError> {
            Ok(0)
        }

        async fn next_identity(&self) -> Result<SessionId, AppError> {
            Ok(SessionId::from(uuid::Uuid::new_v4().to_string()))
        }

        async fn health_check(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn save_batch(&self, _sessions: &[Session]) -> Result<(), AppError> {
            Ok(())
        }

        async fn find_sessions_needing_renewal(&self, _threshold: chrono::Duration) -> Result<Vec<Session>, AppError> {
            Ok(Vec::new())
        }

        async fn get_session_statistics(&self) -> Result<SessionStatistics, AppError> {
            Ok(SessionStatistics {
                total_sessions: 0,
                active_sessions: 0,
                expired_sessions: 0,
                revoked_sessions: 0,
                sessions_created_24h: 0,
                sessions_expired_24h: 0,
                average_session_duration_minutes: 0.0,
                unique_wallets_with_sessions: 0,
            })
        }
    }

    // ============================================================================
    // QUERY HANDLER TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_get_wallet_query_handler_success() {
        // Arrange
        let wallet_addr = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let wallet = WalletUser::create(wallet_addr.clone(), HashSet::new()).unwrap();

        let wallet_repo = Arc::new(MockWalletUserRepository::new().with_wallet(wallet));
        let session_repo = Arc::new(MockSessionRepository::new());

        let handler = GetWalletQueryHandler::new(wallet_repo, session_repo);

        let query = GetWalletQuery {
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            include_permissions: false,
            include_sessions: false,
            correlation_id: None,
            requested_by: None,
        };

        // Act
        let result = handler.handle(query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.wallet_address.to_string(), "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6");
        assert!(response.is_active);
    }

    #[tokio::test]
    async fn test_get_wallet_query_handler_not_found() {
        // Arrange
        let wallet_repo = Arc::new(MockWalletUserRepository::new());
        let session_repo = Arc::new(MockSessionRepository::new());

        let handler = GetWalletQueryHandler::new(wallet_repo, session_repo);

        let query = GetWalletQuery {
            wallet_address: "0x0000000000000000000000000000000000000000".to_string(),
            include_permissions: false,
            include_sessions: false,
            correlation_id: None,
            requested_by: None,
        };

        // Act
        let result = handler.handle(query).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, crate::application::shared::error::ApplicationError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_search_wallets_query_handler() {
        // Arrange
        let wallet1 = WalletUser::create(
            WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap(),
            HashSet::new()
        ).unwrap();
        let wallet2 = WalletUser::create(
            WalletAddress::new("0x1234567890123456789012345678901234567890".to_string()).unwrap(),
            HashSet::new()
        ).unwrap();

        let wallet_repo = Arc::new(
            MockWalletUserRepository::new()
                .with_wallet(wallet1)
                .with_wallet(wallet2)
        );

        let handler = SearchWalletsQueryHandler::new(wallet_repo);

        let query = SearchWalletsQuery {
            search_term: None,
            wallet_pattern: None,
            is_active: Some(true),
            has_permissions: Vec::new(),
            created_after: None,
            created_before: None,
            last_login_after: None,
            pagination: PaginationParams::new(1, 10),
            sort: None,
            include_stats: false,
            requested_by: None,
            correlation_id: None,
        };

        // Act
        let result = handler.handle(query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 2);
    }

    #[tokio::test]
    async fn test_get_wallet_permissions_query_handler() {
        // Arrange
        let wallet_addr = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let mut wallet = WalletUser::create(wallet_addr.clone(), HashSet::new()).unwrap();

        // Grant some permissions
        let perm1 = Permission::new("epsx:analytics:read").unwrap();
        let perm2 = Permission::new("epsx:data:access").unwrap();
        wallet.grant_permission(perm1).unwrap();
        wallet.grant_permission(perm2).unwrap();

        let wallet_repo = Arc::new(MockWalletUserRepository::new().with_wallet(wallet));
        let handler = GetWalletPermissionsQueryHandler::new(wallet_repo);

        let query = GetWalletPermissionsQuery {
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            include_expired: false,
        };

        // Act
        let result = handler.handle(query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.permissions.len(), 2);
        assert_eq!(response.active_permissions.len(), 2);
        assert_eq!(response.expired_permissions.len(), 0);
    }

    #[tokio::test]
    async fn test_list_wallets_query_handler_pagination() {
        // Arrange
        let wallet_repo = Arc::new(
            MockWalletUserRepository::new()
                .with_wallet(WalletUser::create(WalletAddress::new("0x1111111111111111111111111111111111111111".to_string()).unwrap(), HashSet::new()).unwrap())
                .with_wallet(WalletUser::create(WalletAddress::new("0x2222222222222222222222222222222222222222".to_string()).unwrap(), HashSet::new()).unwrap())
                .with_wallet(WalletUser::create(WalletAddress::new("0x3333333333333333333333333333333333333333".to_string()).unwrap(), HashSet::new()).unwrap())
        );

        let handler = ListWalletsQueryHandler::new(wallet_repo);

        let query = ListWalletsQuery {
            limit: 10,
            offset: 0,
            wallet_pattern_filter: None,
            permission_filter: None,
        };

        // Act
        let result = handler.handle(query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 3);
        assert_eq!(response.total_count, 3);
    }
}

#[cfg(test)]
mod wallet_management_service_tests {
    // NOTE: Tests below are disabled - WalletManagementService was removed as part of CQRS standardization
    // Functionality is now handled by CQRS command/query handlers instead of service layer

    /*
    #[tokio::test]
    async fn test_wallet_management_service_creation() {
        // Arrange
        let wallet_repo = Arc::new(MockWalletUserRepository::new()) as Arc<dyn WalletUserRepositoryPort>;
        let session_repo = Arc::new(MockSessionRepository::new()) as Arc<dyn SessionRepositoryPort>;
        let event_bus = Arc::new(MockEventBus) as Arc<dyn crate::domain::DomainEventBus>;

        // Act
        let _service = WalletManagementService::new(wallet_repo, session_repo, event_bus);

        // Assert - just verify it compiles and creates successfully
        assert!(true, "WalletManagementService created successfully");
    }

    #[tokio::test]
    async fn test_get_wallet_statistics() {
        // Arrange
        let wallet_addr = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let mut wallet = WalletUser::create(wallet_addr.clone(), HashSet::new()).unwrap();

        // Add permissions
        wallet.grant_permission(Permission::new("epsx:analytics:read").unwrap()).unwrap();
        wallet.grant_permission(Permission::new("epsx:data:access").unwrap()).unwrap();

        let wallet_repo = Arc::new(MockWalletUserRepository::new().with_wallet(wallet)) as Arc<dyn WalletUserRepositoryPort>;
        let session_repo = Arc::new(MockSessionRepository::new()) as Arc<dyn SessionRepositoryPort>;
        let event_bus = Arc::new(MockEventBus) as Arc<dyn crate::domain::DomainEventBus>;

        let service = WalletManagementService::new(wallet_repo, session_repo, event_bus);

        // Act
        let result = service.get_wallet_statistics("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).await;

        // Assert
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_permissions, 2);
        assert_eq!(stats.active_permissions, 2);
        assert_eq!(stats.expired_permissions, 0);
    }
    */
}

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use crate::application::shared::PaginationParams;

    #[tokio::test]
    async fn test_search_query_validation() {
        use crate::application::shared::Query;

        // Test that queries properly validate their inputs
        let query = SearchWalletsQuery {
            search_term: Some("a".to_string()), // Too short
            wallet_pattern: None,
            is_active: None,
            has_permissions: Vec::new(),
            created_after: None,
            created_before: None,
            last_login_after: None,
            pagination: PaginationParams::new(1, 10),
            sort: None,
            include_stats: false,
            requested_by: None,
            correlation_id: None,
        };

        let result = query.validate();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_query_invalid_date_range() {
        use crate::application::shared::Query;
        use chrono::Utc;

        // Test that date range validation works
        let now = Utc::now();
        let future = now + chrono::Duration::hours(1);

        let query = SearchWalletsQuery {
            search_term: None,
            wallet_pattern: None,
            is_active: None,
            has_permissions: Vec::new(),
            created_after: Some(future),
            created_before: Some(now),
            last_login_after: None,
            pagination: PaginationParams::new(1, 10),
            sort: None,
            include_stats: false,
            requested_by: None,
            correlation_id: None,
        };

        let result = query.validate();
        assert!(result.is_err());
    }
}
