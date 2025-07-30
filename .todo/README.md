# Backend-Centralized Auth/IAM/AC Migration Tasks

## Overview
This folder contains detailed task breakdowns for migrating all authentication, authorization, IAM, and access control logic to the backend while keeping Firebase Analytics in frontends.

## Task Files Structure

### Core Migration Phases
- `phase1-backend-api-foundation.md` - Backend API endpoints and services
- `phase2-frontend-simplification.md` - Simplify frontend auth logic
- `phase3-session-unification.md` - Centralize session management
- `phase4-permission-consolidation.md` - Unify permission validation

### Implementation Details
- `backend-changes.md` - Detailed backend modifications
- `frontend-changes.md` - Frontend simplification tasks
- `admin-frontend-changes.md` - Admin-specific modifications
- `shared-package-changes.md` - Updates to shared packages

### Testing & Validation
- `testing-strategy.md` - Comprehensive testing approach
- `migration-checklist.md` - Validation checklist for each phase

## Key Principles

### ✅ Move to Backend
- All auth/IAM/AC decision making
- Session management and validation
- Permission checking and role hierarchy
- Route protection validation
- Cookie management and security
- CSRF protection and rate limiting

### 🚫 Keep in Frontend
- Firebase Analytics integration
- UI state management (theme, navigation)
- Client-side performance optimizations
- Form validation (non-auth)
- Component rendering logic

## Progress Tracking
Each task file contains:
- [ ] Task checkboxes for easy tracking
- Implementation details and code examples
- File paths and specific changes needed
- Dependencies between tasks
- Testing requirements

## Getting Started
1. Start with Phase 1: Backend API Foundation
2. Complete each phase sequentially
3. Test thoroughly before moving to next phase
4. Use migration checklist to validate each step

## Timeline Estimate
- **Phase 1**: 1 week (Backend APIs)
- **Phase 2**: 1 week (Frontend simplification)  
- **Phase 3**: 1 week (Session unification)
- **Phase 4**: 1 week (Permission consolidation)
- **Total**: 4 weeks + testing

## Success Metrics
- [ ] 60% reduction in frontend auth code
- [ ] Single source of truth for auth decisions
- [ ] All auth logic centralized in backend
- [ ] Firebase Analytics preserved in frontends
- [ ] Improved security with server-side validation
- [ ] Unified session management across apps