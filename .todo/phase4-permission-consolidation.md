# Phase 4: Permission Consolidation

## Overview
Completely centralize all permission validation and checking logic in the backend, removing all frontend permission logic and implementing real-time permission updates.

## Backend Permission System Enhancement

### 1. Centralized Permission API
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 1.1 Comprehensive Permission Endpoints
- [ ] Create comprehensive permission validation API
- [ ] Implement real-time permission updates
- [ ] Add permission caching with invalidation

```rust
// Enhanced permission validation endpoint
pub async fn validate_permissions_handler(
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_checker): Extension<Arc<PermissionChecker>>,
    Json(request): Json<PermissionValidationRequest>,
) -> Result<Json<PermissionValidationResponse>, AuthError> {
    let mut results = Vec::new();
    
    for permission_check in request.permissions {
        let result = permission_checker.check_permission(
            auth_context.user_id,
            &permission_check.permission,
            permission_check.resource.as_deref(),
            permission_check.context.as_ref(),
        ).await?;
        
        results.push(PermissionResult {
            permission: permission_check.permission,
            allowed: result.allowed,
            reason: result.reason,
            expires_at: result.expires_at,
        });
    }
    
    Ok(Json(PermissionValidationResponse {
        user_id: auth_context.user_id,
        results,
        cached_until: Utc::now() + Duration::minutes(5),
    }))
}

// Bulk route permission validation
pub async fn validate_route_permissions_handler(
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_checker): Extension<Arc<PermissionChecker>>,
    Json(request): Json<RoutePermissionRequest>,
) -> Result<Json<RoutePermissionResponse>, AuthError> {
    let mut route_permissions = HashMap::new();
    
    for route in request.routes {
        let access_result = permission_checker.validate_route_access(
            auth_context.user_id,
            &route.path,
            &route.method,
            route.app_type,
        ).await?;
        
        route_permissions.insert(route.path, RouteAccessInfo {
            allowed: access_result.allowed,
            required_permissions: access_result.required_permissions,
            user_permissions: access_result.user_permissions,
            permission_level: access_result.permission_level,
            rate_limit_info: access_result.rate_limit_info,
        });
    }
    
    Ok(Json(RoutePermissionResponse {
        route_permissions,
        user_level: auth_context.user_level,
        cached_until: Utc::now() + Duration::minutes(5),
    }))
}

// Real-time permission status
pub async fn permission_status_handler(
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_checker): Extension<Arc<PermissionChecker>>,
) -> Result<Json<PermissionStatusResponse>, AuthError> {
    let permissions = permission_checker.get_all_user_permissions(auth_context.user_id).await?;
    let roles = permission_checker.get_user_roles(auth_context.user_id).await?;
    let permission_profiles = permission_checker.get_user_permission_profiles(auth_context.user_id).await?;
    
    Ok(Json(PermissionStatusResponse {
        permissions,
        roles,
        permission_profiles,
        capabilities: auth_context.capabilities,
        last_updated: Utc::now(),
    }))
}
```

#### 1.2 Permission Change Notification API
- [ ] WebSocket endpoint for real-time permission updates
- [ ] Server-sent events for permission changes
- [ ] Permission invalidation broadcasting

```rust
// WebSocket handler for real-time permission updates
pub async fn permission_updates_websocket(
    ws: WebSocketUpgrade,
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_notifier): Extension<Arc<PermissionNotifier>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_permission_updates(socket, auth_context, permission_notifier))
}

async fn handle_permission_updates(
    mut socket: WebSocket,
    auth_context: AuthContext,
    permission_notifier: Arc<PermissionNotifier>,
) {
    let mut receiver = permission_notifier.subscribe_to_user_updates(auth_context.user_id).await;
    
    while let Ok(update) = receiver.recv().await {
        let message = serde_json::to_string(&PermissionUpdateMessage {
            update_type: update.update_type,
            affected_permissions: update.affected_permissions,
            new_permissions: update.new_permissions,
            timestamp: update.timestamp,
        }).unwrap();
        
        if socket.send(Message::Text(message)).await.is_err() {
            break;
        }
    }
}

// Server-sent events alternative
pub async fn permission_updates_sse(
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_notifier): Extension<Arc<PermissionNotifier>>,
) -> impl IntoResponse {
    let stream = permission_notifier.subscribe_to_user_updates(auth_context.user_id).await;
    
    Sse::new(stream.map(|update| {
        Event::default()
            .event("permission_update")
            .data(serde_json::to_string(&update).unwrap())
    }))
}
```

### 2. Enhanced Permission Checker Service
**Location**: `apps/backend/src/dom/services/permission_checker.rs`

#### 2.1 Advanced Permission Validation
- [ ] Implement context-aware permission checking
- [ ] Add time-based permission validation
- [ ] Create permission inheritance system

```rust
impl PermissionChecker {
    pub async fn check_permission_with_context(
        &self,
        user_id: UserId,
        permission: &str,
        resource: Option<&str>,
        context: Option<&serde_json::Value>,
    ) -> Result<PermissionCheckResult, PermissionError> {
        // Load user's complete permission set
        let user_permissions = self.load_user_permissions_with_inheritance(user_id).await?;
        
        // Check direct permission
        if self.has_direct_permission(&user_permissions, permission) {
            return Ok(PermissionCheckResult {
                allowed: true,
                reason: PermissionReason::DirectPermission,
                expires_at: None,
            });
        }
        
        // Check wildcard permissions
        if let Some(wildcard_match) = self.check_wildcard_permissions(&user_permissions, permission) {
            return Ok(PermissionCheckResult {
                allowed: true,
                reason: PermissionReason::WildcardMatch(wildcard_match),
                expires_at: None,
            });
        }
        
        // Check role-based permissions
        if let Some(role_permission) = self.check_role_permissions(user_id, permission).await? {
            return Ok(PermissionCheckResult {
                allowed: true,
                reason: PermissionReason::RoleBasedPermission(role_permission),
                expires_at: None,
            });
        }
        
        // Check context-specific permissions
        if let Some(context) = context {
            if let Some(context_permission) = self.check_context_permissions(
                user_id, 
                permission, 
                resource, 
                context
            ).await? {
                return Ok(PermissionCheckResult {
                    allowed: true,
                    reason: PermissionReason::ContextSpecific(context_permission),
                    expires_at: context_permission.expires_at,
                });
            }
        }
        
        // Check time-based permissions
        if let Some(time_permission) = self.check_time_based_permissions(user_id, permission).await? {
            return Ok(PermissionCheckResult {
                allowed: time_permission.is_active(),
                reason: PermissionReason::TimeBased(time_permission.clone()),
                expires_at: Some(time_permission.expires_at),
            });
        }
        
        Ok(PermissionCheckResult {
            allowed: false,
            reason: PermissionReason::NoPermission,
            expires_at: None,
        })
    }
    
    async fn load_user_permissions_with_inheritance(
        &self,
        user_id: UserId,
    ) -> Result<UserPermissionSet, PermissionError> {
        // Load direct permissions
        let direct_permissions = self.iam_repo.get_user_permissions(user_id).await?;
        
        // Load role-based permissions
        let roles = self.iam_repo.get_user_roles(user_id).await?;
        let role_permissions = self.load_role_permissions(&roles).await?;
        
        // Load group-based permissions
        let groups = self.iam_repo.get_user_groups(user_id).await?;
        let group_permissions = self.load_group_permissions(&groups).await?;
        
        // Load permission profile permissions
        let permission_profiles = self.iam_repo.get_user_permission_profiles(user_id).await?;
        let profile_permissions = self.load_profile_permissions(&permission_profiles).await?;
        
        Ok(UserPermissionSet {
            direct_permissions,
            role_permissions,
            group_permissions,
            profile_permissions,
            computed_at: Utc::now(),
        })
    }
    
    pub async fn validate_bulk_permissions(
        &self,
        user_id: UserId,
        permissions: Vec<PermissionCheck>,
    ) -> Result<Vec<PermissionResult>, PermissionError> {
        let mut results = Vec::new();
        
        // Load permissions once for efficiency
        let user_permissions = self.load_user_permissions_with_inheritance(user_id).await?;
        
        for permission_check in permissions {
            let result = self.check_permission_from_set(
                &user_permissions,
                &permission_check.permission,
                permission_check.resource.as_deref(),
                permission_check.context.as_ref(),
            ).await?;
            
            results.push(PermissionResult {
                permission: permission_check.permission,
                allowed: result.allowed,
                reason: result.reason,
                expires_at: result.expires_at,
            });
        }
        
        Ok(results)
    }
}
```

#### 2.2 Permission Caching System
- [ ] Implement Redis-based permission caching
- [ ] Add cache invalidation on permission changes
- [ ] Optimize for high-frequency permission checks

```rust
pub struct PermissionCache {
    redis_client: Arc<redis::Client>,
    cache_ttl: Duration,
}

impl PermissionCache {
    pub async fn get_cached_permissions(
        &self,
        user_id: UserId,
    ) -> Result<Option<CachedPermissionSet>, CacheError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("permissions:user:{}", user_id);
        
        let cached_data: Option<String> = conn.get(&cache_key).await?;
        
        if let Some(data) = cached_data {
            let permissions: CachedPermissionSet = serde_json::from_str(&data)?;
            
            // Check if cache is still valid
            if permissions.expires_at > Utc::now() {
                return Ok(Some(permissions));
            }
        }
        
        Ok(None)
    }
    
    pub async fn cache_permissions(
        &self,
        user_id: UserId,
        permissions: &UserPermissionSet,
    ) -> Result<(), CacheError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("permissions:user:{}", user_id);
        
        let cached_permissions = CachedPermissionSet {
            permissions: permissions.clone(),
            cached_at: Utc::now(),
            expires_at: Utc::now() + self.cache_ttl,
        };
        
        let serialized = serde_json::to_string(&cached_permissions)?;
        
        conn.setex(&cache_key, self.cache_ttl.num_seconds() as usize, serialized).await?;
        
        Ok(())
    }
    
    pub async fn invalidate_user_permissions(
        &self,
        user_id: UserId,
    ) -> Result<(), CacheError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("permissions:user:{}", user_id);
        
        conn.del(&cache_key).await?;
        
        Ok(())
    }
    
    pub async fn invalidate_permissions_by_pattern(
        &self,
        pattern: &str,
    ) -> Result<(), CacheError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if !keys.is_empty() {
            conn.del(keys).await?;
        }
        
        Ok(())
    }
}
```

### 3. Real-time Permission Updates
**Location**: `apps/backend/src/dom/services/permission_notifier.rs`

#### 3.1 Permission Change Notification System
- [ ] Create permission change event system
- [ ] Implement real-time notification broadcasting
- [ ] Add event persistence for reliability

```rust
pub struct PermissionNotifier {
    event_bus: Arc<EventBus>,
    subscribers: Arc<RwLock<HashMap<UserId, Vec<Sender<PermissionUpdate>>>>>,
    redis_client: Arc<redis::Client>,
}

impl PermissionNotifier {
    pub async fn notify_permission_change(
        &self,
        change: PermissionChange,
    ) -> Result<(), NotificationError> {
        // Create permission update event
        let update = PermissionUpdate {
            update_type: change.change_type,
            affected_users: change.affected_users.clone(),
            affected_permissions: change.affected_permissions,
            new_permissions: change.new_permissions,
            timestamp: Utc::now(),
        };
        
        // Broadcast to WebSocket subscribers
        for user_id in &change.affected_users {
            self.send_to_user_subscribers(*user_id, &update).await?;
        }
        
        // Publish to Redis for cross-instance notification
        self.publish_to_redis(&update).await?;
        
        // Invalidate cached permissions
        for user_id in &change.affected_users {
            self.invalidate_user_cache(*user_id).await?;
        }
        
        Ok(())
    }
    
    pub async fn subscribe_to_user_updates(
        &self,
        user_id: UserId,
    ) -> Receiver<PermissionUpdate> {
        let (sender, receiver) = channel(100);
        
        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(sender);
        
        receiver
    }
    
    async fn send_to_user_subscribers(
        &self,
        user_id: UserId,
        update: &PermissionUpdate,
    ) -> Result<(), NotificationError> {
        let subscribers = self.subscribers.read().await;
        
        if let Some(user_subscribers) = subscribers.get(&user_id) {
            for sender in user_subscribers {
                if sender.send(update.clone()).await.is_err() {
                    // Subscriber disconnected, should clean up
                }
            }
        }
        
        Ok(())
    }
    
    async fn publish_to_redis(
        &self,
        update: &PermissionUpdate,
    ) -> Result<(), NotificationError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let channel = "permission_updates";
        let message = serde_json::to_string(update)?;
        
        conn.publish(channel, message).await?;
        
        Ok(())
    }
}
```

## Frontend Permission Logic Removal

### 1. Remove Frontend Permission Checking
**Location**: `apps/frontend/hooks/usePermissions.ts`

#### 1.1 Replace with API Calls
- [ ] Remove all client-side permission logic
- [ ] Replace with backend API calls
- [ ] Add real-time permission updates

```typescript
// REMOVE: Complex client-side permission logic
export function usePermissions() {
  const [permissions, setPermissions] = useState<string[]>([]);
  // ... complex logic to remove
}

// REPLACE WITH: API-based permission system
export function usePermissions() {
  const { data: permissionStatus, mutate } = useSWR(
    '/api/v1/auth/permission-status',
    fetcher,
    { refreshInterval: 30000 } // Refresh every 30 seconds
  );
  
  // Real-time permission updates via WebSocket
  useEffect(() => {
    const ws = new WebSocket(`${WS_URL}/api/v1/auth/permission-updates`);
    
    ws.onmessage = (event) => {
      const update = JSON.parse(event.data) as PermissionUpdate;
      
      // Invalidate SWR cache to trigger refresh
      mutate();
      
      // Optionally show notification about permission changes
      if (update.update_type === 'REVOKED') {
        toast.warn('Some of your permissions have been updated');
      }
    };
    
    return () => ws.close();
  }, [mutate]);
  
  const checkPermission = useCallback(async (permission: string, context?: any) => {
    const response = await fetch('/api/v1/auth/validate-permissions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        permissions: [{ permission, context }]
      })
    });
    
    const result = await response.json();
    return result.results[0]?.allowed || false;
  }, []);
  
  const checkMultiplePermissions = useCallback(async (permissions: PermissionCheck[]) => {
    const response = await fetch('/api/v1/auth/validate-permissions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ permissions })
    });
    
    const result = await response.json();
    return result.results;
  }, []);
  
  return {
    permissions: permissionStatus?.permissions || [],
    roles: permissionStatus?.roles || [],
    permissionProfiles: permissionStatus?.permission_profiles || [],
    checkPermission,
    checkMultiplePermissions,
    loading: !permissionStatus,
  };
}
```

### 2. Remove Admin Permission Logic
**Location**: `apps/admin-frontend/hooks/useFeatureAccess.ts`

#### 2.1 Align with Frontend Permission System
- [ ] Remove admin-specific permission logic
- [ ] Use same API-based system as frontend
- [ ] Maintain admin context

```typescript
// Use same permission system as frontend
export function useAdminPermissions() {
  const basePermissions = usePermissions();
  
  // Add admin-specific helpers
  const hasAdminAccess = useCallback((resource: string) => {
    return basePermissions.checkPermission(`admin:${resource}`);
  }, [basePermissions]);
  
  const canManageUsers = useCallback(() => {
    return basePermissions.checkPermission('admin:users:manage');
  }, [basePermissions]);
  
  const canViewAnalytics = useCallback(() => {
    return basePermissions.checkPermission('admin:analytics:view');
  }, [basePermissions]);
  
  return {
    ...basePermissions,
    hasAdminAccess,
    canManageUsers,
    canViewAnalytics,
  };
}
```

### 3. Remove Permission-based Component Logic
**Location**: Various component files

#### 3.1 Replace Permission Guards with API Calls
- [ ] Remove `<PermissionGuard>` complex logic
- [ ] Replace with simple API-based checks
- [ ] Add loading states for permission checks

```typescript
// REMOVE: Complex permission guard component
interface PermissionGuardProps {
  permission: string;
  fallback?: React.ReactNode;
  children: React.ReactNode;
}

export function PermissionGuard({ permission, fallback, children }: PermissionGuardProps) {
  // Complex client-side logic to remove
}

// REPLACE WITH: Simple API-based guard
export function PermissionGuard({ permission, fallback, children }: PermissionGuardProps) {
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  
  useEffect(() => {
    fetch('/api/v1/auth/validate-permissions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        permissions: [{ permission }]
      })
    })
    .then(response => response.json())
    .then(result => {
      setHasPermission(result.results[0]?.allowed || false);
    })
    .catch(() => setHasPermission(false));
  }, [permission]);
  
  if (hasPermission === null) {
    return <div>Checking permissions...</div>;
  }
  
  if (!hasPermission) {
    return fallback || <div>Access denied</div>;
  }
  
  return <>{children}</>;
}
```

### 4. Remove Route-based Permission Logic
**Location**: `apps/frontend/middleware.ts` and `apps/admin-frontend/middleware.ts`

#### 4.1 Simplify to Backend API Calls
- [ ] Remove complex route-to-permission mapping
- [ ] Remove permission validation cache
- [ ] Use bulk route validation API

```typescript
// REMOVE: Complex middleware permission logic
const routePermissions = {
  '/dashboard': ['dashboard:view'],
  '/analytics': ['analytics:view'],
  // ... complex mapping to remove
};

// REPLACE WITH: Simple API-based validation
export async function middleware(request: NextRequest) {
  const pathname = request.nextUrl.pathname;
  
  // Get commonly accessed routes for bulk validation
  const commonRoutes = [
    pathname,
    '/dashboard',
    '/analytics',
    '/settings'
  ].filter((route, index, self) => self.indexOf(route) === index);
  
  try {
    const response = await fetch('/api/v1/auth/validate-route-permissions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': request.headers.get('cookie') || ''
      },
      body: JSON.stringify({
        routes: commonRoutes.map(path => ({
          path,
          method: 'GET',
          app_type: 'Frontend' // or 'Admin' for admin frontend
        }))
      })
    });
    
    const result = await response.json();
    const currentRouteAccess = result.route_permissions[pathname];
    
    if (!currentRouteAccess?.allowed) {
      return NextResponse.redirect(new URL('/unauthorized', request.url));
    }
    
    // Cache route permissions in header for client use
    const responseHeaders = new Headers();
    responseHeaders.set('X-Route-Permissions', JSON.stringify(result.route_permissions));
    
    return NextResponse.next({
      headers: responseHeaders
    });
    
  } catch (error) {
    console.error('Permission validation failed:', error);
    return NextResponse.redirect(new URL('/login', request.url));
  }
}
```

## Performance Optimization

### 1. Permission Caching Strategy
**Location**: Backend and Frontend

#### 1.1 Multi-level Caching
- [ ] Redis cache for backend permission lookups
- [ ] Frontend memory cache for UI decisions
- [ ] Cache invalidation on permission changes

```rust
// Backend caching strategy
impl PermissionChecker {
    async fn get_permissions_with_cache(
        &self,
        user_id: UserId,
    ) -> Result<UserPermissionSet, PermissionError> {
        // Try cache first
        if let Some(cached) = self.cache.get_cached_permissions(user_id).await? {
            return Ok(cached.permissions);
        }
        
        // Load from database
        let permissions = self.load_user_permissions_with_inheritance(user_id).await?;
        
        // Cache for future use
        self.cache.cache_permissions(user_id, &permissions).await?;
        
        Ok(permissions)
    }
}
```

```typescript
// Frontend caching strategy
class PermissionCache {
  private cache = new Map<string, CachedPermission>();
  private readonly TTL = 5 * 60 * 1000; // 5 minutes
  
  get(key: string): boolean | null {
    const cached = this.cache.get(key);
    if (cached && Date.now() < cached.expiresAt) {
      return cached.allowed;
    }
    this.cache.delete(key);
    return null;
  }
  
  set(key: string, allowed: boolean): void {
    this.cache.set(key, {
      allowed,
      expiresAt: Date.now() + this.TTL
    });
  }
  
  invalidate(pattern?: string): void {
    if (pattern) {
      for (const key of this.cache.keys()) {
        if (key.includes(pattern)) {
          this.cache.delete(key);
        }
      }
    } else {
      this.cache.clear();
    }
  }
}
```

### 2. Batch Permission Loading
**Location**: Frontend components

#### 2.1 Optimize Permission Checks
- [ ] Batch multiple permission checks
- [ ] Preload permissions for likely routes
- [ ] Minimize API calls

```typescript
export function useRoutePermissions(routes: string[]) {
  const { data, error } = useSWR(
    ['route-permissions', routes],
    () => fetch('/api/v1/auth/validate-route-permissions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        routes: routes.map(path => ({
          path,
          method: 'GET',
          app_type: 'Frontend'
        }))
      })
    }).then(r => r.json()),
    {
      revalidateOnFocus: false,
      refreshInterval: 5 * 60 * 1000, // 5 minutes
    }
  );
  
  const hasRouteAccess = useCallback((route: string) => {
    return data?.route_permissions[route]?.allowed || false;
  }, [data]);
  
  return {
    routePermissions: data?.route_permissions || {},
    hasRouteAccess,
    loading: !data && !error,
    error
  };
}
```

## Audit and Monitoring

### 1. Permission Audit System
**Location**: `apps/backend/src/dom/services/audit_service.rs`

#### 1.1 Comprehensive Permission Auditing
- [ ] Log all permission checks
- [ ] Track permission changes
- [ ] Monitor for unusual patterns

```rust
impl AuditService {
    pub async fn log_permission_check(
        &self,
        event: PermissionCheckEvent,
    ) -> Result<(), AuditError> {
        let audit_entry = AuditEntry {
            id: Uuid::new_v4(),
            user_id: Some(event.user_id),
            action: AuditAction::PermissionCheck,
            resource: Some(format!("permission:{}", event.permission)),
            details: serde_json::json!({
                "permission": event.permission,
                "allowed": event.allowed,
                "reason": event.reason,
                "context": event.context,
                "ip_address": event.ip_address,
                "user_agent": event.user_agent,
            }),
            timestamp: Utc::now(),
        };
        
        self.audit_repo.store_audit_entry(audit_entry).await?;
        
        // Check for suspicious patterns
        self.detect_suspicious_permission_activity(event.user_id, &event.permission).await?;
        
        Ok(())
    }
    
    async fn detect_suspicious_permission_activity(
        &self,
        user_id: UserId,
        permission: &str,
    ) -> Result<(), AuditError> {
        // Check for rapid permission checking (possible attack)
        let recent_checks = self.audit_repo.get_recent_permission_checks(
            user_id,
            Duration::minutes(5)
        ).await?;
        
        if recent_checks.len() > 100 {
            self.alert_service.send_security_alert(SecurityAlert {
                alert_type: AlertType::SuspiciousPermissionActivity,
                user_id: Some(user_id),
                details: format!("User performed {} permission checks in 5 minutes", recent_checks.len()),
                severity: AlertSeverity::High,
            }).await?;
        }
        
        Ok(())
    }
}
```

## Testing Requirements

### 1. Comprehensive Permission Testing
**Location**: `apps/backend/__tests__/`

#### 1.1 Permission System Tests
- [ ] Test all permission validation scenarios
- [ ] Test real-time permission updates
- [ ] Test caching and invalidation
- [ ] Test security and audit features

```rust
#[cfg(test)]
mod permission_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_permission_validation_with_context() {
        let permission_checker = setup_permission_checker().await;
        let user_id = create_test_user().await;
        
        // Test direct permission
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
    async fn test_real_time_permission_updates() {
        let notifier = setup_permission_notifier().await;
        let user_id = create_test_user().await;
        
        let mut receiver = notifier.subscribe_to_user_updates(user_id).await;
        
        // Trigger permission change
        notifier.notify_permission_change(PermissionChange {
            change_type: PermissionChangeType::GRANTED,
            affected_users: vec![user_id],
            affected_permissions: vec!["new:permission".to_string()],
            new_permissions: vec!["new:permission".to_string()],
        }).await.unwrap();
        
        // Verify notification received
        let update = receiver.recv().await.unwrap();
        assert_eq!(update.update_type, PermissionUpdateType::GRANTED);
        assert!(update.affected_permissions.contains(&"new:permission".to_string()));
    }
}
```

### 2. Frontend Integration Tests  
**Location**: `apps/frontend/__tests__/`

#### 2.1 API Integration Tests
- [ ] Test permission hook functionality
- [ ] Test real-time updates in UI
- [ ] Test error handling and fallbacks

```typescript
describe('Permission System Integration', () => {
  test('usePermissions hook fetches permissions from API', async () => {
    // Mock API response
    fetchMock.mockResponseOnce(JSON.stringify({
      permissions: ['dashboard:view', 'analytics:view'],
      roles: ['user'],
      permission_profiles: ['Silver'],
    }));
    
    const { result, waitForNextUpdate } = renderHook(() => usePermissions());
    
    await waitForNextUpdate();
    
    expect(result.current.permissions).toContain('dashboard:view');
    expect(result.current.permissions).toContain('analytics:view');
  });
  
  test('real-time permission updates work', async () => {
    const mockWebSocket = new MockWebSocket();
    global.WebSocket = jest.fn(() => mockWebSocket);
    
    const { result } = renderHook(() => usePermissions());
    
    // Simulate permission update
    mockWebSocket.onmessage({
      data: JSON.stringify({
        update_type: 'GRANTED',
        affected_permissions: ['new:permission'],
        timestamp: new Date().toISOString(),
      })
    });
    
    // Should trigger SWR revalidation
    await waitFor(() => {
      expect(result.current.permissions).toContain('new:permission');
    });
  });
});
```

## Migration Strategy

### 1.1 Gradual Permission Migration
- [ ] Feature flag for new permission system
- [ ] Parallel run old and new systems
- [ ] Gradual user migration
- [ ] Monitor for regressions

### 1.2 Data Migration
- [ ] Migrate existing permission data
- [ ] Update audit logs format
- [ ] Ensure zero downtime migration

## Dependencies
- **Requires**: Phase 1, 2, and 3 completed
- **Blocks**: None (final phase)

## Completion Criteria
- [ ] All permission logic centralized in backend
- [ ] Real-time permission updates working
- [ ] Frontend permission logic completely removed
- [ ] Comprehensive caching system implemented
- [ ] Audit and monitoring in place
- [ ] All tests passing (>95% coverage)
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] Zero regression in functionality
- [ ] Real-time updates working reliably