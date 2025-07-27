# EPSX Application Route Map

This document provides a comprehensive overview of all routes across the EPSX application ecosystem.

## Table of Contents
- [Frontend Application Routes](#frontend-application-routes)
- [Admin Frontend Routes](#admin-frontend-routes)
- [Backend API Routes](#backend-api-routes)
- [Route Summary](#route-summary)

---

## Frontend Application Routes

### Public Routes
| Route | File Path | Description |
|-------|-----------|-------------|
| `/` | `apps/frontend/app/page.tsx` | Landing/Home page with hero section, chat, data tech showcase, pricing, and public ranking preview |
| `/privacy` | `apps/frontend/app/privacy/page.tsx` | Privacy policy page |
| `/terms` | `apps/frontend/app/terms/page.tsx` | Terms of service page |
| `/access-denied` | `apps/frontend/app/access-denied/page.tsx` | Access denied page for unauthorized access attempts |
| `/unauthorized` | `apps/frontend/app/unauthorized/page.tsx` | Unauthorized access page |

### Authentication Routes
| Route | File Path | Description |
|-------|-----------|-------------|
| `/login` | `apps/frontend/app/login/page.tsx` | User login page |
| `/register` | `apps/frontend/app/register/page.tsx` | User registration page |
| `/forgot-password` | `apps/frontend/app/forgot-password/page.tsx` | Password recovery page |
| `/reset-password` | `apps/frontend/app/reset-password/page.tsx` | Password reset page |
| `/verify-email` | `apps/frontend/app/verify-email/page.tsx` | Email verification page |

### Protected Routes (Require Authentication)
| Route | File Path | Permission Required | Description |
|-------|-----------|-------------------|-------------|
| `/dashboard` | `apps/frontend/app/dashboard/page.tsx` | `route:/dashboard`, role: `user` | Main user dashboard with navigation cards, user profile, settings, analytics |
| `/analytics` | `apps/frontend/app/analytics/page.tsx` | `route:/analytics/*`, profile: `Silver User`, role: `premium` | Analytics dashboard with comprehensive stock ranking analytics |
| `/analytics/eps` | `apps/frontend/app/analytics/eps/page.tsx` | `route:/analytics/*` | EPS (Earnings Per Share) analytics page |
| `/analytics/pattern-recognition` | `apps/frontend/app/analytics/pattern-recognition/page.tsx` | `route:/analytics/*` | Pattern recognition analytics page |
| `/trading` | `apps/frontend/app/trading/page.tsx` | `route:/trading/*`, profile: `Silver User`, role: `premium` | Trading interface |
| `/my-data` | `apps/frontend/app/my-data/page.tsx` | `route:/profile/*`, role: `user` | User's personal data management page |
| `/settings` | `apps/frontend/app/settings/page.tsx` | `route:/settings`, role: `user` | User settings and preferences |

### Payment Routes
| Route | File Path | Permission Required | Description |
|-------|-----------|-------------------|-------------|
| `/payment` | `apps/frontend/app/payment/page.tsx` | `route:/payment/*`, role: `user` | Main payment page with package selection and secure payment processing |
| `/payment/quick` | `apps/frontend/app/payment/quick/page.tsx` | `route:/payment/*` | Quick payment option |
| `/payment/enterprise` | `apps/frontend/app/payment/enterprise/page.tsx` | `route:/payment/*` | Enterprise payment plans |
| `/payment/return` | `apps/frontend/app/payment/return/page.tsx` | `route:/payment/*` | Payment return/callback page |

### Admin Redirect Route
| Route | File Path | Permission Required | Description |
|-------|-----------|-------------------|-------------|
| `/admin` | `apps/frontend/app/admin/redirect-page.tsx` | `route:/admin/*`, role: `admin` | Legacy admin route that redirects to the separate admin frontend application |

### Special Files
- **Root Layout**: `apps/frontend/app/layout.tsx`
- **Loading States**: Multiple loading.tsx files for specific routes
- **Error Handling**: `apps/frontend/app/error.tsx`, `apps/frontend/app/not-found.tsx`
- **Middleware**: `apps/frontend/middleware.ts` with comprehensive security and permission handling

---

## Admin Frontend Routes

### Main Admin Pages
| Route | File Path | Description |
|-------|-----------|-------------|
| `/` | `apps/admin-frontend/app/page.tsx` | Main Dashboard - Admin home page displaying the main AdminDashboard component |
| `/login` | `apps/admin-frontend/app/login/page.tsx` | Authentication - Admin login page with email/password form |
| `/analytics` | `apps/admin-frontend/app/analytics/page.tsx` | Analytics Dashboard - Displays admin analytics and user statistics |
| `/database` | `apps/admin-frontend/app/database/page.tsx` | Database Management - Database administration interface |
| `/iam` | `apps/admin-frontend/app/iam/page.tsx` | Identity & Access Management - IAM dashboard for managing user access and permissions |
| `/settings` | `apps/admin-frontend/app/settings/page.tsx` | System Settings - Admin configuration and settings management |
| `/unauthorized` | `apps/admin-frontend/app/unauthorized/page.tsx` | Access Denied - Error page for unauthorized access attempts |

### User Management Routes
| Route | File Path | Permission Required | Description |
|-------|-----------|-------------------|-------------|
| `/users` | `apps/admin-frontend/app/users/page.tsx` | `admin.users.manage`, profile: `Admin Assistant`, role: `admin` | Main user management interface |
| `/users/roles` | `apps/admin-frontend/app/users/roles/page.tsx` | `admin.users.manage` | Dashboard for managing user roles and role assignments |
| `/users/permissions` | `apps/admin-frontend/app/users/permissions/page.tsx` | `admin.users.manage` | Interface for managing user permissions |

### Permission & Profile Management
| Route | File Path | Permission Required | Description |
|-------|-----------|-------------------|-------------|
| `/permission-profiles/assign` | `apps/admin-frontend/app/permission-profiles/assign/page.tsx` | `admin.permission_profiles.manage`, profile: `System Administrator`, role: `admin` | Tool for assigning permission profiles to users |
| `/stock-ranking-packages` | `apps/admin-frontend/app/stock-ranking-packages/page.tsx` | `admin.stock_rankings.manage`, profile: `Content Manager`, role: `admin` | Management interface for stock ranking access packages |

### Admin API Routes
| Route | File Path | Description |
|-------|-----------|-------------|
| `/api/admin/analytics/user-statistics` | `apps/admin-frontend/app/api/admin/analytics/user-statistics/route.ts` | User Statistics API - GET endpoint for fetching user analytics and statistics data |

### Admin Middleware
- **File**: `apps/admin-frontend/middleware.ts`
- **Features**: Role hierarchy, permission profiles, security headers, caching
- **Protected Routes**: All admin routes require specific permissions and minimum role levels

---

## Backend API Routes

### Public Routes (No Authentication Required)

#### Health & Status
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/health` | health_check | Health check endpoint returning service status |

#### Authentication - Public
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/login` | multi_login_handler | Multi-provider login handler |
| POST | `/register` | register_handler | User registration |
| POST | `/password-reset` | password_reset_handler | Password reset request |
| GET | `/auth/me-public` | get_public_profile | Get public user profile info |

#### Admin Authentication - Public
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/api/admin/auth/login` | admin_login | Admin login endpoint |

### Authentication Required Routes

#### Authentication - Protected
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/auth/logout` | logout_handler | User logout |
| POST | `/auth/refresh` | refresh_handler | Refresh authentication token |
| GET | `/auth/me` | get_profile | Get current user profile |
| POST | `/auth/session/clear` | clear_session | Clear user session |

#### V1 API Routes (`/api/v1/*`)

##### Authentication V1
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/api/v1/auth/login` | v1_login | V1 login endpoint |
| POST | `/api/v1/auth/register` | v1_register | V1 registration |
| POST | `/api/v1/auth/register-auto` | v1_auto_register | Auto registration |
| POST | `/api/v1/auth/password-reset` | v1_password_reset | V1 password reset |
| POST | `/api/v1/auth/logout` | v1_logout | V1 logout (auth required) |
| POST | `/api/v1/auth/refresh` | v1_refresh | V1 token refresh (auth required) |
| GET | `/api/v1/auth/profile` | v1_get_profile | V1 get profile (auth required) |
| POST | `/api/v1/auth/session/clear` | v1_clear_session | V1 clear session (auth required) |

##### Market Data V1 (Auth Required)
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/api/v1/market-data/stocks/screener` | get_screener_data | Stock screener data |
| GET | `/api/v1/market-data/stocks/eps-growth-ranking` | get_eps_growth_ranking | EPS growth rankings |
| GET | `/api/v1/market-data/symbols` | get_symbols | Available stock symbols |

##### Payments V1 (Auth Required)
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/api/v1/payments/crypto/deposit-address` | get_crypto_deposit_address | Crypto deposit address |
| POST | `/api/v1/payments/musepay/create` | create_musepay_payment | Create MusePay payment |
| POST | `/api/v1/webhooks/payments/musepay` | musepay_webhook | MusePay webhook handler |

##### System V1 (Auth Required)
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/api/v1/system/cache` | clear_cache | Clear system cache |

##### Premium V1 (Auth + Permission Required)
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/api/v1/premium/rankings` | get_premium_rankings | Premium rankings data |

##### Audit Logging V1
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/api/v1/audit/logs` | create_audit_log | Create audit log entry |
| GET | `/api/v1/audit/logs` | search_audit_logs | Search audit logs |
| GET | `/api/v1/audit/logs/:log_id` | get_audit_log | Get specific audit log |
| GET | `/api/v1/audit/statistics` | get_audit_statistics | Audit statistics |
| GET | `/api/v1/audit/export` | export_audit_logs | Export audit logs |

### Admin Routes (Authentication Required)

#### Admin Authentication
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/admin/auth/logout` | admin_logout | Admin logout |
| GET | `/admin/auth/profile` | admin_profile | Admin profile |

#### Admin Analytics
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/admin/analytics/user-statistics` | get_user_statistics | User statistics |

#### User Management
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/admin/users` | list_users | List all users |
| POST | `/admin/users` | create_user | Create new user |
| GET | `/admin/users/:user_id` | get_user | Get specific user |
| PUT | `/admin/users/:user_id` | update_user_role | Update user role |
| DELETE | `/admin/users/:user_id` | soft_delete_user | Soft delete user |
| POST | `/admin/users/batch-update-roles` | batch_update_roles | Bulk update user roles |
| GET | `/admin/users/:user_id/role-history` | get_role_history | Get user role history |

#### Permission Profile Management
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/admin/permission-profiles` | list_permission_profiles | List permission profiles |
| GET | `/admin/permission-profiles/:profile_id` | get_permission_profile | Get permission profile details |
| POST | `/admin/permission-profiles/assign` | assign_permission_profile | Assign permission profile |

#### API Admin Routes
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/api/admin/users` | api_list_users | List users (API version) |
| GET | `/api/admin/users/:id` | api_get_user | Get user by ID (API version) |
| DELETE | `/api/admin/users/:id` | api_delete_user | Delete user (API version) |

### IAM (Identity & Access Management) Routes

#### Role Management
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/iam/roles` | create_role | Create new role |
| GET | `/iam/roles` | list_roles | List all roles |
| GET | `/iam/roles/:role_id` | get_role | Get specific role |
| PUT | `/iam/roles/:role_id` | update_role | Update role |
| DELETE | `/iam/roles/:role_id` | delete_role | Delete role |

#### Policy Management
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/iam/policies` | create_policy | Create new policy |
| GET | `/iam/policies` | list_policies | List all policies |
| GET | `/iam/policies/:policy_id` | get_policy | Get specific policy |
| DELETE | `/iam/policies/:policy_id` | delete_policy | Delete policy |

#### Permission Evaluation
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/iam/evaluate` | evaluate_permissions | Evaluate permissions |

#### User Permission Overrides
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/iam/users/:user_id/overrides` | set_user_overrides | Set user permission overrides |
| GET | `/iam/users/:user_id/overrides` | get_user_overrides | Get user permission overrides |

#### User-Role Assignments
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/iam/users/:user_id/roles/:role_id` | assign_role_to_user | Assign role to user |
| DELETE | `/iam/users/:user_id/roles/:role_id` | remove_role_from_user | Remove role from user |
| GET | `/iam/users/:user_id/roles` | get_user_roles | Get user roles |

### Permission Profile Management Routes

#### CRUD Operations
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/permission-profiles/permission-profiles` | create_permission_profile | Create permission profile |
| GET | `/permission-profiles/permission-profiles` | search_permission_profiles | Search permission profiles |
| GET | `/permission-profiles/permission-profiles/:profile_id` | get_permission_profile | Get permission profile |
| PUT | `/permission-profiles/permission-profiles/:profile_id` | update_permission_profile | Update permission profile |
| DELETE | `/permission-profiles/permission-profiles/:profile_id` | delete_permission_profile | Delete permission profile |

#### Application Operations
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/permission-profiles/permission-profiles/:profile_id/apply` | apply_permission_profile | Apply permission profile |
| GET | `/permission-profiles/permission-profiles/:profile_id/history` | get_application_history | Get application history |

#### System Operations
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/permission-profiles/initialize-defaults` | initialize_default_profiles | Initialize default permission profiles |

### User Management Routes

#### Legacy User Routes
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/api/me` | get_current_user | Get current user |
| PUT | `/api/me` | update_user_profile | Update user profile |
| GET | `/api/users` | list_users | List users |
| GET | `/api/users/:id` | get_user_by_id | Get user by ID |
| DELETE | `/api/users/:id` | delete_user | Delete user |
| POST | `/api/logout` | logout | Logout |

### Audit Routes

#### Audit Operations
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/audit/logs` | create_audit_log | Create audit log entry |
| GET | `/audit/logs` | search_audit_logs | Search audit logs with filters |
| GET | `/audit/logs/:log_id` | get_audit_log | Get specific audit log |
| GET | `/audit/statistics` | get_audit_statistics | Get audit statistics |
| GET | `/audit/export` | export_audit_logs | Export audit logs |

### Real-time Routes

#### WebSocket & SSE
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/realtime/ws` | websocket_handler | WebSocket connection |
| GET | `/realtime/events` | sse_handler | Server-Sent Events stream |
| GET | `/realtime/events/health` | sse_health_check | SSE health check |

#### Real-time Admin
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/realtime/admin/broadcast` | broadcast_notification | Broadcast notification |
| POST | `/realtime/admin/simulate/payment` | simulate_payment_event | Simulate payment event |
| POST | `/realtime/admin/simulate/stock` | simulate_stock_update | Simulate stock update |
| GET | `/realtime/admin/stats` | get_connection_stats | Connection statistics |
| POST | `/realtime/admin/notify/:user_id` | send_user_notification | Send user notification |

### Stock Market Data Routes

#### Stock Screener
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/market-data/stocks/screener` | get_screener_data | Stock screener data |
| GET | `/market-data/stocks/eps-growth-ranking` | get_eps_growth_ranking | EPS growth rankings |
| GET | `/market-data/stocks/screener/ws` | screener_websocket | WebSocket for screener updates |

#### Financial Data
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/market-data/financial/ws` | financial_websocket | Financial data WebSocket |
| GET | `/market-data/financial/subscribe` | subscribe_financial_data | Subscribe to financial data |

#### Price Data
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/market-data/price/ws/price` | price_websocket | Price data WebSocket |
| GET | `/market-data/price/ws/candles` | candles_websocket | Candlestick data WebSocket |
| GET | `/market-data/price/subscribe` | subscribe_price_data | Subscribe to price data |

#### Symbols
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/market-data/symbols` | get_symbols | Get available stock symbols |

### Payment Routes

#### MusePay Integration
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/payments/musepay/create` | create_payment | Create payment |
| GET | `/payments/musepay/:id` | get_payment | Get payment details |
| GET | `/payments/musepay/:id/validate` | validate_payment | Validate payment |
| GET | `/payments/musepay/:id/qrcode` | get_qr_code | Get payment QR code |

#### Crypto Payments
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| GET | `/payments/crypto/deposit-address` | get_crypto_deposit_address | Get crypto deposit address |

#### Webhooks
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/webhooks/payments/musepay` | musepay_webhook | MusePay webhook handler |

### Legacy Auth Routes

#### Token Verification
| Method | Route | Handler | Description |
|--------|-------|---------|-------------|
| POST | `/verify` | verify_token | Verify authentication token |

## WebSocket Endpoints

| Endpoint | Type | Description |
|----------|------|-------------|
| `/realtime/ws` | WebSocket | General real-time communication |
| `/market-data/stocks/screener/ws` | WebSocket | Real-time screener updates |
| `/market-data/financial/ws` | WebSocket | Financial data streaming |
| `/market-data/price/ws/price` | WebSocket | Real-time price updates |
| `/market-data/price/ws/candles` | WebSocket | Candlestick data streaming |
| `/realtime/events` | Server-Sent Events | Real-time event streaming |

---

## Route Summary

### Frontend (Next.js)
- **Total Routes**: 22 page routes
- **Public Routes**: 5
- **Auth Routes**: 5  
- **Protected Routes**: 12
- **Special Files**: Layout, loading states, error boundaries
- **Middleware**: Comprehensive security and permission handling

### Admin Frontend (Next.js)
- **Total Routes**: 12 page routes + 1 API route
- **Protected Routes**: 11 (all require admin permissions)
- **Public Routes**: 1 (login)
- **API Routes**: 1 (user statistics)
- **Middleware**: Role hierarchy and permission-based access control

### Backend (Rust/Axum)
- **Total Endpoints**: 100+ API endpoints
- **Public Endpoints**: 5
- **Auth Required**: 95+
- **WebSocket Endpoints**: 5
- **SSE Endpoints**: 2
- **Webhook Endpoints**: 1
- **Admin Endpoints**: 25+
- **IAM Endpoints**: 15+

### Security Features
- **Authentication**: Firebase Auth integration, session management
- **Authorization**: Role-based access control (RBAC), permission profiles
- **Rate Limiting**: Configurable rate limiting middleware
- **Security Headers**: CSP, XSS protection, CORS configuration
- **Audit Logging**: Comprehensive audit trail for all operations

### Architecture
- **Frontend**: Next.js 13+ with App Router, TypeScript, Tailwind CSS
- **Admin**: Separate Next.js admin application with enhanced security
- **Backend**: Rust with Axum framework, clean architecture pattern
- **Database**: PostgreSQL with migrations
- **Real-time**: WebSockets and Server-Sent Events
- **Payments**: MusePay integration, crypto payments
- **Market Data**: Real-time stock data streaming

This route map represents a comprehensive, production-ready application with modern architecture, security best practices, and scalable design patterns.

---

## API Duplication Analysis

### **⚠️ CRITICAL: Duplicate APIs Found**

The following APIs exist in both legacy and v1 versions. **These duplicates should be resolved by removing legacy versions and standardizing on v1:**

#### Authentication Duplicates
| Legacy Route | V1 Route | Status |
|-------------|----------|---------|
| `/login` | `/api/v1/auth/login` | 🔄 **DUPLICATE** |
| `/register` | `/api/v1/auth/register` | 🔄 **DUPLICATE** |
| `/password-reset` | `/api/v1/auth/password-reset` | 🔄 **DUPLICATE** |
| `/auth/logout` | `/api/v1/auth/logout` | 🔄 **DUPLICATE** |
| `/auth/refresh` | `/api/v1/auth/refresh` | 🔄 **DUPLICATE** |
| `/auth/me` | `/api/v1/auth/profile` | 🔄 **DUPLICATE** |

#### Market Data Duplicates
| Legacy Route | V1 Route | Status |
|-------------|----------|---------|
| `/market-data/stocks/screener` | `/api/v1/market-data/stocks/screener` | 🔄 **DUPLICATE** |
| `/market-data/stocks/eps-growth-ranking` | `/api/v1/market-data/stocks/eps-growth-ranking` | 🔄 **DUPLICATE** |
| `/market-data/symbols` | `/api/v1/market-data/symbols` | 🔄 **DUPLICATE** |

#### Payment Duplicates
| Legacy Route | V1 Route | Status |
|-------------|----------|---------|
| `/payments/crypto/deposit-address` | `/api/v1/payments/crypto/deposit-address` | 🔄 **DUPLICATE** |
| `/payments/musepay/create` | `/api/v1/payments/musepay/create` | 🔄 **DUPLICATE** |

#### Audit Duplicates
| Legacy Route | V1 Route | Status |
|-------------|----------|---------|
| `/audit/*` | `/api/v1/audit/*` | 🔄 **DUPLICATE** |

### **📋 Migration Required**

These routes need to be moved under v1 structure:

#### Admin Routes
| Current Route | Target V1 Route | Status |
|--------------|----------------|---------|
| `/admin/*` | `/api/v1/admin/*` | 🔀 **NEEDS MIGRATION** |
| `/api/admin/*` | `/api/v1/admin/*` | 🔀 **NEEDS MIGRATION** |

#### IAM Routes  
| Current Route | Target V1 Route | Status |
|--------------|----------------|---------|
| `/iam/*` | `/api/v1/iam/*` | 🔀 **NEEDS MIGRATION** |

#### Permission Profile Routes
| Current Route | Target V1 Route | Status |
|--------------|----------------|---------|
| `/permission-profiles/*` | `/api/v1/permission-profiles/*` | 🔀 **NEEDS MIGRATION** |

#### Legacy User Routes
| Current Route | Target V1 Route | Status |
|--------------|----------------|---------|
| `/api/me` | `/api/v1/users/profile` | 🔀 **NEEDS MIGRATION** |
| `/api/users/*` | `/api/v1/users/*` | 🔀 **NEEDS MIGRATION** |

### **✅ Recommended Action Plan**

1. **Phase 1**: Remove duplicate legacy routes, keep only v1 versions
2. **Phase 2**: Move non-versioned routes under `/api/v1/`
3. **Phase 3**: Update frontend/admin applications to use v1 endpoints exclusively
4. **Phase 4**: Update documentation and route map

### **🔧 Implementation Files**

**Backend Router**: `/apps/backend/src/web/mod.rs` (lines 228-326)
**V1 Routes**: `/apps/backend/src/web/mod.rs` (lines 64-144)
**Stock Routes**: `/apps/backend/src/stock/mod.rs` 
**Payment Routes**: `/apps/backend/src/payment/routes.rs`