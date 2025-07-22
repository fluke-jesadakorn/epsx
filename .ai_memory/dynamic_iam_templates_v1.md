# Dynamic IAM Templates Implementation - Memory Bank v1.0

## Project Overview
Implementing dynamic custom IAM templates that can be created in admin-frontend and consumed by the main frontend application.

## Current Status: Planning Phase Complete
- ✅ Plan created with 10 main tasks
- ⏳ Ready to begin implementation

## Implementation Plan

### Phase 1: Foundation & Analysis (Priority: High)
1. **[PENDING]** Analyze existing IAM system in admin-frontend to understand current template structure
2. **[PENDING]** Design dynamic template schema with configurable permissions and metadata

### Phase 2: Admin Frontend Implementation (Priority: Medium)
3. **[PENDING]** Create template management UI in admin-frontend for CRUD operations
4. **[PENDING]** Implement template storage and retrieval in Firestore
5. **[PENDING]** Create template validation and permission checking logic
6. **[PENDING]** Add template assignment functionality to user management

### Phase 3: Frontend Integration (Priority: High)
7. **[PENDING]** Implement template consumption in frontend auth middleware
8. **[PENDING]** Update shared auth package to support dynamic templates

### Phase 4: Enhancement Features (Priority: Low)
9. **[PENDING]** Create template preview and testing functionality
10. **[PENDING]** Add audit logging for template changes and assignments

## Key Architecture Decisions

### Template Schema Structure (To Be Refined)
```typescript
interface CustomTemplate {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  conditions: TemplateCondition[];
  metadata: TemplateMetadata;
  version: number;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  isActive: boolean;
}
```

### Key Components to Build
- **Template Builder**: Visual permission designer in admin-frontend
- **Template Engine**: Runtime permission evaluation in frontend
- **Sync Mechanism**: Real-time template updates across applications
- **Fallback System**: Default templates when custom ones fail

## File Locations (To Be Created/Modified)

### Admin Frontend
- `apps/admin-frontend/components/iam/template-management/` - Template CRUD UI
- `apps/admin-frontend/services/templateService.ts` - Template API calls
- `apps/admin-frontend/types/template.ts` - Template type definitions

### Shared Packages
- `packages/auth/src/templates/` - Template evaluation logic
- `packages/types/src/template.ts` - Shared template types

### Frontend
- `apps/frontend/middleware.ts` - Updated to use dynamic templates
- `apps/frontend/context/auth-context.tsx` - Template-aware auth context

### Firestore Collections
- `customTemplates/` - Template definitions
- `userTemplateAssignments/` - User-template mappings

## Current Todo List State
All 10 tasks are in "pending" status, ready for implementation to begin.

## Next Steps When Resuming
1. Start with Task 1: Analyze existing IAM system
2. Look at current files:
   - `apps/admin-frontend/components/iam/`
   - `apps/admin-frontend/services/iamService.ts`
   - `config/iam/default-roles.ts`
   - `packages/auth/src/`

## Environment Context
- Working directory: `/Users/fluke/Desktop/Work/Outsource/epsx`
- Branch: development
- Monorepo: Turborepo + PNPM workspaces
- Key packages: @epsx/auth, @epsx/types, @epsx/ui
- Apps: admin-frontend (3001), frontend (3000)

## Version History
- v1.0 (2025-07-22): Initial planning phase complete, ready for implementation

## Last Updated
Date: 2025-07-22
Phase: Planning Complete - Ready for Implementation
Version: 1.0