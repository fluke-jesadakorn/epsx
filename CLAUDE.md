# EPSX Claude Memory & Orchestration

## Project Overview
**EPSX**: EPS growth-based trading platform with mobile-first design, IAM core module, and PancakeSwap-inspired theming.

### Core Objectives
- Enable easy stock trading through strategic guidance
- EPS growth analysis without user investment knowledge
- Mobile-first responsive design
- Plugin architecture for future expansion
- 20K+ user scalability target

## Current Status

### ✅ Phase 1: Project Setup & Documentation (COMPLETED)
**Date Completed**: 2025-01-23  
**Status**: 100% Complete

#### Achievements
- **Documentation Structure**: Created `.requirement_docs/` with comprehensive docs
- **Business Requirements**: EPS trading platform goals, compliance strategy
- **Development Requirements**: Tech stack, architecture, scalability specs
- **Migration Plan**: Cleanup strategy and optimization roadmap  
- **Architecture Design**: Hexagonal + Clean Code backend structure
- **IAM Design**: Core module design for cross-project reusability
- **Code Cleanup**: Removed 180+ unnecessary files, added structured code

#### Technical Outcomes
- Git state cleaned (all deleted files removed)
- Hexagonal architecture backend implemented
- Token-optimized codebase structure
- Comprehensive documentation foundation

#### Files Created
```
.requirement_docs/
├── business-requirements.md    # Business goals, compliance, success metrics
├── development-requirements.md # Tech stack, architecture, scalability
├── migration-plan.md          # Cleanup tasks, optimization strategy
├── architecture-design.md     # System design, patterns, structure
└── iam-design.md             # IAM core module specification
```

## 🔄 Phase 2: Architecture Enhancement (CURRENT)

### Next Tasks
1. **Backend Architecture Enhancement**
   - Enhance hexagonal + clean code structure
   - Prepare database abstraction layer (Firebase → others)
   - Implement microservices-ready patterns
   - Add comprehensive error handling

2. **Frontend Architecture**
   - Establish monorepo shared design system
   - Implement SSR-first approach with cookies
   - Create mobile-first responsive framework

### Commands & Scripts
```bash
# Development
pnpm dev                # Start all development servers
pnpm dev:frontend       # Frontend only (port 3000)
pnpm dev:admin         # Admin only (port 3001)
pnpm dev:backend       # Backend only (Rust)

# Quality Assurance
pnpm lint              # Check all projects
pnpm type-check        # Type checking
pnpm test              # Run all tests

# Build & Deploy
pnpm build             # Build everything
pnpm build:frontend    # Build frontend
pnpm build:admin       # Build admin
pnpm build:backend     # Build backend (Rust)
```

### Known Issues
- Type errors in `packages/utils/src/lib/YHprice.ts` (Yahoo Finance API types)
- Need to fix import references after package cleanup

## 📋 Future Phases

### Phase 3: Design System & Theming
- PancakeSwap-inspired theme system
- Variance-based theming
- Mobile-first responsive components
- Shared UI library across monorepo

### Phase 4: IAM Core Module
- Extract IAM logic into standalone module
- Plugin architecture implementation  
- Role-based access control
- Permission template system

### Phase 5: Testing Infrastructure
- Jest unit tests (frontend)
- Playwright E2E tests
- Rust unit/integration tests
- CI/CD pipeline setup

### Phase 6: Stock Trading Features
- EPS growth strategy implementation
- Stock recommendation system
- Buy/sell interface (non-financial wording)
- Subscription management

## 🏗️ Architecture Context

### Backend (Rust)
```
src/
├── web/           # Web layer (routes, handlers, middleware)
├── app/           # Application layer (use cases, DTOs, ports)
├── dom/           # Domain layer (entities, values, services)
└── infra/         # Infrastructure layer (repos, services, events)
```

### Frontend Stack
- **Framework**: Next.js 15 with App Router
- **Language**: TypeScript
- **Styling**: Tailwind CSS + Radix UI
- **Auth**: Firebase Auth + Cookies
- **State**: Context + Hooks

### Monorepo Structure
```
epsx/
├── apps/
│   ├── frontend/          # User trading platform
│   ├── admin-frontend/    # Admin dashboard  
│   └── backend/          # Rust API server
├── packages/             # Shared libraries (future)
├── .requirement_docs/    # Documentation
└── configs/             # Shared configurations
```

## 🎯 Success Criteria
- [ ] Mobile-first responsive design
- [ ] 20K+ user scalability
- [ ] Plugin-ready architecture  
- [ ] Comprehensive testing coverage
- [ ] Clean, optimized codebase
- [ ] Complete documentation

## 📝 Development Notes

### Code Style Requirements
- **Naming**: Shortest possible function/variable/file names
- **File Structure**: Minimal files, maximum consolidation
- **Performance**: Optimized for token efficiency
- **Testing**: Jest (frontend) + Playwright (E2E) + Rust tests

### Current Working Directory Context
- Main branch: `development`
- Recent cleanup removed packages: `auth`, `api-client`, `shared`
- New hexagonal architecture implemented in backend
- Firebase integration maintained with abstraction layer

## 📋 Task Management

### Progress Tracking
For detailed progress tracking, task completion status, and phase management, see:
**File**: `PROJECT_PROGRESS.md`

**Usage**: When executing tasks, always update progress in `PROJECT_PROGRESS.md` by checking off completed items and updating progress percentages.

### Task Execution Commands
When user requests task execution, refer to `PROJECT_PROGRESS.md` for:
- Current phase status and next tasks
- Detailed task breakdowns with checkboxes
- Success criteria and milestones
- Progress metrics and completion tracking

## 🔍 Last Session Summary
**Date**: 2025-01-23  
**Focus**: Project setup and task tracking  
**Completed**: Phase 1 entirely + created comprehensive progress tracker (`PROJECT_PROGRESS.md`)  
**Next**: Execute Phase 2 tasks as directed, updating progress tracker

---

**Last Updated**: 2025-01-23  
**Phase**: 2 (Architecture Enhancement)  
**Priority**: Backend structure optimization and frontend design system  
**Progress Tracker**: `PROJECT_PROGRESS.md` (32% overall completion)