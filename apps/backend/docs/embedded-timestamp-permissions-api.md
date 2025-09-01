# Embedded Timestamp Permissions API Documentation

## Overview

The EPSX platform supports **embedded timestamp permissions** that allow for time-limited access control. This API documentation covers all endpoints for managing permissions with embedded expiry timestamps.

**Permission Format**: `"platform:resource:action:unix_timestamp"`

### Examples
- **Permanent Permission**: `"epsx:analytics:view"` (never expires)
- **Temporary Permission**: `"epsx:analytics:view:1735689600"` (expires at Unix timestamp)
- **Mixed Support**: Users can have both permanent and temporary permissions

## Authentication

All endpoints require admin authentication with appropriate permissions:
- **Header**: `Authorization: Bearer <jwt_token>`
- **Required Permissions**: `admin:users:manage` or `admin:*:*`

## Endpoints

### 1. Grant Embedded Permission

Grant a timestamped permission to a specific user.

**Endpoint**: `POST /api/v1/admin/users/{user_id}/embedded-permissions`

**Parameters**:
- `user_id` (path): User's Firebase UID or internal user ID

**Request Body**:
```json
{
  "embedded_permission": "epsx:analytics:view:1735689600",
  "base_permission": "epsx:analytics:view",
  "platform": "epsx",
  "resource": "analytics",
  "action": "view",
  "expiry_timestamp": 1735689600,
  "reason": "Temporary analytics access for report generation",
  "metadata": {
    "granted_for": "Q4_2024_report",
    "department": "finance"
  }
}
```

**Response** (200 OK):
```json
{
  "permission": "epsx:analytics:view:1735689600",
  "expires_at": 1735689600
}
```

**Error Responses**:
- `400 Bad Request`: Invalid request body or user ID
- `401 Unauthorized`: Missing or invalid authentication
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: User not found
- `500 Internal Server Error`: Database or server error

---

### 2. Bulk Grant Embedded Permissions

Grant embedded permissions to multiple users simultaneously.

**Endpoint**: `POST /api/v1/admin/users/bulk/embedded-permissions`

**Request Body**:
```json
{
  "user_ids": ["user-1", "user-2", "user-3"],
  "permissions": [
    {
      "base_permission": "epsx:analytics:view",
      "platform": "epsx",
      "resource": "analytics",
      "action": "view",
      "expiry_timestamp": 1735689600
    },
    {
      "base_permission": "epsx:rankings:view:25",
      "platform": "epsx",
      "resource": "rankings",
      "action": "view",
      "expiry_timestamp": 1735689600
    }
  ],
  "reason": "Bulk permission grant for team analytics access",
  "metadata": {
    "batch_id": "batch_2024_001",
    "project": "Q4_analytics"
  }
}
```

**Response** (200 OK):
```json
{
  "successful": [
    {
      "user_id": "user-1",
      "permissions": [
        "epsx:analytics:view:1735689600",
        "epsx:rankings:view:25:1735689600"
      ]
    },
    {
      "user_id": "user-2", 
      "permissions": [
        "epsx:analytics:view:1735689600",
        "epsx:rankings:view:25:1735689600"
      ]
    }
  ],
  "failed": [
    {
      "user_id": "user-3",
      "error": "User not found"
    }
  ],
  "summary": {
    "total": 3,
    "successful": 2,
    "failed": 1
  }
}
```

---

### 3. Validate Embedded Permissions

Validate permissions for expiry status and filter out expired ones.

**Endpoint**: `POST /api/v1/admin/users/{user_id}/embedded-permissions/validate`

**Request Body**:
```json
{
  "permissions": [
    "epsx:analytics:view",
    "epsx:rankings:view:25:1735689600",
    "admin:users:manage:1703980800"
  ]
}
```

**Response** (200 OK):
```json
{
  "valid": [
    "epsx:analytics:view",
    "epsx:rankings:view:25:1735689600"
  ],
  "expired": [
    {
      "permission": "admin:users:manage:1703980800",
      "base_permission": "admin:users:manage", 
      "expired_at": 1703980800,
      "expired_for": 2678400000
    }
  ],
  "expiring_soon": [
    {
      "permission": "epsx:rankings:view:25:1735689600",
      "base_permission": "epsx:rankings:view:25",
      "expires_at": 1735689600,
      "expires_in": 3600000
    }
  ],
  "summary": {
    "total": 3,
    "valid_count": 2,
    "expired_count": 1,
    "expiring_soon_count": 1
  }
}
```

---

### 4. Get Permission Expiry Status

Get comprehensive expiry status and health information for a user's permissions.

**Endpoint**: `GET /api/v1/admin/users/{user_id}/permissions/expiry-status`

**Response** (200 OK):
```json
{
  "user_id": "user-123",
  "permissions": [
    {
      "permission": "epsx:analytics:view",
      "base_permission": "epsx:analytics:view",
      "expires_at": null,
      "is_expired": false,
      "time_remaining": null,
      "expires_in": "Never"
    },
    {
      "permission": "epsx:rankings:view:25:1735689600",
      "base_permission": "epsx:rankings:view:25",
      "expires_at": 1735689600,
      "is_expired": false,
      "time_remaining": 3600000,
      "expires_in": "1 hour"
    },
    {
      "permission": "admin:users:manage:1703980800",
      "base_permission": "admin:users:manage",
      "expires_at": 1703980800,
      "is_expired": true,
      "time_remaining": 0,
      "expires_in": "Expired"
    }
  ],
  "health": {
    "has_expired": true,
    "has_expiring_soon": true,
    "next_expiry": 1735689600,
    "time_until_next_expiry": 3600000
  }
}
```

---

### 5. Extend Embedded Permission

Extend the expiry time of an existing timestamped permission.

**Endpoint**: `POST /api/v1/admin/users/{user_id}/embedded-permissions/extend`

**Request Body**:
```json
{
  "permission": "epsx:analytics:view:1735689600",
  "new_expiry_timestamp": 1735796400,
  "reason": "Extending access for additional week of analysis"
}
```

**Response** (200 OK):
```json
{
  "old_permission": "epsx:analytics:view:1735689600",
  "new_permission": "epsx:analytics:view:1735796400",
  "extension": 106800000
}
```

---

### 6. Revoke Embedded Permission

Remove a specific timestamped permission from a user.

**Endpoint**: `POST /api/v1/admin/users/{user_id}/embedded-permissions/revoke`

**Request Body**:
```json
{
  "permission": "epsx:analytics:view:1735689600",
  "reason": "Access no longer required - project completed"
}
```

**Response** (200 OK):
```json
{
  "message": "Permission revoked successfully"
}
```

---

### 7. Cleanup Expired Permissions

System-wide cleanup of expired embedded permissions.

**Endpoint**: `POST /api/v1/admin/embedded-permissions/cleanup-expired`

**Request Body**:
```json
{
  "dry_run": false,
  "batch_size": 100
}
```

**Response** (200 OK):
```json
{
  "cleaned": 45,
  "failed": 2,
  "details": [
    {
      "user_id": "user-123",
      "permission": "epsx:analytics:view:1703980800",
      "expired_at": 1703980800,
      "status": "cleaned",
      "error": null
    },
    {
      "user_id": "user-456",
      "permission": "admin:users:manage:1703980800",
      "expired_at": 1703980800,
      "status": "failed",
      "error": "Database constraint violation"
    }
  ]
}
```

## Permission Health Monitoring

### Health Scores

The system provides real-time health scoring for permission status:

- **Excellent** (90-100%): Majority permanent permissions, no expired permissions
- **Good** (70-89%): Mixed permanent/temporary, no expired permissions
- **Warning** (50-69%): Some permissions expiring within 24 hours
- **Critical** (0-49%): Multiple expired permissions or system issues

### Expiry Thresholds

- **Expiring Soon**: Permissions expiring within 24 hours
- **Critical**: Permissions expiring within 1 hour
- **Expired**: Permissions with timestamp < current time

## Best Practices

### 1. Permission Lifecycle Management
```javascript
// Recommended workflow for temporary access
const grantTemporaryAccess = async (userId, basePermission, hours) => {
  const expiryTimestamp = Math.floor(Date.now() / 1000) + (hours * 3600);
  
  await grantEmbeddedPermission({
    user_id: userId,
    base_permission: basePermission,
    expiry_timestamp: expiryTimestamp,
    reason: `Temporary access for ${hours} hours`
  });
};

// Monitor and extend before expiry
const monitorAndExtend = async (userId, permission, additionalHours) => {
  const status = await getPermissionExpiryStatus(userId);
  const expiringSoon = status.permissions.filter(p => 
    p.permission === permission && 
    p.time_remaining < 3600000 // Less than 1 hour
  );
  
  if (expiringSoon.length > 0) {
    await extendEmbeddedPermission({
      user_id: userId,
      permission: permission,
      new_expiry_timestamp: Math.floor(Date.now() / 1000) + (additionalHours * 3600)
    });
  }
};
```

### 2. Bulk Operations
```javascript
// Efficient bulk permission management
const grantTeamAccess = async (userIds, permissions, duration) => {
  const expiryTimestamp = Math.floor(Date.now() / 1000) + (duration * 3600);
  
  const permissionsWithTimestamp = permissions.map(perm => ({
    base_permission: perm,
    platform: perm.split(':')[0],
    resource: perm.split(':')[1], 
    action: perm.split(':')[2],
    expiry_timestamp: expiryTimestamp
  }));
  
  return await grantBulkEmbeddedPermissions({
    user_ids: userIds,
    permissions: permissionsWithTimestamp,
    reason: `Team access for ${duration} hours`
  });
};
```

### 3. Health Monitoring
```javascript
// Regular permission health checks
const performHealthCheck = async () => {
  const users = await getAllUsers();
  const healthReports = [];
  
  for (const user of users) {
    const status = await getPermissionExpiryStatus(user.id);
    
    if (status.health.has_expired || status.health.has_expiring_soon) {
      healthReports.push({
        user_id: user.id,
        health: status.health,
        action_required: true
      });
    }
  }
  
  return healthReports;
};
```

### 4. Automated Cleanup
```javascript
// Scheduled cleanup process
const scheduledCleanup = async () => {
  // Dry run first to see what would be cleaned
  const dryRun = await cleanupExpiredPermissions({
    dry_run: true,
    batch_size: 50
  });
  
  console.log(`Would clean ${dryRun.cleaned} expired permissions`);
  
  // If reasonable number, proceed with actual cleanup
  if (dryRun.cleaned < 1000) {
    return await cleanupExpiredPermissions({
      dry_run: false,
      batch_size: 50
    });
  }
};
```

## Error Handling

### Common Error Scenarios

1. **Permission Already Exists**: When granting a permission the user already has
2. **Invalid Timestamp**: When timestamp is in the past or malformed
3. **User Not Found**: When the specified user ID doesn't exist
4. **Database Constraints**: When permission conflicts with database constraints
5. **Rate Limiting**: When too many requests are made too quickly

### Error Response Format
```json
{
  "error": "error_code",
  "message": "Human readable error message",
  "details": "Additional context or user_id",
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "req_123456789"
}
```

## Rate Limits

- **Individual Operations**: 100 requests per minute per admin user
- **Bulk Operations**: 10 requests per minute per admin user  
- **System Cleanup**: 1 request per hour per admin user
- **Validation Requests**: 200 requests per minute per admin user

## Security Considerations

1. **Audit Logging**: All embedded permission operations are logged
2. **Permission Isolation**: Platform-specific permissions prevent cross-platform access
3. **Expiry Enforcement**: Expired permissions are automatically filtered system-wide
4. **Admin Authorization**: All operations require admin-level permissions
5. **Rate Limiting**: Prevents abuse of permission management endpoints

## Migration Notes

The embedded timestamp system is fully backward compatible:
- Existing permanent permissions continue to work unchanged
- New timestamped permissions can be added alongside permanent ones
- All existing APIs automatically handle both formats
- No database migrations required - uses existing `user_permissions` table

## Support

For technical support with embedded timestamp permissions:
- Check the comprehensive test suite in `tests/integration_embedded_permissions.rs`
- Review E2E tests in `__test__/e2e/embedded-timestamp-permissions-admin.spec.ts`
- Refer to the [MIGRATION.md](../MIGRATION.md) guide for implementation details