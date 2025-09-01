# EPSX API Reference

## Overview

EPSX provides a comprehensive REST API for user management, analytics, real-time data, and administrative functions. The API is organized around REST principles with clear authentication and authorization requirements.

## Base URLs

- **Production**: `https://api.epsx.com`
- **Staging**: `https://stage-api.epsx.com`
- **Development**: `http://localhost:8000`

## Authentication

Most endpoints require Firebase JWT token authentication via the `Authorization` header:

```
Authorization: Bearer <firebase_jwt_token>
```

### Authentication Flow

1. **Login**: `POST /api/v1/auth/login`
2. **Get Profile**: `GET /api/v1/auth/profile`
3. **Refresh Token**: `POST /api/v1/auth/refresh`
4. **Logout**: `POST /api/v1/auth/logout`

## Authorization Levels

- **Public**: No authentication required
- **Authenticated**: Valid Firebase JWT token required
- **Admin**: Admin role required in user profile
- **Super Admin**: Super admin role required
- **Permission Required**: Specific permission profile features required

---

## Public Endpoints

### Health Check
- **GET** `/health`
- **Description**: System health status
- **Authentication**: None
- **Response**: 
  ```json
  {
    "status": "healthy",
    "timestamp": "2025-01-15T10:30:00Z",
    "service": "epsx-backend"
  }
  ```

### Public Auth Status
- **GET** `/auth/me-public`
- **Description**: Check authentication status without requiring auth
- **Authentication**: None

---

## Authentication Endpoints

### User Login
- **POST** `/api/v1/auth/login`
- **Description**: Authenticate user with email/password
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "email": "user@example.com",
    "password": "password123"
  }
  ```

### User Registration
- **POST** `/api/v1/auth/register`
- **Description**: Register new user account
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "email": "user@example.com",
    "password": "password123",
    "confirm_password": "password123"
  }
  ```

### Auto Registration
- **POST** `/api/v1/auth/register-auto`
- **Description**: Auto-register user with Firebase UID
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "firebase_uid": "firebase_user_id",
    "email": "user@example.com"
  }
  ```

### Password Reset
- **POST** `/api/v1/auth/password-reset`
- **Description**: Initiate password reset process
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "email": "user@example.com"
  }
  ```

### Logout
- **POST** `/api/v1/auth/logout`
- **Description**: Logout current user session
- **Authentication**: Authenticated

### Token Refresh
- **POST** `/api/v1/auth/refresh`
- **Description**: Refresh authentication token
- **Authentication**: Authenticated

### Get User Profile
- **GET** `/api/v1/auth/profile`
- **Description**: Get current authenticated user profile
- **Authentication**: Authenticated
- **Response**:
  ```json
  {
    "id": "uuid",
    "firebase_uid": "firebase_user_id",
    "email": "user@example.com",
    "role": "user",
    "permission_profiles": [...]
  }
  ```

### Clear Session
- **POST** `/api/v1/auth/session/clear`
- **Description**: Clear user session
- **Authentication**: Authenticated

---

## User Management

### Get Current User Profile
- **GET** `/api/v1/users/profile`
- **Description**: Get current user's profile information
- **Authentication**: Authenticated

### Update User Profile
- **PUT** `/api/v1/users/profile`
- **Description**: Update current user's profile
- **Authentication**: Authenticated
- **Request Body**:
  ```json
  {
    "email": "newemail@example.com",
    "preferences": {...}
  }
  ```

### List Users (Admin)
- **GET** `/api/v1/users`
- **Description**: List all users (admin only)
- **Authentication**: Admin
- **Query Parameters**:
  - `limit`: Number of results (default: 50)
  - `offset`: Pagination offset (default: 0)
  - `search`: Search term for email/name

### Get User by ID (Admin)
- **GET** `/api/v1/users/:id`
- **Description**: Get specific user by ID
- **Authentication**: Admin

### Delete User (Admin)
- **DELETE** `/api/v1/users/:id`
- **Description**: Soft delete user account
- **Authentication**: Admin

---

## Market Data

### Get Available Symbols
- **GET** `/api/v1/market-data/symbols`
- **Description**: Get list of available trading symbols
- **Authentication**: Authenticated
- **Response**:
  ```json
  {
    "symbols": ["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", "META"],
    "total": 6,
    "last_updated": "2025-01-15T10:30:00Z"
  }
  ```

---

## Payment System

### Get Crypto Deposit Address
- **GET** `/api/v1/payments/crypto/deposit-address`
- **Description**: Get cryptocurrency deposit address
- **Authentication**: Authenticated

### Create MusePay Payment
- **POST** `/api/v1/payments/musepay/create`
- **Description**: Create new MusePay payment
- **Authentication**: Authenticated

### MusePay Webhook
- **POST** `/api/v1/webhooks/payments/musepay`
- **Description**: Handle MusePay payment notifications
- **Authentication**: System (webhook signature verification)

---

## Premium Features

### Premium Rankings
- **GET** `/api/v1/premium/rankings`
- **Description**: Get premium stock rankings
- **Authentication**: Permission Required (Silver+ profile)

---

## System Management

### Clear Cache
- **POST** `/api/v1/system/cache`
- **Description**: Clear system cache
- **Authentication**: Authenticated

---

## Audit System

### Create Audit Log
- **POST** `/api/v1/audit/logs`
- **Description**: Create audit log entry (for frontend logging)
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "user_id": "uuid_optional",
    "action": "LOGIN_SUCCESS",
    "resource_type": "auth",
    "details": {...},
    "ip_address": "192.168.1.1",
    "user_agent": "Mozilla/5.0...",
    "event_category": "authentication",
    "severity": "medium",
    "success": true
  }
  ```

### Search Audit Logs (Admin)
- **GET** `/api/v1/audit/logs`
- **Description**: Search audit logs with filters
- **Authentication**: Admin
- **Query Parameters**:
  - `actor_id`: Filter by user ID
  - `action`: Filter by action type
  - `resource_type`: Filter by resource type
  - `from_time`: Start time (ISO 8601)
  - `to_time`: End time (ISO 8601)
  - `limit`: Results per page (default: 100)
  - `offset`: Pagination offset

### Get Audit Log (Admin)
- **GET** `/api/v1/audit/logs/:log_id`
- **Description**: Get specific audit log entry
- **Authentication**: Admin

### Audit Statistics (Admin)
- **GET** `/api/v1/audit/statistics`
- **Description**: Get audit statistics for reporting
- **Authentication**: Admin
- **Query Parameters**:
  - `from_time`: Start time (required)
  - `to_time`: End time (required)

### Export Audit Logs (Admin)
- **GET** `/api/v1/audit/export`
- **Description**: Export audit logs for reporting
- **Authentication**: Admin
- **Query Parameters**:
  - `format`: Export format (json, csv, xml)
  - All search filters from `/logs` endpoint

---

## IAM (Identity & Access Management)

### Role Management

#### Create Role
- **POST** `/api/v1/iam/roles`
- **Authentication**: Admin

#### List Roles
- **GET** `/api/v1/iam/roles`
- **Authentication**: Admin

#### Get Role
- **GET** `/api/v1/iam/roles/:role_id`
- **Authentication**: Admin

#### Update Role
- **PUT** `/api/v1/iam/roles/:role_id`
- **Authentication**: Admin

#### Delete Role
- **DELETE** `/api/v1/iam/roles/:role_id`
- **Authentication**: Admin

### Policy Management

#### Create Policy
- **POST** `/api/v1/iam/policies`
- **Authentication**: Admin

#### List Policies
- **GET** `/api/v1/iam/policies`
- **Authentication**: Admin

#### Get Policy
- **GET** `/api/v1/iam/policies/:policy_id`
- **Authentication**: Admin

#### Delete Policy
- **DELETE** `/api/v1/iam/policies/:policy_id`
- **Authentication**: Admin

### Permission Evaluation

#### Evaluate Permission
- **POST** `/api/v1/iam/evaluate`
- **Description**: Evaluate user permission for specific action
- **Authentication**: Admin

### User Permission Management

#### Set User Permission Overrides
- **POST** `/api/v1/iam/users/:user_id/overrides`
- **Authentication**: Admin

#### Get User Permission Overrides
- **GET** `/api/v1/iam/users/:user_id/overrides`
- **Authentication**: Admin

#### Assign Role to User
- **POST** `/api/v1/iam/users/:user_id/roles/:role_id`
- **Authentication**: Admin

#### Remove Role from User
- **DELETE** `/api/v1/iam/users/:user_id/roles/:role_id`
- **Authentication**: Admin

#### Get User Roles
- **GET** `/api/v1/iam/users/:user_id/roles`
- **Authentication**: Admin

---

## Permission Profiles

### Profile Management

#### Create Permission Profile
- **POST** `/api/v1/permission-profiles/permission-profiles`
- **Description**: Create new permission profile
- **Authentication**: Admin
- **Request Body**:
  ```json
  {
    "name": "Premium User",
    "description": "Premium tier access",
    "category": "user",
    "permissions": [...],
    "tier": "gold"
  }
  ```

#### Search Permission Profiles
- **GET** `/api/v1/permission-profiles/permission-profiles`
- **Description**: Search permission profiles with filters
- **Authentication**: Admin
- **Query Parameters**:
  - `name`: Filter by name (partial match)
  - `category`: Filter by category
  - `tier`: Filter by target tier
  - `active_only`: Only active profiles (default: true)
  - `limit`: Results per page (default: 50)
  - `offset`: Pagination offset

#### Get Permission Profile
- **GET** `/api/v1/permission-profiles/permission-profiles/:profile_id`
- **Authentication**: Admin

#### Update Permission Profile
- **PUT** `/api/v1/permission-profiles/permission-profiles/:profile_id`
- **Authentication**: Admin

#### Delete Permission Profile
- **DELETE** `/api/v1/permission-profiles/permission-profiles/:profile_id`
- **Description**: Soft delete permission profile
- **Authentication**: Admin

### Profile Application

#### Apply Permission Profile
- **POST** `/api/v1/permission-profiles/permission-profiles/:profile_id/apply`
- **Description**: Apply permission profile to users
- **Authentication**: Admin
- **Request Body**:
  ```json
  {
    "user_ids": ["uuid1", "uuid2"],
    "reason": "Promotional upgrade",
    "overrides": {...},
    "expires_at": "2025-12-31T00:00:00Z"
  }
  ```

#### Get Application History
- **GET** `/api/v1/permission-profiles/permission-profiles/:profile_id/history`
- **Description**: Get permission profile application history
- **Authentication**: Admin

### System Operations

#### Initialize Default Permission Profiles
- **POST** `/api/v1/permission-profiles/initialize-defaults`
- **Description**: Create default permission profiles
- **Authentication**: Super Admin

---

## Real-time Communication

### WebSocket Connection
- **GET** `/api/v1/realtime/ws`
- **Description**: Upgrade to WebSocket connection
- **Authentication**: Authenticated
- **Protocol**: WebSocket upgrade

### Server-Sent Events
- **GET** `/api/v1/realtime/events`
- **Description**: Connect to server-sent events stream
- **Authentication**: Authenticated
- **Content-Type**: text/event-stream

### SSE Health Check
- **GET** `/api/v1/realtime/events/health`
- **Description**: Health check for SSE connections
- **Authentication**: Authenticated

### Admin Real-time Operations

#### Broadcast Notification
- **POST** `/api/v1/realtime/admin/broadcast`
- **Description**: Broadcast notification to all connected clients
- **Authentication**: Admin

#### Simulate Payment Event
- **POST** `/api/v1/realtime/admin/simulate/payment`
- **Description**: Simulate payment event for testing
- **Authentication**: Admin

#### Simulate Stock Update
- **POST** `/api/v1/realtime/admin/simulate/stock`
- **Description**: Simulate stock price update for testing
- **Authentication**: Admin

#### Get Connection Statistics
- **GET** `/api/v1/realtime/admin/stats`
- **Description**: Get real-time connection statistics
- **Authentication**: Admin

#### Send User Notification
- **POST** `/api/v1/realtime/admin/notify/:user_id`
- **Description**: Send notification to specific user
- **Authentication**: Admin

---

## Admin Management

### Admin Authentication
- **POST** `/api/v1/admin/auth/logout`
- **Description**: Admin logout
- **Authentication**: Admin

- **GET** `/api/v1/admin/auth/profile`
- **Description**: Get admin profile
- **Authentication**: Admin

### Admin Analytics
- **GET** `/api/v1/admin/analytics/user-statistics`
- **Description**: Get user analytics and statistics
- **Authentication**: Admin

### Admin User Management

#### List Users
- **GET** `/api/v1/admin/users`
- **Authentication**: Admin

#### Create User
- **POST** `/api/v1/admin/users`
- **Authentication**: Admin

#### Get User
- **GET** `/api/v1/admin/users/:user_id`
- **Authentication**: Admin

#### Update User Role
- **PUT** `/api/v1/admin/users/:user_id`
- **Authentication**: Admin

#### Soft Delete User
- **DELETE** `/api/v1/admin/users/:user_id`
- **Authentication**: Admin

#### Bulk Update Roles
- **POST** `/api/v1/admin/users/batch-update-roles`
- **Description**: Update multiple user roles in batch
- **Authentication**: Admin

#### Get Role History
- **GET** `/api/v1/admin/users/:user_id/role-history`
- **Description**: Get user's role change history
- **Authentication**: Admin

### Admin Permission Profile Management

#### List Permission Profiles
- **GET** `/api/v1/admin/permission-profiles`
- **Authentication**: Admin

#### Get Permission Profile Details
- **GET** `/api/v1/admin/permission-profiles/:profile_id`
- **Authentication**: Admin

#### Assign Permission Profile Directly
- **POST** `/api/v1/admin/permission-profiles/assign`
- **Description**: Directly assign permission profile to user
- **Authentication**: Admin

---

## Admin API Separate Routes

### Admin Login (Public)
- **POST** `/api/admin/auth/login`
- **Description**: Admin-specific login endpoint
- **Authentication**: Public
- **Request Body**:
  ```json
  {
    "email": "admin@example.com",
    "password": "admin_password"
  }
  ```

---

## Error Responses

All endpoints return consistent error responses:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input parameters",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    }
  },
  "timestamp": "2025-01-15T10:30:00Z"
}
```

### Common Error Codes

- `AUTHENTICATION_REQUIRED`: 401 - Authentication token required
- `AUTHORIZATION_FAILED`: 403 - Insufficient permissions
- `VALIDATION_ERROR`: 400 - Invalid input parameters
- `RESOURCE_NOT_FOUND`: 404 - Requested resource not found
- `RATE_LIMIT_EXCEEDED`: 429 - Too many requests
- `INTERNAL_ERROR`: 500 - Server internal error

---

## Rate Limiting

Rate limits are enforced per user and endpoint based on permission profiles:

- **Bronze**: 10 requests/minute, 100/hour, 1000/day
- **Silver**: 50 requests/minute, 500/hour, 5000/day
- **Gold**: 200 requests/minute, 2000/hour, 20000/day
- **Admin**: Unlimited

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1642248000
```

---

## Pagination

List endpoints support pagination:

**Query Parameters**:
- `limit`: Results per page (default: 50, max: 500)
- `offset`: Number of items to skip (default: 0)

**Response Format**:
```json
{
  "data": [...],
  "pagination": {
    "total": 1250,
    "limit": 50,
    "offset": 0,
    "has_more": true
  }
}
```

---

## Versioning

The API uses URL versioning with `/api/v1/` prefix. Future versions will be introduced as `/api/v2/`, etc., with backward compatibility maintained for at least 12 months.

Current version: **v1.0**

---

## SDKs and Libraries

Official SDKs are available for:
- **JavaScript/TypeScript**: `@epsx/api-client`
- **Python**: `epsx-python-sdk` (planned)
- **Go**: `epsx-go-sdk` (planned)

Example usage with TypeScript SDK:
```typescript
import { EpsxClient } from '@epsx/api-client';

const client = new EpsxClient({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.epsx.com'
});

const profile = await client.auth.getProfile();
```