# Dynamic Template Integration Guide

This guide explains how to use the enhanced permission service with dynamic template support.

## Overview

The `@epsx/auth` package now supports dynamic templates alongside the existing AWS IAM-inspired permission system. Templates are converted to IAM-style policies and integrated seamlessly with existing permission evaluation.

## Setup

### 1. Initialize the Permission Service with Template Support

```typescript
import { PermissionService, createTemplateIntegratedService } from '@epsx/auth';
import { dynamicTemplateService } from './your-template-service';

// Get the permission service instance
const permissionService = PermissionService.getInstance();

// Set up template integration
permissionService.setTemplateService(dynamicTemplateService);
```

### 2. Using Template-Aware Hooks

```tsx
import { 
  useTemplatePermissions, 
  useEffectivePermissions,
  useResourcePermission 
} from '@epsx/auth';

function UserDashboard({ userId }: { userId: string }) {
  // Get template-specific permissions
  const { templatePermissions, loading, hasTemplatePermission } = useTemplatePermissions(userId);
  
  // Get combined static + template permissions
  const { effectivePermissions } = useEffectivePermissions(userId);
  
  // Check specific resource permission
  const { hasPermission: canViewAnalytics } = useResourcePermission(
    userId, 
    'analytics', 
    'view'
  );

  if (loading) return <div>Loading permissions...</div>;

  return (
    <div>
      {hasTemplatePermission('dashboard.view') && (
        <DashboardWidget />
      )}
      
      {canViewAnalytics && (
        <AnalyticsSection />
      )}
      
      {/* Show active templates */}
      <div className="mt-4">
        <h3>Active Templates: {templatePermissions?.templateSources.length || 0}</h3>
        {templatePermissions?.templateSources.map(source => (
          <div key={source.templateId}>
            {source.templateName} - {source.contributedPermissions.length} permissions
          </div>
        ))}
      </div>
    </div>
  );
}
```

### 3. Using Template-Aware Permission Gates

```tsx
import { 
  TemplatePermissionGate, 
  TemplateConflictWarning,
  ActiveTemplatesDisplay,
  EnhancedPermissionGate 
} from '@epsx/auth';

function ProtectedFeature({ userId }: { userId: string }) {
  return (
    <div>
      {/* Show template conflicts if any */}
      <TemplateConflictWarning userId={userId}>
        
        {/* Basic template permission gate */}
        <TemplatePermissionGate 
          userId={userId}
          permission="admin.users.view"
          fallback={<div>Access denied</div>}
        >
          <UserManagementPanel />
        </TemplatePermissionGate>

        {/* Resource-based permission gate */}
        <TemplatePermissionGate
          userId={userId}
          resource="analytics"
          action="export"
          showSource={true}
          fallback={<div>Cannot export analytics</div>}
        >
          <ExportButton />
        </TemplatePermissionGate>

        {/* Enhanced gate with debugging info */}
        <EnhancedPermissionGate
          userId={userId}
          permission="reports.create"
          showDetails={true}
          fallback={<div>Cannot create reports</div>}
        >
          <CreateReportButton />
        </EnhancedPermissionGate>

      </TemplateConflictWarning>

      {/* Show active templates */}
      <ActiveTemplatesDisplay 
        userId={userId} 
        className="mt-4 p-4 border rounded"
      />
    </div>
  );
}
```

### 4. Programmatic Permission Checking

```typescript
import { PermissionService } from '@epsx/auth';

async function checkUserAccess(userId: string) {
  const permissionService = PermissionService.getInstance();
  
  // Check specific permission with templates
  const canManageUsers = await permissionService.hasPermissionWithTemplates(
    userId,
    'admin:users',
    'manage'
  );
  
  // Get detailed permission information
  const effectivePermissions = await permissionService.getEffectivePermissions(userId);
  
  console.log('Static permissions:', effectivePermissions.staticPermissions);
  console.log('Template permissions:', effectivePermissions.templatePermissions);
  console.log('Combined policies:', effectivePermissions.combinedPolicies);
  
  // Get template evaluation details
  const templateResult = await permissionService.evaluateUserTemplates(userId);
  
  console.log('Active templates:', templateResult.templateSources);
  console.log('Conflicts:', templateResult.conflicts);
  
  return {
    canManageUsers,
    activeTemplates: templateResult.templateSources.length,
    hasConflicts: templateResult.conflicts.length > 0
  };
}
```

### 5. Template Service Integration

To integrate with your template service, implement the expected interface:

```typescript
interface TemplateServiceInterface {
  getUserActiveAssignments(userId: string): Promise<TemplateAssignment[]>;
  getTemplatesForAssignments(assignments: TemplateAssignment[]): Promise<DynamicTemplate[]>;
  getTemplate(templateId: string): Promise<DynamicTemplate | null>;
}

// Example implementation
class MyTemplateService implements TemplateServiceInterface {
  async getUserActiveAssignments(userId: string): Promise<TemplateAssignment[]> {
    // Fetch from your database
    return await this.db.getActiveAssignments(userId);
  }
  
  async getTemplatesForAssignments(assignments: TemplateAssignment[]): Promise<DynamicTemplate[]> {
    const templateIds = assignments.map(a => a.templateId);
    return await this.db.getTemplates(templateIds);
  }
  
  async getTemplate(templateId: string): Promise<DynamicTemplate | null> {
    return await this.db.getTemplate(templateId);
  }
}
```

## Architecture

### Template to Policy Conversion

Dynamic templates are converted to AWS IAM-style policies:

1. **Template Permissions** → **Policy Statements**
2. **Permission Scopes** → **Resource ARNs**
3. **Template Conditions** → **Policy Conditions**
4. **Inheritance** → **Multiple Policy Statements**

### Permission Evaluation Flow

1. **Load User Data** → Package tier, roles, static permissions
2. **Get Template Assignments** → Active, non-expired assignments
3. **Filter Compatible Templates** → Check package tier compatibility
4. **Resolve Inheritance** → Build complete permission chains
5. **Apply Overrides** → Assignment-specific modifications
6. **Filter by Scope** → Remove permissions outside user's scope
7. **Convert to Policies** → Transform to IAM policy format
8. **Merge with Static** → Combine with existing permissions
9. **Evaluate Request** → Standard IAM evaluation logic

### Conflict Resolution

When multiple templates provide the same permission:

- **MERGE_PERMISSIVE**: Use most permissive version
- **MERGE_RESTRICTIVE**: Use most restrictive version
- **FAIL**: Deny permission entirely
- **FIRST_WINS**: Use first template's version
- **LAST_WINS**: Use last template's version

## Best Practices

### 1. Caching

The permission service includes caching. Refresh when templates change:

```typescript
const permissionService = PermissionService.getInstance();
permissionService.refreshCache();
```

### 2. Error Handling

Always handle template evaluation errors gracefully:

```typescript
const { templatePermissions, error } = useTemplatePermissions(userId);

if (error) {
  // Log error and fallback to static permissions
  console.error('Template evaluation failed:', error);
  // Continue with static permission checking
}
```

### 3. Performance

- Template evaluation is cached for 5 minutes
- Use `useResourcePermission` for single permission checks
- Use `useEffectivePermissions` when you need multiple permission checks

### 4. Debugging

Use the debug component to troubleshoot permissions:

```tsx
import { PermissionDebugInfo } from '@epsx/auth';

<PermissionDebugInfo userId={userId} className="mt-4" />
```

## Migration from Static Permissions

1. **Keep existing code working** - The enhanced service is backward compatible
2. **Gradually adopt template-aware hooks** - Replace existing permission hooks
3. **Add template visualization** - Use `ActiveTemplatesDisplay` component
4. **Monitor conflicts** - Use `TemplateConflictWarning` component
5. **Update permission gates** - Migrate to `TemplatePermissionGate`

## Example: Complete Integration

```tsx
import React from 'react';
import { 
  PermissionService,
  useEffectivePermissions,
  TemplatePermissionGate,
  TemplateConflictWarning,
  ActiveTemplatesDisplay,
  PermissionDebugInfo
} from '@epsx/auth';
import { dynamicTemplateService } from './services/templateService';

// Initialize on app startup
const permissionService = PermissionService.getInstance();
permissionService.setTemplateService(dynamicTemplateService);

function AdminPanel({ userId }: { userId: string }) {
  const { effectivePermissions, loading } = useEffectivePermissions(userId);

  if (loading) return <div>Loading...</div>;

  return (
    <div className="p-6">
      <h1>Admin Panel</h1>
      
      {/* Show conflicts */}
      <TemplateConflictWarning userId={userId}>
        
        {/* Protected features */}
        <TemplatePermissionGate
          userId={userId}
          permission="admin.users.manage"
          fallback={<div>User management not available</div>}
        >
          <UserManagementSection />
        </TemplatePermissionGate>

        <TemplatePermissionGate
          userId={userId}
          resource="analytics"
          action="view"
          showSource={true}
        >
          <AnalyticsSection />
        </TemplatePermissionGate>

      </TemplateConflictWarning>

      {/* Show active templates */}
      <ActiveTemplatesDisplay userId={userId} className="mt-6" />

      {/* Debug info in development */}
      {process.env.NODE_ENV === 'development' && (
        <PermissionDebugInfo userId={userId} className="mt-6" />
      )}
    </div>
  );
}
```

This integration provides a seamless way to use dynamic templates while maintaining compatibility with existing AWS IAM-style permissions.