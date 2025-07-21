# Firebase IAM Implementation Guide

## Overview

This document describes the Firebase Firestore implementation of the Identity and Access Management (IAM) system for the EPSX admin-frontend application.

## Architecture

### Firebase Collections

#### 1. `users`
Main user collection with basic information and package tier:
```typescript
{
  id: string;
  email: string;
  name: string;
  displayName?: string;
  emailVerified: boolean;
  disabled: boolean;
  packageTier: PackageTier;
  subscriptionStatus: SubscriptionStatus;
  lastPaymentDate?: Date;
  createdAt: Date;
  updatedAt: Date;
  // ... other user fields
}
```

#### 2. `custom_permissions`
Custom permissions granted to specific users:
```typescript
{
  id: string;
  userId: string;
  featureId: string;
  permission: Permission; // 'READ', 'WRITE', 'DELETE', 'ADMIN'
  grantedBy: string;
  grantedAt: Date;
  expiresAt?: Date;
  reason?: string;
  isActive: boolean;
  revokedAt?: Date;
  revokedBy?: string;
  revokeReason?: string;
}
```

#### 3. `effective_permissions`
Flattened view of all user permissions (package + custom):
```typescript
{
  id: string;
  userId: string;
  featureId: string;
  permission: Permission;
  source: PermissionSource; // 'PACKAGE', 'CUSTOM', 'ROLE'
  grantedAt: Date;
  expiresAt?: Date;
  grantedBy: string;
  customPermissionId?: string; // Reference to custom permission if applicable
}
```

#### 4. `permission_audit_logs`
Audit trail for all permission changes:
```typescript
{
  id: string;
  userId: string;
  action: string;
  resource: string;
  performedBy: string;
  reason?: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}
```

## Firebase Service Layer

### `FirebaseIAMService`

The main service class that handles all Firebase Firestore operations:

#### Key Methods:

- `getUsers(filters?)` - Fetch users with optional filtering
- `getUserWithPermissions(userId)` - Get user with all permission details
- `updateUserPackageTier(userId, newTier, updatedBy)` - Update user package and apply permissions
- `applyPackagePermissions(userId, packageTier)` - Apply package-specific permissions
- `grantCustomPermission(userId, featureId, permission, grantedBy, options?)` - Grant custom permission
- `revokeCustomPermission(permissionId, revokedBy, reason?)` - Revoke custom permission
- `hasFeatureAccess(userId, featureId)` - Check if user has access to feature
- `getUserEffectivePermissions(userId)` - Get all effective permissions for user
- `bulkApplyTemplate(userIds, templateId, appliedBy)` - Apply permission template to multiple users
- `createAuditLog(logEntry)` - Create audit log entry
- `cleanupExpiredPermissions()` - Remove expired permissions

## Integration with Payment System

The Firebase IAM system integrates with the payment system through the `usePaymentIntegration` hook:

```typescript
// When payment succeeds
await firebaseIAMService.applyPackagePermissions(userId, packageTier);

// When package downgrades
await firebaseIAMService.updateUserPackageTier(userId, newTier, 'SYSTEM');

// When subscription expires
await firebaseIAMService.updateUserPackageTier(userId, PackageTier.FREE, 'SYSTEM');
```

## React Components

### `FirebaseFeatureGuard`
Conditionally renders components based on user permissions:

```tsx
<FirebaseFeatureGuard featureId="advanced_analytics">
  <AdvancedAnalyticsComponent />
</FirebaseFeatureGuard>

<FirebasePackageTierGuard requiredTier="GOLD">
  <PremiumFeature />
</FirebasePackageTierGuard>
```

### `useFeatureAccess` Hook
React hook for checking feature access:

```tsx
const { hasAccess, loading, error } = useFeatureAccess(userId, 'advanced_analytics');
```

## Firebase Security Rules

The system includes comprehensive Firestore security rules:

- **Admin Access**: Only authenticated admin users can read/write IAM data
- **User Access**: Users can read their own effective permissions
- **Permission Checks**: Granular permission checks for different operations
- **Audit Protection**: Audit logs are system-managed only

## Setup and Initialization

### 1. Firebase Configuration
Ensure your Firebase configuration is set up in `lib/firebase.ts`:

```typescript
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  // ... other config
};
```

### 2. Environment Variables
Set up the following environment variables:
```env
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your_project.appspot.com
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
NEXT_PUBLIC_FIREBASE_APP_ID=your_app_id
```

### 3. Initialize IAM System
Run the initialization script to set up collections and sample data:

```typescript
import { initializeFirebaseIAM } from './scripts/initializeFirebaseIAM';

// Initialize the system (run once)
await initializeFirebaseIAM();

// Check system health
await checkFirebaseIAMHealth();
```

### 4. Deploy Security Rules
Deploy the Firestore security rules to your Firebase project:

```bash
firebase deploy --only firestore:rules
```

## Usage Examples

### Basic Permission Check
```typescript
const hasAccess = await firebaseIAMService.hasFeatureAccess('user123', 'premium_analytics');
if (hasAccess) {
  // Show premium analytics
}
```

### Grant Custom Permission
```typescript
await firebaseIAMService.grantCustomPermission(
  'user123',
  'beta_features',
  'READ',
  'admin456',
  {
    reason: 'Beta tester',
    expiresAt: new Date('2024-12-31')
  }
);
```

### Update Package Tier
```typescript
await firebaseIAMService.updateUserPackageTier('user123', PackageTier.GOLD, 'admin456');
```

### Bulk Apply Template
```typescript
await firebaseIAMService.bulkApplyTemplate(
  ['user1', 'user2', 'user3'],
  'enterprise_template',
  'admin456'
);
```

## Monitoring and Maintenance

### Performance Considerations
1. **Indexing**: Create composite indexes for frequently queried fields
2. **Caching**: Consider implementing caching for permission checks
3. **Batch Operations**: Use Firebase batch writes for bulk operations

### Monitoring
1. **Audit Logs**: Monitor audit logs for suspicious activity
2. **Permission Usage**: Track which permissions are most used
3. **Performance**: Monitor query performance and optimize as needed

### Maintenance Tasks
1. **Cleanup Expired Permissions**: Run periodically to remove expired permissions
2. **Archive Old Audit Logs**: Archive or delete old audit logs to manage storage
3. **Update Permission Templates**: Keep permission templates up to date with new features

## Migration from Mock API

To migrate from the mock API implementation:

1. **Update Service Layer**: Switch from HTTP calls to Firebase calls
2. **Update Components**: No changes needed - they use the same service interface
3. **Initialize Data**: Run the initialization script to populate Firebase
4. **Test Permissions**: Verify all permission checks work with Firebase
5. **Deploy Rules**: Deploy the security rules to Firebase

## Troubleshooting

### Common Issues

1. **Permission Denied Errors**: Check Firebase security rules and admin authentication
2. **Slow Queries**: Add appropriate indexes in Firebase console
3. **Missing Permissions**: Verify package permissions are applied correctly
4. **Audit Logs Not Created**: Check if audit logging is properly configured

### Debug Mode
Enable debug logging for Firebase operations:

```typescript
// Enable debug logging
const debugMode = process.env.NODE_ENV === 'development';
if (debugMode) {
  console.log('Firebase IAM Debug Mode Enabled');
}
```

## Future Enhancements

1. **Real-time Updates**: Implement real-time permission updates using Firebase listeners
2. **Permission Inheritance**: Add role-based permission inheritance
3. **Advanced Analytics**: Add detailed permission usage analytics
4. **Bulk Operations UI**: Create admin interface for bulk permission operations
5. **Permission Templates**: Expand permission template system with more granular controls
