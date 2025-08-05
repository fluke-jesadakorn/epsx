# EPSX Casbin Authorization API Reference

## Overview

The EPSX Casbin Authorization API provides comprehensive policy-based access control for the EPSX trading platform. This API supports role-based access control (RBAC), dynamic policy management, and real-time authorization decisions.

**Base URL:** `https://api.epsx.com/api/v1`  
**Authentication:** Bearer token or session-based authentication  
**Content-Type:** `application/json`

## Table of Contents

1. [Authentication](#authentication)
2. [IAM Endpoints](#iam-endpoints)
3. [Admin Casbin Endpoints](#admin-casbin-endpoints)
4. [Health & Monitoring](#health--monitoring)
5. [Error Responses](#error-responses)
6. [Rate Limiting](#rate-limiting)

## Authentication

All protected endpoints require authentication via:
- **Bearer Token:** `Authorization: Bearer <token>` 
- **Session Cookie:** `session_id=<session_id>`

### Role Hierarchy
- `admin`: Full system access
- `moderator`: User management capabilities  
- `premium_user`: Advanced trading features
- `basic_user`: Basic trading functionality

## IAM Endpoints

### Check Permission
Check if a user has permission for a specific resource and action.

```http
GET /api/v1/iam/check-permission/{user_id}/{resource}/{action}
```

**Parameters:**
- `user_id` (string): User identifier
- `resource` (string): Resource path (e.g., `/api/v1/trading`)
- `action` (string): HTTP method (GET, POST, PUT, DELETE)

**Response:**
```json
{
  "allowed": true,
  "user_id": "user123",
  "resource": "/api/v1/trading",
  "action": "GET"
}
```

### Evaluate Permission
Detailed permission evaluation with context.

```http
POST /api/v1/iam/evaluate-permission
```

**Request Body:**
```json
{
  "user_id": "user123",
  "resource": "/api/v1/analytics",
  "action": "GET"
}
```

**Response:**
```json
{
  "allowed": true,
  "user_id": "user123", 
  "resource": "/api/v1/analytics",
  "action": "GET"
}
```

### Assign Role
Assign a role to a user.

```http
POST /api/v1/iam/assign-role/{user_id}/{role}
```

**Parameters:**
- `user_id` (string): User identifier
- `role` (string): Role name (admin, moderator, premium_user, basic_user)

**Response:**
```http
200 OK - Role assigned successfully
409 CONFLICT - Role already assigned
```

### Revoke Role
Remove a role from a user.

```http
DELETE /api/v1/iam/revoke-role/{user_id}/{role}
```

**Response:**
```http
200 OK - Role revoked successfully
404 NOT FOUND - Role not assigned to user
```

### List User Roles
Get all roles assigned to a user.

```http
GET /api/v1/iam/user-roles/{user_id}
```

**Response:**
```json
{
  "roles": ["premium_user", "moderator"],
  "user_id": "user123"
}
```

### Add Policy
Add a new authorization policy.

```http
POST /api/v1/iam/add-policy
```

**Request Body:**
```json
{
  "subject": "premium_user",
  "object": "/api/v1/advanced-analytics", 
  "action": "GET"
}
```

**Response:**
```http
201 CREATED - Policy added successfully
200 OK - Policy already exists
```

### Remove Policy
Remove an authorization policy.

```http
POST /api/v1/iam/remove-policy
```

**Request Body:**
```json
{
  "subject": "premium_user",
  "object": "/api/v1/advanced-analytics",
  "action": "GET"
}
```

### List Policies
Get all authorization policies.

```http
GET /api/v1/iam/policies
```

**Response:**
```json
[
  {
    "subject": "admin",
    "object": "/api/v1/admin",
    "action": "GET"
  },
  {
    "subject": "basic_user", 
    "object": "/api/v1/trading",
    "action": "GET"
  }
]
```

## Admin Casbin Endpoints

### Get All Policies
Retrieve all policies and role inheritances (Admin only).

```http
GET /api/v1/admin/casbin/policies
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "policies": [
      ["admin", "/api/v1/admin", "GET"],
      ["basic_user", "/api/v1/trading", "GET"]
    ],
    "role_inheritances": [
      ["admin_user", "admin"],
      ["user123", "premium_user"]
    ],
    "total_policies": 27,
    "total_role_inheritances": 4
  }
}
```

### Add Policy (Admin)
Add a single policy with enhanced response.

```http
POST /api/v1/admin/casbin/add-policy
```

**Request Body:**
```json
{
  "subject": "new_role",
  "object": "/api/v1/feature",
  "action": "POST"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Policy added successfully",
  "affected_count": 1
}
```

### Remove Policy (Admin)
Remove a single policy.

```http
POST /api/v1/admin/casbin/remove-policy
```

**Request Body:**
```json
{
  "subject": "old_role",
  "object": "/api/v1/deprecated",
  "action": "GET"
}
```

### Batch Add Policies
Add multiple policies in a single request.

```http
POST /api/v1/admin/casbin/batch-policies
```

**Request Body:**
```json
{
  "policies": [
    {
      "subject": "new_user",
      "object": "/api/v1/resource1",
      "action": "GET"
    },
    {
      "subject": "new_user", 
      "object": "/api/v1/resource2",
      "action": "POST"
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully processed 2 policies",
  "affected_count": 2
}
```

### Assign Role (Admin)
Assign role with detailed response.

```http
POST /api/v1/admin/casbin/assign-role
```

**Request Body:**
```json
{
  "user": "user456",
  "role": "moderator"
}
```

### Remove Role (Admin)
Remove role assignment.

```http
POST /api/v1/admin/casbin/remove-role
```

**Request Body:**
```json
{
  "user": "user456", 
  "role": "moderator"
}
```

### Get User Roles (Admin)
Get detailed role information for a user.

```http
GET /api/v1/admin/casbin/user-roles/{user_id}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "user_id": "user123",
    "roles": ["premium_user"],
    "role_count": 1
  }
}
```

### Get User Permissions (Admin)
Get all permissions for a user.

```http
GET /api/v1/admin/casbin/user-permissions/{user_id}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "user_id": "user123",
    "permissions": [
      ["/api/v1/trading", "GET"],
      ["/api/v1/analytics", "GET"]
    ],
    "permission_count": 2
  }
}
```

### Test Policy
Test policy enforcement for debugging.

```http
POST /api/v1/admin/casbin/test-policy
```

**Request Body:**
```json
{
  "subject": "test_user",
  "object": "/api/v1/admin",
  "action": "GET"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "subject": "test_user",
    "object": "/api/v1/admin", 
    "action": "GET",
    "allowed": false,
    "enforcement_result": "DENY"
  }
}
```

### Reload Policies
Reload policies from database.

```http
POST /api/v1/admin/casbin/reload-policies
```

**Response:**
```json
{
  "success": true,
  "message": "Policies reloaded successfully"
}
```

### Cache Statistics
Get cache performance metrics.

```http
GET /api/v1/admin/casbin/cache-stats
```

**Response:**
```json
{
  "total_entries": 1500,
  "active_entries": 1200,
  "expired_entries": 300,
  "max_entries": 10000,
  "cache_hit_ratio": 87.5,
  "default_ttl_seconds": 300
}
```

### Clear Cache
Clear the policy cache.

```http
POST /api/v1/admin/casbin/clear-cache
```

**Response:**
```json
{
  "success": true,
  "message": "Policy cache cleared successfully"
}
```

## Health & Monitoring

### Health Check
Comprehensive system health check.

```http
GET /api/v1/health/health-check
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1703875200,
  "version": "1.0.0",
  "environment": "production",
  "checks": {
    "casbin_service": {
      "status": "healthy",
      "message": "Casbin service is operational",
      "response_time_ms": 15
    },
    "database": {
      "status": "healthy", 
      "message": "Database connection is healthy",
      "response_time_ms": 25
    },
    "cache": {
      "status": "healthy",
      "message": "Cache system status",
      "response_time_ms": 5
    }
  },
  "metrics": {
    "casbin_cache_stats": {
      "total_entries": 1200,
      "hit_ratio": 0.85
    },
    "database_stats": {
      "total_policies": 27,
      "total_roles": 4
    }
  }
}
```

### Readiness Check
Lightweight readiness check for load balancers.

```http
GET /api/v1/health/readiness
```

**Response:**
```json
{
  "status": "ready",
  "timestamp": 1703875200
}
```

### Liveness Check
Simple liveness probe.

```http
GET /api/v1/health/liveness
```

**Response:**  
```json
{
  "status": "alive",
  "timestamp": 1703875200
}
```

### Metrics
Detailed system metrics for monitoring.

```http
GET /api/v1/health/metrics
```

**Response:**
```json
{
  "casbin_cache_stats": {
    "total_entries": 1500,
    "hit_ratio": 0.87,
    "memory_usage_bytes": 2048000
  },
  "policy_enforcement_stats": {
    "total_requests": 50000,
    "successful_requests": 49500,
    "average_response_time_ms": 12.5,
    "p95_response_time_ms": 45.0,
    "requests_per_second": 125.5
  },
  "database_stats": {
    "total_policies": 27,
    "total_roles": 4,
    "connection_pool_size": 10,
    "active_connections": 3
  }
}
```

### Diagnostic
Diagnostic information for troubleshooting.

```http
GET /api/v1/health/diagnostic
```

**Response:**
```json
{
  "timestamp": 1703875200,
  "service": "casbin_authorization",
  "policy_summary": {
    "total_policies": 27,
    "role_policies": 4,
    "policy_breakdown": {
      "user_policies": 20,
      "role_policies": 7
    }
  },
  "cache_diagnostics": {
    "utilization_percent": 12.0,
    "efficiency": 80.0,
    "recommendation": "Cache size appears optimal"
  },
  "recommendations": [
    "System performing optimally"
  ]
}
```

## Error Responses

### Standard Error Format
```json
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "User does not have permission to access this resource",
    "details": {
      "user_id": "user123",
      "resource": "/api/v1/admin",
      "action": "GET"
    },
    "timestamp": "2024-12-05T10:30:00Z"
  }
}
```

### Common HTTP Status Codes

| Status Code | Description | Example |
|-------------|-------------|---------|
| `200 OK` | Request successful | Policy check passed |
| `201 CREATED` | Resource created | New policy added |
| `400 BAD REQUEST` | Invalid request data | Missing required fields |
| `401 UNAUTHORIZED` | Authentication required | Invalid or missing token |
| `403 FORBIDDEN` | Permission denied | User lacks required role |
| `404 NOT FOUND` | Resource not found | User or policy doesn't exist |
| `409 CONFLICT` | Resource already exists | Duplicate policy |
| `429 TOO MANY REQUESTS` | Rate limit exceeded | Request throttled |
| `500 INTERNAL SERVER ERROR` | Server error | Database connection failed |
| `503 SERVICE UNAVAILABLE` | Service temporarily down | Circuit breaker open |

### Error Codes

| Error Code | Description | Resolution |
|------------|-------------|------------|
| `INVALID_INPUT` | Request validation failed | Check request format and required fields |
| `PERMISSION_DENIED` | Authorization failed | Verify user permissions and roles |
| `POLICY_NOT_FOUND` | Policy doesn't exist | Check policy existence before removal |
| `USER_NOT_FOUND` | User doesn't exist | Verify user ID is correct |
| `ROLE_NOT_ASSIGNED` | Role not assigned to user | Assign role before attempting removal |
| `DATABASE_ERROR` | Database operation failed | Check database connectivity |
| `CACHE_ERROR` | Cache operation failed | Cache will fallback to database |
| `CIRCUIT_BREAKER_OPEN` | Service temporarily unavailable | Wait for service recovery |

## Rate Limiting

Rate limits are applied per user and endpoint:

| Endpoint Group | Limit | Window |
|----------------|--------|--------|
| Permission Checks | 1000 requests | 1 minute |
| Policy Management | 100 requests | 1 minute |
| Admin Operations | 50 requests | 1 minute |
| Health Checks | 300 requests | 1 minute |

**Rate Limit Headers:**
- `X-RateLimit-Limit`: Request limit per window
- `X-RateLimit-Remaining`: Remaining requests in current window  
- `X-RateLimit-Reset`: Window reset time (Unix timestamp)

## Code Examples

### JavaScript/TypeScript
```javascript
// Check user permission
async function checkPermission(userId, resource, action) {
  const response = await fetch(
    `/api/v1/iam/check-permission/${userId}/${encodeURIComponent(resource)}/${action}`,
    {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    }
  );
  
  if (!response.ok) {
    throw new Error(`Permission check failed: ${response.status}`);
  }
  
  const result = await response.json();
  return result.allowed;
}

// Batch add policies (admin)
async function addPolicies(policies) {
  const response = await fetch('/api/v1/admin/casbin/batch-policies', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${adminToken}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ policies })
  });
  
  return response.json();
}
```

### Python
```python
import requests

class CasbinClient:
    def __init__(self, base_url, token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }
    
    def check_permission(self, user_id, resource, action):
        url = f"{self.base_url}/iam/check-permission/{user_id}/{resource}/{action}"
        response = requests.get(url, headers=self.headers)
        response.raise_for_status()
        return response.json()['allowed']
    
    def assign_role(self, user_id, role):
        url = f"{self.base_url}/iam/assign-role/{user_id}/{role}"
        response = requests.post(url, headers=self.headers)
        return response.status_code == 200
```

### cURL Examples
```bash
# Check permission
curl -H "Authorization: Bearer $TOKEN" \
     "https://api.epsx.com/api/v1/iam/check-permission/user123/api%2Fv1%2Ftrading/GET"

# Add policy
curl -X POST \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"subject":"new_user","object":"/api/v1/resource","action":"GET"}' \
     "https://api.epsx.com/api/v1/iam/add-policy"

# Get cache stats
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     "https://api.epsx.com/api/v1/admin/casbin/cache-stats"
```

---

**API Version:** v1  
**Last Updated:** December 2024  
**Contact:** backend-team@epsx.com