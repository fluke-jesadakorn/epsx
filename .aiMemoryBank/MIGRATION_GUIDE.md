# Migration Guide: From Role-Based to AWS IAM-Inspired Permission System

## Overview
This guide outlines the migration process from the current role-based system to the new AWS IAM-inspired permission service.

## Current System vs New System

### Current System (Legacy)
- **Role-based**: Users have simple roles like `BRONZE`, `SILVER`, `GOLD`, `PLATINUM`
- **Token-based**: Features gated by token balance
- **Simple permissions**: Basic enum-based permissions
- **Hardcoded limits**: Ranking limits based on subscription level

### New System (AWS IAM-Inspired)
- **Policy-based**: Flexible policies define what users can do
- **Resource-action model**: Fine-grained control over specific resources and actions
- **Conditional access**: Time-based, IP-based, and context-aware permissions
- **Hierarchical groups**: Users can belong to multiple groups with inherited permissions
- **Audit logging**: Complete audit trail of all permission checks

## Migration Strategy

### Phase 1: Parallel System (Current)
- ✅ New permission service created alongside existing system
- ✅ Legacy hooks updated to include new permission properties
- ✅ Gradual integration of new permission checks
- ✅ Backward compatibility maintained

### Phase 2: Database Migration
```sql
-- Run migration scripts
-- 1. Create new permission tables
\i /packages/auth/src/permission-service/database/schema.sql

-- 2. Migrate existing user data
\i /packages/auth/src/permission-service/database/migration.sql
```

### Phase 3: Hook Migration
```typescript
// OLD: useRankingAccess
const { maxRankings, userLevel, isExpired } = useRankingAccess();

// NEW: usePermissionAwareAccess
const { 
  permissions, 
  canAccessResource, 
  canAccessRankings 
} = usePermissionAwareAccess();
```

### Phase 4: Component Updates
```tsx
// OLD: Role-based gates
{userLevel === 'PLATINUM' && <PremiumFeature />}

// NEW: Permission-based gates
<PermissionGate resource="stock:analytics" action="analyze">
  <PremiumFeature />
</PermissionGate>
```

## Files Updated

### Hooks
- ✅ `useRankingAccess.ts` - Added new permission properties
- ✅ `usePermissionAwareAccess.ts` - New bridge hook created
- ✅ `useFeatureAccess.ts` - Added permission service integration

### Components
- ✅ `TokenGatedFeature.tsx` - Added permission-based access control
- ✅ `RoleAwareLazyStockRankingTable.tsx` - Integrated new permission system

### Services
- ✅ `PermissionService.ts` - Core permission evaluation engine
- ✅ `PolicyTemplates.ts` - Pre-built policy templates

## Key Changes by Component

### 1. useRankingAccess Hook
```typescript
interface RankingAccess {
  // Legacy properties (maintained)
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  
  // NEW: Permission-based properties
  canAnalyze: boolean;
  canExport: boolean;
  canScreen: boolean;
}
```

### 2. TokenGatedFeature Component
```tsx
interface TokenGatedFeatureProps {
  feature: TokenFeature;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  
  // NEW: Resource-based access control
  resource?: string;
  action?: string;
}
```

### 3. Permission-Aware Access Hook
```typescript
interface PermissionAwareAccess {
  // Legacy interface compatibility
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  
  // NEW: Permission-based interface
  permissions: {
    canRead: boolean;
    canAnalyze: boolean;
    canExport: boolean;
    canScreen: boolean;
    canManage: boolean;
  };
  
  // Access control functions
  canAccessResource: (resource: string, action: string) => boolean;
  canAccessRankings: (limit: number) => boolean;
}
```

## Testing Strategy

### Unit Tests
```typescript
// Test permission evaluation
describe('PermissionService', () => {
  it('should allow access with correct permissions', () => {
    // Test permission evaluation logic
  });
  
  it('should deny access without permissions', () => {
    // Test denial logic
  });
});
```

### Integration Tests
```typescript
// Test component integration
describe('TokenGatedFeature', () => {
  it('should render children when user has permission', () => {
    // Test component rendering with permissions
  });
  
  it('should show upgrade prompt when user lacks permission', () => {
    // Test fallback behavior
  });
});
```

## Rollback Plan

### If Migration Fails
1. **Database**: Keep legacy tables intact during migration
2. **Hooks**: Legacy hooks remain functional
3. **Components**: Backward compatibility maintained
4. **Feature Flags**: Can disable new system via environment variables

### Environment Variables
```bash
# Control migration phases
ENABLE_NEW_PERMISSION_SYSTEM=false
ENABLE_PERMISSION_LOGGING=true
MIGRATION_PHASE=1
```

## Performance Considerations

### Caching Strategy
- Permission checks cached per user session
- Policy evaluation results cached for 5 minutes
- Database queries optimized with proper indexes

### Monitoring
- Permission check latency monitoring
- Cache hit rate tracking
- Error rate monitoring

## Security Considerations

### Audit Logging
- All permission checks logged
- Failed access attempts tracked
- Admin actions audited

### Data Protection
- Sensitive permission data encrypted
- Database access restricted
- Regular security audits

## Next Steps

1. **Complete Database Migration**: Run migration scripts in staging
2. **Update Remaining Components**: Migrate all permission-related components
3. **Add Tests**: Comprehensive test coverage for new system
4. **Performance Testing**: Load testing with new permission system
5. **Security Audit**: Review new permission model for security issues
6. **Documentation**: Update API documentation with new permission model

## Troubleshooting

### Common Issues
1. **Import Errors**: Module not found errors when importing new permission service
   - Solution: Ensure proper package exports and tsconfig paths
   
2. **Type Errors**: TypeScript compilation errors
   - Solution: Update type definitions and interfaces
   
3. **Performance Issues**: Slow permission checks
   - Solution: Implement caching and optimize database queries

### Debug Commands
```bash
# Check permission service status
npm run permission:status

# Run permission tests
npm run test:permissions

# Clear permission cache
npm run permission:clear-cache
```

## Support

For questions or issues during migration:
- Check the README.md in `/packages/auth/src/permission-service/`
- Review the implementation examples in the codebase
- Contact the development team for assistance
