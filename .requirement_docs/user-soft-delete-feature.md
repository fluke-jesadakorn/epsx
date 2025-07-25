# User Soft Delete Feature

## Overview
This document outlines the implementation of a soft delete feature for users in the EPSX application. The feature allows administrators to remove users from the system without permanently deleting their data, providing the ability to restore users if needed.

## Implementation Details

### Database Changes
- **Migration**: `003_add_soft_delete_to_users.sql`
- **Added Column**: `deleted_at TIMESTAMPTZ NULL` to the `users` table
- **Indexes Added**:
  - `idx_users_deleted_at` - General performance index on deleted_at
  - `idx_users_active` - Partial index for active users (WHERE deleted_at IS NULL)

### Domain Changes

#### User Entity (`apps/backend/src/dom/entities/user.rs`)
- Added `deleted_at: Option<DateTime<Utc>>` field
- Added methods:
  - `deleted_at()` - Getter for deleted_at field
  - `is_deleted()` - Check if user is soft deleted
  - `soft_delete()` - Mark user as deleted
  - `restore()` - Restore soft deleted user
- Updated `is_active()` to exclude deleted users

#### Repository Layer (`apps/backend/src/infra/db/postgres/`)
- **Main Repo**: Updated queries to filter out soft deleted users (WHERE deleted_at IS NULL)
- **New Repo**: `user_repo_soft_delete.rs` with specialized methods:
  - `soft_delete()` - Soft delete a user
  - `restore()` - Restore a soft deleted user
  - `get_with_deleted()` - Get user including deleted ones (admin only)
  - `list_deleted()` - List soft deleted users
  - `count_deleted()` - Count soft deleted users
  - `hard_delete()` - Permanently delete already soft deleted users

#### Permission System
- Added `can_admin_modify_user()` method to `PermissionChecker`
- Enforces role hierarchy for deletion permissions:
  - SuperAdmin can delete anyone except themselves
  - Admin can delete lower roles but not SuperAdmin
  - Other roles cannot delete users

#### Events
- Added `UserDeletedEvent` domain event
- Dispatched when a user is soft deleted for audit trails and notifications

### Application Layer

#### DTOs (`apps/backend/src/app/dtos/user.rs`)
- Added `SoftDeleteUserReq` with validation
- Added `SoftDeleteUserRes` response DTO
- Added `ReasonTooLong` validation error

#### Use Cases (`apps/backend/src/app/use_cases/user.rs`)
- Added `soft_delete_user()` method with:
  - Admin permission verification
  - Self-deletion prevention
  - Role hierarchy checking
  - Audit logging
  - Domain event dispatch

### Web Layer

#### Admin Handler (`apps/backend/src/web/admin/handlers.rs`)
- Added `soft_delete_user_handler()` endpoint handler
- Added request DTO `AdminSoftDeleteUserRequest`
- Comprehensive error handling and logging

#### Admin Routes (`apps/backend/src/web/admin/routes.rs`)
- Added `DELETE /admin/users/:user_id` route
- Protected by admin authentication middleware

## API Endpoint

### Delete User (Soft Delete)
- **Method**: DELETE
- **Path**: `/admin/users/{user_id}`
- **Authentication**: Admin or SuperAdmin required
- **Request Body**:
  ```json
  {
    "reason": "Optional deletion reason (max 500 chars)"
  }
  ```
- **Response**:
  ```json
  {
    "usr": {
      "uid": "user_id",
      "email": "user@example.com",
      "disabled": true,
      // ... other user fields
    },
    "deleted_at": "2024-01-15T10:30:00Z"
  }
  ```

## Business Rules

1. **Permission Requirements**:
   - Only Admin and SuperAdmin roles can soft delete users
   - Users cannot delete themselves
   - Admin cannot delete SuperAdmin users
   - SuperAdmin can delete any user except themselves

2. **Data Integrity**:
   - Soft deleted users are excluded from normal queries
   - User sessions remain valid until natural expiration
   - Related data (payments, permissions) retain CASCADE relationships
   - Audit trail is maintained for all deletions

3. **Validation**:
   - User ID must be valid and exist
   - Deletion reason is optional but limited to 500 characters
   - Cannot delete already deleted users

4. **Audit and Events**:
   - All deletions are logged in level history as role change to "DELETED"
   - `UserDeletedEvent` is dispatched for integration with other systems
   - Admin actions are traced with timestamps and reasons

## Migration Instructions

1. Run the database migration:
   ```sql
   -- Apply migration 003_add_soft_delete_to_users.sql
   ```

2. The application layer changes are backward compatible
3. Existing queries will automatically exclude soft deleted users
4. Admin endpoints will be available immediately after deployment

## Testing Considerations

1. **Unit Tests**:
   - Test permission validation logic
   - Test soft delete and restore functionality
   - Test event dispatching

2. **Integration Tests**:
   - Test admin endpoint with various role combinations
   - Test database queries filter deleted users correctly
   - Test cascade behavior with related entities

3. **Manual Testing**:
   - Verify admin can delete appropriate users
   - Verify deleted users don't appear in normal user lists
   - Verify audit logs are created correctly

## Future Enhancements

1. **Admin UI**: Add user management interface for soft delete operations
2. **Bulk Operations**: Support bulk soft delete of multiple users
3. **Restore Endpoint**: Add admin endpoint to restore soft deleted users
4. **Cleanup Job**: Scheduled job to hard delete old soft deleted users
5. **Notifications**: Email notifications for user deletions

## Security Considerations

1. **Data Privacy**: Soft deleted users' data remains in the system
2. **Access Control**: Strict role-based access for deletion operations
3. **Audit Trail**: Complete logging of all deletion activities
4. **Self-Protection**: Prevention of self-deletion for safety