# EPSX Migration & Cleanup Plan

## 1. Current State Analysis

### Git Status Summary
- **Deleted Files**: Multiple packages, IAM components, auth modules
- **Modified Files**: Core app files, configurations
- **Untracked Files**: New auth implementations, domain modules

### Cleanup Priorities
1. **High Priority**: Remove deleted file references
2. **Medium Priority**: Consolidate similar functionality
3. **Low Priority**: Optimize file structure

## 2. File Cleanup Tasks

### Phase 1: Remove Deleted Dependencies
**Target**: Clean up import references and dependencies

#### Deleted Packages to Clean Up
```
packages/api-client/
packages/auth/
packages/shared/
```

#### Deleted IAM Components
```
apps/admin-frontend/components/iam/
apps/admin-frontend/services/enhancedIAMService.ts
apps/admin-frontend/services/firebaseIAMService.ts
```

#### Deleted Frontend Components
```
apps/frontend/components/auth/AuthForms.tsx
apps/frontend/components/auth/ClientAuthGuard.tsx
apps/frontend/services/auth/auth.service.ts
```

### Phase 2: Consolidate Similar Files
**Target**: Reduce file count and optimize structure

#### Auth Consolidation
- Merge auth services into single service per app
- Combine auth components with similar functionality
- Standardize auth patterns across apps

#### Component Optimization
- Merge similar UI components
- Consolidate utility functions
- Optimize import paths

### Phase 3: Code Optimization
**Target**: Shortest names, minimal files

#### Naming Optimization
- **Functions**: Use shortest descriptive names
- **Variables**: Single letter where context is clear
- **Files**: Abbreviated but clear names
- **Folders**: Minimal depth structure

#### File Structure Optimization
```
Before: apps/frontend/components/auth/EmailPasswordForm.tsx
After: apps/frontend/components/auth/EmailForm.tsx

Before: usePermissionAwareAccess.ts  
After: usePerm.ts

Before: firebaseAuthIAMService.ts
After: fbAuth.ts
```

## 3. Architecture Migration

### Backend Structure Optimization
```
Current: src/web/auth/enhanced_handlers.rs
Target:  src/web/auth/handlers.rs (consolidated)

Current: Multiple auth modules
Target:  Single auth module with sub-components
```

### Frontend Structure Optimization
```
Current: Multiple auth contexts and providers
Target:  Single auth system with unified interface

Current: Separate IAM context and auth context  
Target:  Integrated permission-aware auth context
```

## 4. Migration Checklist

### ✅ Phase 1: Immediate Cleanup
- [ ] Remove deleted package imports from package.json files
- [ ] Clean up deleted component imports
- [ ] Remove unused dependency references
- [ ] Update broken import paths

### ⏳ Phase 2: Code Consolidation
- [ ] Merge auth services (admin-frontend)
- [ ] Merge auth services (frontend)  
- [ ] Consolidate auth components
- [ ] Optimize backend auth modules
- [ ] Update all import references

### 🔄 Phase 3: Optimization
- [ ] Rename functions to shortest form
- [ ] Rename variables for brevity
- [ ] Rename files for efficiency
- [ ] Optimize folder structure
- [ ] Update documentation

### 🧪 Phase 4: Testing & Validation
- [ ] Run lint checks
- [ ] Run type checks
- [ ] Run all tests
- [ ] Verify functionality
- [ ] Performance validation

## 5. Risk Mitigation

### Backup Strategy
- Create feature branch before major changes
- Incremental changes with testing
- Rollback plan for each phase

### Testing Strategy
- Test after each consolidation
- Verify auth flows work correctly
- Check all imports resolve
- Validate build processes

### Dependencies Management
- Update package.json files incrementally
- Verify no circular dependencies
- Check for unused dependencies

## 6. Token Efficiency Goals

### File Reduction Targets
- **50% reduction** in auth-related files
- **30% reduction** in component files
- **40% reduction** in service files

### Naming Efficiency
- **Average 30% shorter** function names
- **Average 25% shorter** variable names  
- **Average 35% shorter** file names

### Code Efficiency
- Remove duplicate code
- Optimize import statements
- Consolidate similar functions
- Minimize file structure depth

## 7. Success Criteria

### Functional Requirements
- All authentication flows work correctly
- All applications build successfully
- All tests pass
- No broken imports or references

### Performance Requirements
- Faster build times
- Reduced bundle sizes
- Improved code readability
- Lower token consumption

### Maintenance Requirements
- Simplified file structure
- Clear naming conventions
- Documented changes
- Updated development workflow

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-23  
**Status**: Ready for Implementation  
**Estimated Effort**: 2-3 development days  
**Dependencies**: None (can start immediately)