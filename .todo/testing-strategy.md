# Testing Strategy for Backend-Centralized Auth Migration

## Overview
Comprehensive testing strategy to ensure the backend-centralized auth migration maintains functionality, security, and performance while eliminating frontend auth logic.

## Testing Principles

### Quality Gates
- **Zero Regression**: No loss of existing functionality
- **Security First**: Enhanced security with no vulnerabilities
- **Performance Maintained**: No degradation in performance metrics
- **User Experience**: Seamless experience for all user types
- **Firebase Analytics Preserved**: Analytics functionality unchanged

### Test-Driven Migration
- Write tests before implementing changes
- Maintain >90% test coverage throughout migration
- Use feature flags for safe deployment
- Comprehensive regression testing

## Phase 1: Backend API Foundation Testing

### Unit Tests (Backend)

#### 1.1 Session Management APIs
**Location**: `apps/backend/__tests__/unit/auth/`

```rust
// Test session validation endpoint
#[tokio::test]
async fn test_validate_session_endpoint() {
    let app = setup_test_app().await;
    let session = create_test_session("regular").await;
    
    let response = app
        .post("/api/v1/auth/validate-session")
        .header("Cookie", format!("sess_id={}", session.id))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let body: SessionValidationResponse = response.json().await;
    assert!(body.authenticated);
    assert_eq!(body.user_id, session.user_id);
}

#[tokio::test]
async fn test_validate_session_expired() {
    let app = setup_test_app().await;
    let session = create_expired_session().await;
    
    let response = app
        .post("/api/v1/auth/validate-session")
        .header("Cookie", format!("sess_id={}", session.id))
        .send()
        .await;
    
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_validate_session_invalid_cookie() {
    let app = setup_test_app().await;
    
    let response = app
        .post("/api/v1/auth/validate-session")
        .header("Cookie", "sess_id=invalid")
        .send()
        .await;
    
    assert_eq!(response.status(), 401);
}
```

#### 1.2 Route Protection APIs
```rust
#[tokio::test]
async fn test_validate_route_access_allowed() {
    let app = setup_test_app().await;
    let user = create_test_user_with_permissions(vec!["dashboard:view"]).await;
    let session = create_session_for_user(user.id).await;
    
    let response = app
        .post("/api/v1/auth/validate-access")
        .header("Cookie", format!("sess_id={}", session.id))
        .json(&RouteAccessRequest {
            route: "/dashboard".to_string(),
            method: "GET".to_string(),
            app_type: AppType::Frontend,
        })
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let body: RouteAccessResponse = response.json().await;
    assert!(body.allowed);
}

#[tokio::test]
async fn test_validate_route_access_denied() {
    let app = setup_test_app().await;
    let user = create_test_user_with_permissions(vec![]).await;
    let session = create_session_for_user(user.id).await;
    
    let response = app
        .post("/api/v1/auth/validate-access")
        .header("Cookie", format!("sess_id={}", session.id))
        .json(&RouteAccessRequest {
            route: "/admin".to_string(),
            method: "GET".to_string(),
            app_type: AppType::Admin,
        })
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let body: RouteAccessResponse = response.json().await;
    assert!(!body.allowed);
}
```

#### 1.3 Permission Checking APIs
```rust
#[tokio::test]
async fn test_check_permission_direct() {
    let permission_checker = setup_permission_checker().await;
    let user_id = create_test_user_with_permissions(vec!["dashboard:view"]).await.id;
    
    let result = permission_checker.check_permission_with_context(
        user_id,
        "dashboard:view",
        None,
        None,
    ).await.unwrap();
    
    assert!(result.allowed);
    assert_eq!(result.reason, PermissionReason::DirectPermission);
}

#[tokio::test]
async fn test_check_permission_wildcard() {
    let permission_checker = setup_permission_checker().await;
    let user_id = create_test_user_with_permissions(vec!["dashboard:*"]).await.id;
    
    let result = permission_checker.check_permission_with_context(
        user_id,
        "dashboard:view",
        None,
        None,
    ).await.unwrap();
    
    assert!(result.allowed);
    matches!(result.reason, PermissionReason::WildcardMatch(_));
}
```

### Integration Tests (Backend)

#### 1.1 End-to-End Auth Flow
**Location**: `apps/backend/__tests__/integration/`

```rust
#[tokio::test]
async fn test_complete_auth_flow() {
    let app = setup_test_app().await;
    
    // Login
    let login_response = app
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            app_origin: AppOrigin::Frontend,
        })
        .send()
        .await;
    
    assert_eq!(login_response.status(), 200);
    let session_cookie = extract_session_cookie(&login_response);
    
    // Validate session
    let validate_response = app
        .post("/api/v1/auth/validate-session")
        .header("Cookie", session_cookie.clone())
        .send()
        .await;
    
    assert_eq!(validate_response.status(), 200);
    
    // Check route access
    let route_response = app
        .post("/api/v1/auth/validate-access")
        .header("Cookie", session_cookie.clone())
        .json(&RouteAccessRequest {
            route: "/dashboard".to_string(),
            method: "GET".to_string(),
            app_type: AppType::Frontend,
        })
        .send()
        .await;
    
    assert_eq!(route_response.status(), 200);
    
    // Logout
    let logout_response = app
        .post("/api/v1/auth/logout")
        .header("Cookie", session_cookie)
        .send()
        .await;
    
    assert_eq!(logout_response.status(), 200);
}
```

### Performance Tests (Backend)

#### 1.1 API Response Time Tests
**Location**: `apps/backend/__tests__/performance/`

```rust
#[tokio::test]
async fn test_auth_api_performance() {
    let app = setup_test_app().await;
    let session = create_test_session("regular").await;
    let session_cookie = format!("sess_id={}", session.id);
    
    // Test session validation performance
    let start = Instant::now();
    for _ in 0..100 {
        let response = app
            .post("/api/v1/auth/validate-session")
            .header("Cookie", session_cookie.clone())
            .send()
            .await;
        assert_eq!(response.status(), 200);
    }
    let duration = start.elapsed();
    
    // Should complete 100 requests in less than 1 second
    assert!(duration.as_millis() < 1000);
    
    // Average response time should be under 10ms
    let avg_response_time = duration.as_millis() / 100;
    assert!(avg_response_time < 10);
}

#[tokio::test]
async fn test_permission_check_performance() {
    let app = setup_test_app().await;
    let user = create_test_user_with_permissions(vec!["dashboard:view", "analytics:view"]).await;
    let session = create_session_for_user(user.id).await;
    let session_cookie = format!("sess_id={}", session.id);
    
    let start = Instant::now();
    for _ in 0..1000 {
        let response = app
            .post("/api/v1/auth/check-permission")
            .header("Cookie", session_cookie.clone())
            .json(&PermissionCheckRequest {
                permission: "dashboard:view".to_string(),
                resource: None,
                context: None,
            })
            .send()
            .await;
        assert_eq!(response.status(), 200);
    }
    let duration = start.elapsed();
    
    // Should complete 1000 permission checks in less than 2 seconds
    assert!(duration.as_millis() < 2000);
}
```

### Security Tests (Backend)

#### 1.1 Authentication Security Tests
```rust
#[tokio::test]
async fn test_session_hijacking_prevention() {
    let app = setup_test_app().await;
    let session = create_test_session("regular").await;
    
    // Attempt to use session from different IP
    let response = app
        .post("/api/v1/auth/validate-session")
        .header("Cookie", format!("sess_id={}", session.id))
        .header("X-Forwarded-For", "malicious-ip")
        .send()
        .await;
    
    // Should detect IP change and potentially invalidate session
    // Implementation depends on security policy
    assert!(response.status() == 401 || response.status() == 200);
}

#[tokio::test]
async fn test_csrf_protection() {
    let app = setup_test_app().await;
    let session = create_test_session("regular").await;
    
    // Attempt state-changing operation without CSRF token
    let response = app
        .post("/api/v1/auth/logout")
        .header("Cookie", format!("sess_id={}", session.id))
        // No CSRF token provided
        .send()
        .await;
    
    assert_eq!(response.status(), 403); // Should be blocked by CSRF protection
}
```

## Phase 2: Frontend Simplification Testing

### Frontend Unit Tests

#### 2.1 Simplified Middleware Tests
**Location**: `apps/frontend/__tests__/middleware/`

```typescript
// Mock the backend API
jest.mock('node-fetch');
const mockFetch = fetch as jest.MockedFunction<typeof fetch>;

describe('Simplified Middleware', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('allows access when backend validates route access', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        route_permissions: {
          '/dashboard': { allowed: true, permissions: ['dashboard:view'] }
        }
      })
    } as Response);

    const request = new NextRequest('http://localhost:3000/dashboard');
    const response = await middleware(request);

    expect(response).toBeInstanceOf(NextResponse);
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/v1/auth/validate-route-permissions'),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          routes: expect.arrayContaining([
            expect.objectContaining({
              path: '/dashboard',
              method: 'GET',
              app_type: 'Frontend'
            })
          ])
        })
      })
    );
  });

  test('redirects to unauthorized when backend denies access', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        route_permissions: {
          '/admin': { allowed: false, permissions: [] }
        }
      })
    } as Response);

    const request = new NextRequest('http://localhost:3000/admin');
    const response = await middleware(request);

    expect(response.status).toBe(307); // Redirect
    expect(response.headers.get('Location')).toContain('/unauthorized');
  });

  test('handles API failures gracefully', async () => {
    mockFetch.mockRejectedValueOnce(new Error('API unavailable'));

    const request = new NextRequest('http://localhost:3000/dashboard');
    const response = await middleware(request);

    expect(response.status).toBe(307); // Redirect to login
    expect(response.headers.get('Location')).toContain('/login');
  });
});
```

#### 2.2 Simplified Auth Context Tests
```typescript
describe('Simplified Auth Context', () => {
  test('fetches user data from backend API', async () => {
    const mockUserData = {
      user: { id: '1', email: 'test@example.com' },
      permissions: ['dashboard:view'],
      authenticated: true
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockUserData
    } as Response);

    const { result, waitForNextUpdate } = renderHook(() => useAuth(), {
      wrapper: AuthProvider,
    });

    await waitForNextUpdate();

    expect(result.current.user).toEqual(mockUserData.user);
    expect(result.current.permissions).toEqual(mockUserData.permissions);
    expect(result.current.authenticated).toBe(true);
  });

  test('handles authentication failures', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401
    } as Response);

    const { result, waitForNextUpdate } = renderHook(() => useAuth(), {
      wrapper: AuthProvider,
    });

    await waitForNextUpdate();

    expect(result.current.authenticated).toBe(false);
    expect(result.current.user).toBe(null);
  });
});
```

#### 2.3 Permission Hook Tests
```typescript
describe('usePermissions Hook', () => {
  test('checks permission via backend API', async () => {
    const mockPermissionResponse = {
      results: [{ permission: 'dashboard:view', allowed: true }]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPermissionResponse
    } as Response);

    const { result } = renderHook(() => usePermissions());
    const hasPermission = await result.current.checkPermission('dashboard:view');

    expect(hasPermission).toBe(true);
    expect(mockFetch).toHaveBeenCalledWith(
      '/api/v1/auth/validate-permissions',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          permissions: [{ permission: 'dashboard:view' }]
        })
      })
    );
  });

  test('handles multiple permission checks', async () => {
    const mockResponse = {
      results: [
        { permission: 'dashboard:view', allowed: true },
        { permission: 'admin:users', allowed: false }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const { result } = renderHook(() => usePermissions());
    const results = await result.current.checkMultiplePermissions([
      { permission: 'dashboard:view' },
      { permission: 'admin:users' }
    ]);

    expect(results).toHaveLength(2);
    expect(results[0].allowed).toBe(true);
    expect(results[1].allowed).toBe(false);
  });
});
```

### Firebase Analytics Preservation Tests

#### 2.1 Analytics Functionality Tests
```typescript
describe('Firebase Analytics Preservation', () => {
  test('analytics tracking still works after auth changes', async () => {
    // Mock Firebase Analytics
    const mockAnalytics = {
      logEvent: jest.fn(),
      setUserId: jest.fn(),
      setUserProperties: jest.fn()
    };

    jest.mock('firebase/analytics', () => ({
      getAnalytics: () => mockAnalytics,
      logEvent: mockAnalytics.logEvent,
      setUserId: mockAnalytics.setUserId,
      setUserProperties: mockAnalytics.setUserProperties
    }));

    const { firebaseAnalytics } = await import('lib/firebase-analytics');

    // Simulate user login
    firebaseAnalytics.setUserId('test-user-id');
    firebaseAnalytics.logEvent('login', { method: 'email' });

    expect(mockAnalytics.setUserId).toHaveBeenCalledWith('test-user-id');
    expect(mockAnalytics.logEvent).toHaveBeenCalledWith('login', { method: 'email' });
  });

  test('analytics hooks still function correctly', () => {
    const { result } = renderHook(() => useFirebaseAnalytics());

    expect(result.current.logEvent).toBeDefined();
    expect(result.current.setUserId).toBeDefined();
    expect(result.current.setUserProperties).toBeDefined();

    // Test analytics event logging
    act(() => {
      result.current.logEvent('page_view', { page_title: 'Dashboard' });
    });

    // Should not throw errors
    expect(true).toBe(true);
  });
});
```

### Integration Tests (Frontend)

#### 2.1 End-to-End Auth Flow Tests
```typescript
describe('End-to-End Auth Flow', () => {
  test('complete login to dashboard flow works', async () => {
    // Mock successful login response
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, user: { id: '1' } })
      })
      // Mock session validation
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ authenticated: true, user: { id: '1' } })
      })
      // Mock route validation
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          route_permissions: { '/dashboard': { allowed: true } }
        })
      });

    render(<App />);

    // Navigate to login
    fireEvent.click(screen.getByText('Login'));

    // Fill login form
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'test@example.com' }
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password' }
    });

    // Submit login
    fireEvent.click(screen.getByText('Sign In'));

    // Should redirect to dashboard
    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    });

    // Verify API calls were made
    expect(mockFetch).toHaveBeenCalledTimes(3);
  });
});
```

## Phase 3: Session Unification Testing

### Backend Session Tests

#### 3.1 Unified Session Management Tests
```rust
#[tokio::test]
async fn test_unified_session_creation() {
    let session_repo = setup_session_repo().await;
    let user_id = create_test_user().await.id;

    let session = session_repo.create_unified_session(
        user_id,
        SessionType::Regular,
        AppOrigin::Frontend,
        vec!["dashboard:view".to_string()],
    ).await.unwrap();

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.session_type, SessionType::Regular);
    assert_eq!(session.app_origin, AppOrigin::Frontend);
    assert!(session.capabilities.contains(&"dashboard:view".to_string()));
}

#[tokio::test]
async fn test_cross_app_session_validation() {
    let session_repo = setup_session_repo().await;
    let admin_user = create_admin_user().await;

    // Create admin session
    let session = session_repo.create_unified_session(
        admin_user.id,
        SessionType::Admin,
        AppOrigin::Admin,
        vec!["frontend_access".to_string()],
    ).await.unwrap();

    // Should be valid for admin app
    let admin_validation = session_repo.validate_unified_session(
        &session.id,
        AppOrigin::Admin,
    ).await.unwrap();
    assert!(admin_validation.is_some());

    // Should also be valid for frontend (admin has frontend access)
    let frontend_validation = session_repo.validate_unified_session(
        &session.id,
        AppOrigin::Frontend,
    ).await.unwrap();
    assert!(frontend_validation.is_some());
}

#[tokio::test]
async fn test_regular_user_admin_access_denied() {
    let session_repo = setup_session_repo().await;
    let regular_user = create_regular_user().await;

    let session = session_repo.create_unified_session(
        regular_user.id,
        SessionType::Regular,
        AppOrigin::Frontend,
        vec![],
    ).await.unwrap();

    // Should be valid for frontend
    let frontend_validation = session_repo.validate_unified_session(
        &session.id,
        AppOrigin::Frontend,
    ).await.unwrap();
    assert!(frontend_validation.is_some());

    // Should NOT be valid for admin
    let admin_validation = session_repo.validate_unified_session(
        &session.id,
        AppOrigin::Admin,
    ).await.unwrap();
    assert!(admin_validation.is_none());
}
```

#### 3.2 Session Security Tests
```rust
#[tokio::test]
async fn test_session_security_monitoring() {
    let security_service = setup_session_security_service().await;
    let session_id = "test-session-id";

    // First request
    let request_info1 = RequestInfo {
        ip_address: "192.168.1.1".to_string(),
        user_agent: "Mozilla/5.0...".to_string(),
    };

    let result1 = security_service.validate_session_security(
        session_id,
        request_info1,
    ).await.unwrap();
    assert_eq!(result1, SecurityValidationResult::Valid);

    // Second request from different IP
    let request_info2 = RequestInfo {
        ip_address: "192.168.1.2".to_string(),
        user_agent: "Mozilla/5.0...".to_string(),
    };

    let result2 = security_service.validate_session_security(
        session_id,
        request_info2,
    ).await.unwrap();

    // Should log security event but still allow (depends on policy)
    assert_eq!(result2, SecurityValidationResult::Valid);

    // Verify security event was logged
    let security_events = get_security_events_for_session(session_id).await;
    assert!(!security_events.is_empty());
    assert!(security_events.iter().any(|e| matches!(e, SecurityEvent::IpAddressChange { .. })));
}
```

### Frontend Cookie Tests

#### 3.1 Unified Cookie Handling Tests
```typescript
describe('Unified Cookie Handling', () => {
  test('uses sess_id cookie for both apps', () => {
    // Mock cookies
    Object.defineProperty(document, 'cookie', {
      writable: true,
      value: 'sess_id=test-session-123'
    });

    // Test frontend cookie reading
    const sessionId = CookieReader.get('sess_id');
    expect(sessionId).toBe('test-session-123');

    // Test admin cookie reading (should use same cookie)
    const adminSessionId = CookieReader.get('sess_id');
    expect(adminSessionId).toBe('test-session-123');

    // Should NOT find admin_sess_id
    const oldAdminSession = CookieReader.get('admin_sess_id');
    expect(oldAdminSession).toBeUndefined();
  });

  test('handles missing session cookie gracefully', () => {
    Object.defineProperty(document, 'cookie', {
      writable: true,
      value: ''
    });

    const sessionId = CookieReader.get('sess_id');
    expect(sessionId).toBeUndefined();

    const hasSession = CookieReader.has('sess_id');
    expect(hasSession).toBe(false);
  });
});
```

### Database Migration Tests

#### 3.1 Session Table Migration Tests
```rust
#[tokio::test]
async fn test_session_table_migration() {
    let pool = setup_test_database().await;

    // Run migration
    run_migration(&pool, "003_unify_sessions.sql").await.unwrap();

    // Verify new columns exist
    let columns = sqlx::query!(
        "SELECT column_name FROM information_schema.columns WHERE table_name = 'sessions'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let column_names: Vec<String> = columns.iter()
        .map(|r| r.column_name.clone())
        .collect();

    assert!(column_names.contains(&"session_type".to_string()));
    assert!(column_names.contains(&"app_origin".to_string()));
    assert!(column_names.contains(&"capabilities".to_string()));
    assert!(column_names.contains(&"security_info".to_string()));
}

#[tokio::test]
async fn test_existing_session_migration() {
    let pool = setup_test_database().await;

    // Insert test data before migration
    sqlx::query!(
        "INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES ($1, $2, $3, $4)",
        "old-session-id",
        "user-123",
        Utc::now(),
        Utc::now() + Duration::days(7)
    )
    .execute(&pool)
    .await
    .unwrap();

    // Run migration
    run_migration(&pool, "003_unify_sessions.sql").await.unwrap();

    // Verify existing session was migrated
    let migrated_session = sqlx::query!(
        "SELECT session_type, app_origin FROM sessions WHERE id = $1",
        "old-session-id"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(migrated_session.session_type, Some("regular".to_string()));
    assert_eq!(migrated_session.app_origin, Some("frontend".to_string()));
}
```

## Phase 4: Permission Consolidation Testing

### Backend Permission Tests

#### 4.1 Centralized Permission API Tests
```rust
#[tokio::test]
async fn test_bulk_permission_validation() {
    let app = setup_test_app().await;
    let user = create_test_user_with_permissions(vec![
        "dashboard:view",
        "analytics:view",
        "settings:read"
    ]).await;
    let session = create_session_for_user(user.id).await;

    let response = app
        .post("/api/v1/auth/validate-permissions")
        .header("Cookie", format!("sess_id={}", session.id))
        .json(&PermissionValidationRequest {
            permissions: vec![
                PermissionCheck { permission: "dashboard:view".to_string(), resource: None, context: None },
                PermissionCheck { permission: "admin:users".to_string(), resource: None, context: None },
                PermissionCheck { permission: "analytics:view".to_string(), resource: None, context: None },
            ]
        })
        .send()
        .await;

    assert_eq!(response.status(), 200);
    let body: PermissionValidationResponse = response.json().await;
    
    assert_eq!(body.results.len(), 3);
    assert!(body.results[0].allowed); // dashboard:view - should be allowed
    assert!(!body.results[1].allowed); // admin:users - should be denied
    assert!(body.results[2].allowed); // analytics:view - should be allowed
}

#[tokio::test]
async fn test_context_aware_permission_checking() {
    let permission_checker = setup_permission_checker().await;
    let user_id = create_test_user().await.id;

    // Give user permission to edit their own profile
    create_user_permission(user_id, "profile:edit", Some("own")).await;

    // Should allow editing own profile
    let own_profile_result = permission_checker.check_permission_with_context(
        user_id,
        "profile:edit",
        Some("own"),
        Some(&serde_json::json!({ "owner_id": user_id.to_string() })),
    ).await.unwrap();
    assert!(own_profile_result.allowed);

    // Should deny editing other's profile
    let other_profile_result = permission_checker.check_permission_with_context(
        user_id,
        "profile:edit",
        Some("other"),
        Some(&serde_json::json!({ "owner_id": "other-user-id" })),
    ).await.unwrap();
    assert!(!other_profile_result.allowed);
}
```

#### 4.2 Real-time Permission Update Tests
```rust
#[tokio::test]
async fn test_real_time_permission_notifications() {
    let notifier = setup_permission_notifier().await;
    let user_id = create_test_user().await.id;

    let mut receiver = notifier.subscribe_to_user_updates(user_id).await;

    // Simulate permission change
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        notifier.notify_permission_change(PermissionChange {
            change_type: PermissionChangeType::GRANTED,
            affected_users: vec![user_id],
            affected_permissions: vec!["new:permission".to_string()],
            new_permissions: vec!["new:permission".to_string()],
        }).await.unwrap();
    });

    // Wait for notification
    let update = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive update within 1 second")
        .unwrap();

    assert_eq!(update.update_type, PermissionUpdateType::GRANTED);
    assert!(update.affected_permissions.contains(&"new:permission".to_string()));
}

#[tokio::test]
async fn test_permission_cache_invalidation() {
    let cache = setup_permission_cache().await;
    let user_id = create_test_user().await.id;

    // Cache some permissions
    let permissions = create_test_permission_set();
    cache.cache_permissions(user_id, &permissions).await.unwrap();

    // Verify cached
    let cached = cache.get_cached_permissions(user_id).await.unwrap();
    assert!(cached.is_some());

    // Invalidate cache
    cache.invalidate_user_permissions(user_id).await.unwrap();

    // Verify cache cleared
    let after_invalidation = cache.get_cached_permissions(user_id).await.unwrap();
    assert!(after_invalidation.is_none());
}
```

### Frontend Permission Tests

#### 4.1 Permission Hook Tests (API-based)
```typescript
describe('API-based Permission Hooks', () => {
  test('usePermissions fetches from backend API', async () => {
    const mockPermissionStatus = {
      permissions: ['dashboard:view', 'analytics:view'],
      roles: ['user'],
      permission_profiles: ['Silver'],
      last_updated: new Date().toISOString()
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPermissionStatus
    } as Response);

    const { result, waitForNextUpdate } = renderHook(() => usePermissions());

    await waitForNextUpdate();

    expect(result.current.permissions).toEqual(['dashboard:view', 'analytics:view']);
    expect(result.current.roles).toEqual(['user']);
    expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/permission-status');
  });

  test('real-time permission updates via WebSocket', async () => {
    const mockWebSocket = new MockWebSocket();
    global.WebSocket = jest.fn(() => mockWebSocket);

    const { result } = renderHook(() => usePermissions());

    // Simulate WebSocket message
    act(() => {
      mockWebSocket.onmessage({
        data: JSON.stringify({
          update_type: 'GRANTED',
          affected_permissions: ['new:permission'],
          timestamp: new Date().toISOString()
        })
      });
    });

    // Should trigger SWR revalidation
    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/permission-status');
    });
  });

  test('permission checking calls backend API', async () => {
    const mockResponse = {
      results: [{ permission: 'dashboard:view', allowed: true }]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const { result } = renderHook(() => usePermissions());
    const hasPermission = await result.current.checkPermission('dashboard:view');

    expect(hasPermission).toBe(true);
    expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/validate-permissions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        permissions: [{ permission: 'dashboard:view' }]
      })
    });
  });
});
```

#### 4.2 Permission Guard Component Tests
```typescript
describe('PermissionGuard Component', () => {
  test('shows content when permission is granted', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        results: [{ permission: 'dashboard:view', allowed: true }]
      })
    } as Response);

    render(
      <PermissionGuard permission="dashboard:view">
        <div>Protected Content</div>
      </PermissionGuard>
    );

    expect(screen.getByText('Checking permissions...')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText('Protected Content')).toBeInTheDocument();
    });
  });

  test('shows fallback when permission is denied', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        results: [{ permission: 'admin:users', allowed: false }]
      })
    } as Response);

    render(
      <PermissionGuard 
        permission="admin:users"
        fallback={<div>Access Denied</div>}
      >
        <div>Admin Content</div>
      </PermissionGuard>
    );

    await waitFor(() => {
      expect(screen.getByText('Access Denied')).toBeInTheDocument();
    });

    expect(screen.queryByText('Admin Content')).not.toBeInTheDocument();
  });
});
```

## Performance Testing

### Load Testing

#### 1.1 Backend API Load Tests
```rust
#[tokio::test]
async fn test_auth_api_under_load() {
    let app = setup_test_app().await;
    let sessions: Vec<_> = (0..100)
        .map(|_| create_test_session("regular"))
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Concurrent API calls
    let tasks: Vec<_> = sessions
        .iter()
        .map(|session| {
            let app = app.clone();
            let session_id = session.id.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let response = app
                    .post("/api/v1/auth/validate-session")
                    .header("Cookie", format!("sess_id={}", session_id))
                    .send()
                    .await;
                let duration = start.elapsed();
                (response.status(), duration)
            })
        })
        .collect();

    let results: Vec<_> = future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All requests should succeed
    assert!(results.iter().all(|(status, _)| *status == 200));

    // 95th percentile response time should be under 50ms
    let mut durations: Vec<_> = results.iter().map(|(_, duration)| *duration).collect();
    durations.sort();
    let p95_index = (durations.len() as f64 * 0.95) as usize;
    let p95_duration = durations[p95_index];
    
    assert!(p95_duration.as_millis() < 50);
}
```

#### 1.2 Permission Checking Performance Tests
```typescript
describe('Permission Checking Performance', () => {
  test('bulk permission checks are efficient', async () => {
    const permissions = Array.from({ length: 50 }, (_, i) => 
      ({ permission: `permission:${i}` })
    );

    const mockResponse = {
      results: permissions.map(p => ({ ...p, allowed: true }))
    };

    let callCount = 0;
    mockFetch.mockImplementation(async () => {
      callCount++;
      return {
        ok: true,
        json: async () => mockResponse
      } as Response;
    });

    const { result } = renderHook(() => usePermissions());
    
    const start = Date.now();
    await result.current.checkMultiplePermissions(permissions);
    const duration = Date.now() - start;

    // Should complete in reasonable time
    expect(duration).toBeLessThan(100);
    
    // Should only make one API call for bulk check
    expect(callCount).toBe(1);
  });
});
```

## Security Testing

### Penetration Testing

#### 1.1 Authentication Security Tests
```typescript
describe('Authentication Security', () => {
  test('prevents session fixation attacks', async () => {
    // Attempt to set malicious session cookie
    const maliciousSessionId = 'malicious-session-123';
    
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401
    } as Response);

    Object.defineProperty(document, 'cookie', {
      writable: true,
      value: `sess_id=${maliciousSessionId}`
    });

    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.authenticated).toBe(false);
    });

    // Verify backend was called to validate the session
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/v1/auth/validate-session'),
      expect.objectContaining({
        headers: expect.objectContaining({
          'Cookie': `sess_id=${maliciousSessionId}`
        })
      })
    );
  });

  test('prevents unauthorized route access', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        route_permissions: {
          '/admin': { allowed: false, permissions: [] }
        }
      })
    } as Response);

    const request = new NextRequest('http://localhost:3000/admin');
    const response = await middleware(request);

    expect(response.status).toBe(307); // Redirect
    expect(response.headers.get('Location')).toContain('/unauthorized');
  });
});
```

#### 1.2 Permission Security Tests
```rust
#[tokio::test]
async fn test_permission_escalation_prevention() {
    let permission_checker = setup_permission_checker().await;
    let regular_user_id = create_regular_user().await.id;

    // Attempt to check admin permission
    let result = permission_checker.check_permission_with_context(
        regular_user_id,
        "admin:users:delete",
        None,
        None,
    ).await.unwrap();

    // Should be denied
    assert!(!result.allowed);
    assert_eq!(result.reason, PermissionReason::NoPermission);
}

#[tokio::test]
async fn test_wildcard_permission_security() {
    let permission_checker = setup_permission_checker().await;
    let user_id = create_test_user_with_permissions(vec!["dashboard:*"]).await.id;

    // Should allow dashboard permissions
    let dashboard_result = permission_checker.check_permission_with_context(
        user_id,
        "dashboard:view",
        None,
        None,
    ).await.unwrap();
    assert!(dashboard_result.allowed);

    // Should NOT allow admin permissions (wildcard should be scoped)
    let admin_result = permission_checker.check_permission_with_context(
        user_id,
        "admin:users",
        None,
        None,
    ).await.unwrap();
    assert!(!admin_result.allowed);
}
```

## Regression Testing

### Functional Regression Tests

#### 1.1 User Journey Tests
```typescript
describe('User Journey Regression Tests', () => {
  test('complete user journey works end-to-end', async () => {
    // Mock API responses for complete flow
    mockFetch
      // Login
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, user: { id: '1', email: 'test@example.com' } })
      })
      // Session validation
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ 
          authenticated: true, 
          user: { id: '1', email: 'test@example.com' },
          permissions: ['dashboard:view', 'analytics:view']
        })
      })
      // Route validation
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          route_permissions: {
            '/dashboard': { allowed: true },
            '/analytics': { allowed: true },
            '/admin': { allowed: false }
          }
        })
      })
      // Permission check
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          results: [{ permission: 'analytics:view', allowed: true }]
        })
      });

    const { container } = render(<App />);

    // 1. User logs in
    fireEvent.click(screen.getByText('Login'));
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'test@example.com' }
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password' }
    });
    fireEvent.click(screen.getByText('Sign In'));

    // 2. User navigates to dashboard
    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    });

    // 3. User tries to access analytics (should work)
    fireEvent.click(screen.getByText('Analytics'));
    await waitFor(() => {
      expect(screen.getByText('Analytics Dashboard')).toBeInTheDocument();
    });

    // 4. User tries to access admin (should be blocked)
    fireEvent.click(screen.getByText('Admin'));
    await waitFor(() => {
      expect(screen.getByText('Access Denied')).toBeInTheDocument();
    });

    // Verify all expected API calls were made
    expect(mockFetch).toHaveBeenCalledTimes(4);
  });
});
```

#### 1.2 Admin Flow Regression Tests
```typescript
describe('Admin Flow Regression Tests', () => {
  test('admin user can access both admin and frontend features', async () => {
    // Mock admin user responses
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ 
          success: true, 
          user: { id: 'admin-1', email: 'admin@example.com', roles: ['admin'] }
        })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          authenticated: true,
          user: { id: 'admin-1', email: 'admin@example.com' },
          permissions: ['admin:*', 'dashboard:view'],
          session_type: 'Admin',
          capabilities: ['frontend_access']
        })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          route_permissions: {
            '/admin': { allowed: true },
            '/admin/users': { allowed: true },
            '/dashboard': { allowed: true }
          }
        })
      });

    render(<AdminApp />);

    // Admin login
    fireEvent.click(screen.getByText('Admin Login'));
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'admin@example.com' }
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'admin-password' }
    });
    fireEvent.click(screen.getByText('Sign In'));

    // Should access admin dashboard
    await waitFor(() => {
      expect(screen.getByText('Admin Dashboard')).toBeInTheDocument();
    });

    // Should be able to manage users
    fireEvent.click(screen.getByText('User Management'));
    await waitFor(() => {
      expect(screen.getByText('User Management Dashboard')).toBeInTheDocument();
    });

    // Should also be able to access frontend features
    fireEvent.click(screen.getByText('Frontend Dashboard'));
    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    });
  });
});
```

## Test Data Management

### Test Data Setup

#### 1.1 Database Test Data
```rust
pub async fn setup_test_data() -> TestContext {
    let pool = setup_test_database().await;
    
    // Create test users
    let regular_user = User {
        id: UserId::new("regular-user-1"),
        email: "regular@example.com".to_string(),
        roles: vec!["user".to_string()],
        permission_profiles: vec!["Silver".to_string()],
        created_at: Utc::now(),
    };
    
    let admin_user = User {
        id: UserId::new("admin-user-1"),
        email: "admin@example.com".to_string(),
        roles: vec!["admin".to_string()],
        permission_profiles: vec!["Admin".to_string()],
        created_at: Utc::now(),
    };
    
    // Insert test data
    insert_test_user(&pool, &regular_user).await;
    insert_test_user(&pool, &admin_user).await;
    
    // Create test permissions
    create_test_permissions(&pool).await;
    
    TestContext {
        pool,
        regular_user,
        admin_user,
    }
}

async fn create_test_permissions(pool: &PgPool) {
    let permissions = vec![
        "dashboard:view",
        "dashboard:edit",
        "analytics:view",
        "analytics:export",
        "admin:users:view",
        "admin:users:edit",
        "admin:users:delete",
        "admin:analytics:view",
        "admin:system:manage",
    ];
    
    for permission in permissions {
        sqlx::query!(
            "INSERT INTO permissions (name, description) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
            permission,
            format!("Test permission: {}", permission)
        )
        .execute(pool)
        .await
        .unwrap();
    }
}
```

#### 1.2 Frontend Test Data
```typescript
export const createMockApiResponses = () => {
  const regularUserSession = {
    authenticated: true,
    user: {
      id: 'regular-user-1',
      email: 'regular@example.com',
      roles: ['user'],
      permission_profiles: ['Silver']
    },
    permissions: ['dashboard:view', 'analytics:view'],
    session_type: 'Regular'
  };

  const adminUserSession = {
    authenticated: true,
    user: {
      id: 'admin-user-1',
      email: 'admin@example.com',
      roles: ['admin'],
      permission_profiles: ['Admin']
    },
    permissions: ['admin:*', 'dashboard:view', 'analytics:view'],
    session_type: 'Admin',
    capabilities: ['frontend_access']
  };

  const routePermissions = {
    '/dashboard': { allowed: true, permissions: ['dashboard:view'] },
    '/analytics': { allowed: true, permissions: ['analytics:view'] },
    '/admin': { allowed: false, permissions: ['admin:access'] },
    '/admin/users': { allowed: false, permissions: ['admin:users:view'] }
  };

  return {
    regularUserSession,
    adminUserSession,
    routePermissions
  };
};
```

## Continuous Testing

### CI/CD Integration

#### 1.1 Automated Test Pipeline
```yaml
# .github/workflows/auth-migration-tests.yml
name: Auth Migration Tests

on:
  push:
    branches: [features/improve-roles/iam/acl]
  pull_request:
    branches: [development]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: epsx_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run Backend Tests
        run: |
          cd apps/backend
          cargo test --lib auth::
          cargo test --lib session::
          cargo test --lib permission::
        env:
          DATABASE_URL: postgres://postgres:test@localhost/epsx_test

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install Dependencies
        run: |
          cd apps/frontend
          npm install
          
      - name: Run Frontend Tests
        run: |
          cd apps/frontend
          npm test -- --coverage --watchAll=false
          
      - name: Run Admin Frontend Tests
        run: |
          cd apps/admin-frontend
          npm test -- --coverage --watchAll=false

  integration-tests:
    runs-on: ubuntu-latest
    needs: [backend-tests, frontend-tests]
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Test Environment
        run: |
          docker-compose -f docker-compose.test.yml up -d
          
      - name: Run Integration Tests
        run: |
          npm run test:integration
          
      - name: Run E2E Tests
        run: |
          npm run test:e2e

  performance-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Performance Tests
        run: |
          npm run test:performance
          
      - name: Upload Performance Results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: performance-results.json
```

## Test Reporting

### Coverage and Quality Metrics

#### 1.1 Test Coverage Requirements
- **Backend**: >90% line coverage for all auth-related modules
- **Frontend**: >85% line coverage for auth components and hooks
- **Integration**: >80% coverage of auth user journeys
- **E2E**: 100% coverage of critical auth flows

#### 1.2 Quality Gates
- All tests must pass before merge
- No security vulnerabilities in dependencies
- Performance benchmarks must be met
- No regression in user experience metrics

## Test Documentation

### Test Case Documentation
Each test should include:
- **Purpose**: What the test validates
- **Prerequisites**: Required test data and setup
- **Steps**: Detailed test steps
- **Expected Results**: What should happen
- **Cleanup**: How to clean up after the test

### Test Data Documentation
- Document all test users and their permissions
- Explain test scenarios and data relationships
- Provide setup and teardown procedures
- Include troubleshooting guides for common test failures

This comprehensive testing strategy ensures that the backend-centralized auth migration maintains all existing functionality while improving security, performance, and maintainability.