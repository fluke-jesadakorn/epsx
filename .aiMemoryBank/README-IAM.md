# EPSX Admin Frontend - Enhanced IAM System

This directory contains the comprehensive Identity and Access Management (IAM) system for the EPSX admin frontend. The system provides granular permission management, automatic package-based permission assignment, and custom permission capabilities.

## Features

### 🎯 Core Capabilities

- **Package-Based Permissions**: Automatic permission assignment when users upgrade packages
- **Custom Permissions**: Admin ability to grant/revoke individual permissions 
- **Permission Templates**: Pre-defined permission sets for common scenarios
- **Audit Logging**: Complete tracking of all permission changes
- **Bulk Operations**: Apply permissions to multiple users simultaneously
- **Payment Integration**: Automatic permission updates on payment events

### 📦 Package Tiers

The system supports the following package tiers with inherited permissions:

- **FREE**: Basic dashboard access
- **BRONZE**: + API personal (1k calls/month), limited rankings (5 items)
- **SILVER**: + API company (10k calls/month), enhanced rankings (25 items)  
- **GOLD**: + API partner (50k calls/month), data export, premium rankings (50 items)
- **PLATINUM**: + Unlimited API, white-label features, unlimited rankings
- **ENTERPRISE**: + Custom integrations, full analytics, custom branding

## File Structure

```
apps/admin-frontend/
├── components/admin/
│   ├── IAMDashboard.tsx          # Main IAM management dashboard
│   ├── EnhancedUserList.tsx      # User list with permission management
│   └── UserPermissionManager.tsx # Individual user permission editor
├── components/auth/
│   └── FeatureGuard.tsx          # Component for feature access control
├── hooks/
│   ├── useFeatureAccess.ts       # Hook for checking feature permissions
│   └── usePaymentIntegration.ts  # Payment event handling
├── services/
│   └── iamService.ts             # IAM API service layer
├── types/admin/
│   └── iam-enhanced.ts           # Enhanced IAM type definitions
├── config/
│   └── packagePermissions.ts    # Package permission configurations
└── app/api/admin/iam/           # API routes for IAM operations
```

## Usage

### 1. User Management

Navigate to `/admin/users` to access the enhanced user management interface:

- View all users with package tiers and permission counts
- Filter by package tier, subscription status, or custom permissions
- Quick package upgrades via dropdown
- Individual permission management via "Manage Permissions" button

### 2. IAM Dashboard

Access the full IAM dashboard at `/admin/iam`:

- Overview statistics and metrics
- User management with advanced filtering
- Permission template management
- Audit log viewing
- Testing tools for payment integration

### 3. Permission Management

For individual users:

```tsx
// Open permission manager for specific user
<UserPermissionManager 
  userId="user-123" 
  onClose={() => setSelectedUser(null)} 
/>
```

### 4. Feature Guards

Protect components based on permissions:

```tsx
import { FeatureGuard, TierGuard } from '@/components/auth/FeatureGuard';

// Feature-based protection
<FeatureGuard featureId="dashboard_advanced" userId={currentUser.id}>
  <AdvancedDashboard />
</FeatureGuard>

// Tier-based protection
<TierGuard requiredTier="gold" userTier={user.packageTier}>
  <PremiumFeature />
</TierGuard>
```

### 5. Payment Integration

The system automatically handles payment events:

```tsx
// Payment integration hook (automatically used in IAMDashboard)
const { triggerPaymentSuccess } = usePaymentIntegration();

// Manual trigger for testing
triggerPaymentSuccess('user-123', 'gold', 'transaction-456');
```

## API Endpoints

### User Management
- `GET /api/admin/iam/users` - List users with filters
- `GET /api/admin/iam/users/{userId}/detailed` - Get user with permissions
- `PATCH /api/admin/iam/users/{userId}/package-tier` - Update package tier

### Permission Management
- `POST /api/admin/iam/apply-package-permissions` - Apply package permissions
- `POST /api/admin/iam/custom-permissions` - Grant custom permission
- `DELETE /api/admin/iam/custom-permissions/{id}` - Revoke custom permission
- `POST /api/admin/iam/bulk-apply-template` - Bulk apply template

### Feature Access
- `GET /api/admin/iam/users/{userId}/feature-access/{featureId}` - Check feature access
- `POST /api/admin/iam/preview-upgrade` - Preview package upgrade effects

### Audit & Logging
- `POST /api/admin/iam/audit-log` - Create audit log entry
- `GET /api/admin/iam/users/{userId}/audit-logs` - Get user audit logs

## Configuration

### Package Permissions

Modify `config/packagePermissions.ts` to adjust default package permissions:

```typescript
export const DEFAULT_PACKAGE_PERMISSIONS: Record<PackageTier, PackagePermission[]> = {
  [PackageTier.GOLD]: [
    {
      id: 'gold_api_partner',
      packageTier: PackageTier.GOLD,
      featureId: 'api_partner',
      permission: { 
        action: 'execute', 
        resource: 'api:partner',
        conditions: [{ type: 'usage_limit', value: 50000, operator: 'lt' }]
      },
      isDefault: true,
      autoGranted: true,
    },
    // ... more permissions
  ]
};
```

### Permission Templates

Add custom permission templates:

```typescript
export const PERMISSION_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'custom_template',
    name: 'Custom Template',
    description: 'Description of the template',
    category: 'Custom',
    isSystem: false,
    permissions: [
      { action: 'view', resource: 'custom:feature' }
    ],
  }
];
```

## Testing

The IAM Dashboard includes testing tools for:

1. **Payment Success**: Simulates successful payment and permission grant
2. **Package Downgrade**: Tests downgrade scenarios  
3. **Subscription Expiry**: Tests automatic downgrade to FREE

Access testing tools via the "Testing Tools" tab in the IAM Dashboard.

## Integration with Payment System

The system listens for the following browser events:

- `paymentSuccess`: Auto-applies package permissions
- `packageDowngrade`: Handles tier downgrades
- `subscriptionExpiry`: Downgrades to FREE tier

Trigger these events from your payment system:

```javascript
// After successful payment
window.dispatchEvent(new CustomEvent('paymentSuccess', {
  detail: { userId, packageTier, transactionId }
}));
```

## Security Considerations

1. **Permission Inheritance**: Higher tiers inherit lower tier permissions
2. **Explicit Deny**: Custom permissions can override package permissions
3. **Audit Trail**: All permission changes are logged with reasons
4. **Expiration**: Custom permissions can have expiration dates
5. **Role-based Access**: Admin-only features are protected

## Future Enhancements

- [ ] Role-based permission inheritance
- [ ] Time-based permission conditions
- [ ] IP-based access restrictions
- [ ] Advanced audit log filtering
- [ ] Permission usage analytics
- [ ] Automated permission cleanup
- [ ] Integration with external identity providers

## Database Schema (Recommended)

```sql
-- Users table (extend existing)
ALTER TABLE users ADD COLUMN package_tier VARCHAR(20) DEFAULT 'free';
ALTER TABLE users ADD COLUMN subscription_status VARCHAR(20) DEFAULT 'inactive';
ALTER TABLE users ADD COLUMN last_payment_date TIMESTAMP;

-- Package permissions table
CREATE TABLE package_permissions (
  id VARCHAR(255) PRIMARY KEY,
  package_tier VARCHAR(20) NOT NULL,
  feature_id VARCHAR(255) NOT NULL,
  action VARCHAR(50) NOT NULL,
  resource VARCHAR(255) NOT NULL,
  conditions JSON,
  is_default BOOLEAN DEFAULT true,
  auto_granted BOOLEAN DEFAULT true
);

-- Custom permissions table
CREATE TABLE custom_permissions (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(255) NOT NULL,
  feature_id VARCHAR(255) NOT NULL,
  action VARCHAR(50) NOT NULL,
  resource VARCHAR(255) NOT NULL,
  conditions JSON,
  granted_by VARCHAR(255) NOT NULL,
  granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP,
  reason TEXT,
  is_active BOOLEAN DEFAULT true
);

-- Audit logs table
CREATE TABLE permission_audit_logs (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(255) NOT NULL,
  action TEXT NOT NULL,
  resource VARCHAR(255),
  performed_by VARCHAR(255) NOT NULL,
  reason TEXT,
  timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  metadata JSON
);
```
