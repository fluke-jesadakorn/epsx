# EPSX Migration Guide

*Complete step-by-step migration from current state to target architecture*

## Migration Overview

This guide outlines the transition from the current Firebase+Firestore system to the target architecture with Next.js SSR, Rust Clean Architecture, and PostgreSQL.

### Current State
- Next.js with client-side Firebase authentication
- Firestore for data storage
- Mixed client/server rendering
- Basic IAM with Firebase custom claims

### Target State  
- Next.js 15 with Server-Side Rendering (SSR-first)
- Rust backend with Clean Architecture
- PostgreSQL for all business data
- Firebase Auth for authentication only
- Dynamic template system with admin assignments

## Phase 1: Frontend SSR Migration (5 days)

### Day 1: Authentication System Migration

**Tasks:**
1. **Remove Firebase Client Dependencies**
   ```bash
   # Remove client-side Firebase
   pnpm remove firebase @firebase/app @firebase/auth
   
   # Update package.json
   pnpm add next@15 @types/cookies-next
   ```

2. **Implement Server-Side Auth**
   ```typescript
   // lib/auth-server.ts
   export async function getServerSideAuth(): Promise<User | null> {
     const cookieStore = cookies();
     const token = cookieStore.get('auth-token')?.value;
     
     if (!token) return null;
     
     const response = await fetch(`${process.env.BACKEND_URL}/api/v1/authentication/me`, {
       headers: { Authorization: `Bearer ${token}` }
     });
     
     return response.ok ? await response.json() : null;
   }
   ```

3. **Create Server Actions**
   ```typescript
   // app/actions/auth.ts
   'use server'
   
   export async function loginAction(formData: FormData) {
     const email = formData.get('email') as string;
     const password = formData.get('password') as string;
     
     const response = await fetch(`${process.env.BACKEND_URL}/api/v1/authentication/login`, {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ email, password })
     });
     
     if (response.ok) {
       const { token } = await response.json();
       cookies().set('auth-token', token, {
         httpOnly: true,
         secure: true,
         sameSite: 'strict',
         maxAge: 86400
       });
       redirect('/dashboard');
     } else {
       throw new Error('Authentication failed');
     }
   }
   ```

**Validation:**
- [ ] Login works with HTTP-only cookies
- [ ] Server Components can access user data
- [ ] Middleware protects routes correctly

### Day 2-3: Component Migration to SSR

**Tasks:**
1. **Convert Auth Pages to Server Components**
   ```typescript
   // app/login/page.tsx
   import { getServerSideAuth } from '@/lib/auth-server';
   import { loginAction } from '@/app/actions/auth';
   import { redirect } from 'next/navigation';
   
   export default async function LoginPage() {
     const user = await getServerSideAuth();
     if (user) redirect('/dashboard');
     
     return (
       <form action={loginAction}>
         <input name="email" type="email" required />
         <input name="password" type="password" required />
         <button type="submit">Login</button>
       </form>
     );
   }
   ```

2. **Separate Server/Client Components**
   ```bash
   mkdir -p components/server components/client components/shared
   
   # Move interactive components to client/
   mv components/auth/LoginForm.tsx components/client/
   
   # Create server versions
   touch components/server/LoginForm.server.tsx
   ```

3. **Update Component Imports**
   ```typescript
   // components/client/LoginForm.tsx
   'use client'
   
   import { useState } from 'react';
   
   export function LoginForm({ action }: { action: (formData: FormData) => void }) {
     const [loading, setLoading] = useState(false);
     // Client-side interactivity
   }
   ```

**Validation:**
- [ ] All auth pages render server-side
- [ ] Client components work with 'use client'
- [ ] Form submissions use Server Actions

### Day 4-5: State Management & API Integration

**Tasks:**
1. **Setup SSR Context Providers**
   ```typescript
   // app/layout.tsx
   import { getServerSideAuth } from '@/lib/auth-server';
   
   export default async function RootLayout({ children }) {
     const initialAuth = await getServerSideAuth();
     
     return (
       <html>
         <body>
           <AuthProvider initialAuth={initialAuth}>
             <ThemeProvider>
               {children}
             </ThemeProvider>
           </AuthProvider>
         </body>
       </html>
     );
   }
   ```

2. **Create Server Actions for Data Mutations**
   ```typescript
   // app/actions/user.ts
   'use server'
   
   export async function updateProfileAction(formData: FormData) {
     const user = await getServerSideAuth();
     if (!user) redirect('/login');
     
     const response = await fetch(`${process.env.BACKEND_URL}/api/v1/users/profile`, {
       method: 'PUT',
       headers: { 
         'Authorization': `Bearer ${await getAuthToken()}`,
         'Content-Type': 'application/json' 
       },
       body: JSON.stringify({
         name: formData.get('name'),
         preferences: JSON.parse(formData.get('preferences') as string)
       })
     });
     
     if (response.ok) {
       revalidatePath('/profile');
     }
   }
   ```

**Validation:**
- [ ] Server Components fetch data on server
- [ ] Client Components hydrate correctly
- [ ] Server Actions handle mutations properly

## Phase 2: Backend Clean Architecture (4 days)

### Day 1: Domain Layer Setup

**Tasks:**
1. **Create Core Domain Entities**
   ```rust
   // src/dom/entities/user.rs
   use uuid::Uuid;
   use chrono::{DateTime, Utc};
   
   #[derive(Debug, Clone)]
   pub struct User {
       pub id: UserId,
       pub firebase_uid: String,
       pub email: String,
       pub roles: Vec<Role>,
       pub subscription: Option<SubscriptionLevel>,
       pub created_at: DateTime<Utc>,
   }
   
   impl User {
       pub fn new(firebase_uid: String, email: String) -> Self {
           Self {
               id: UserId::new(),
               firebase_uid,
               email,
               roles: vec![Role::default_user()],
               subscription: None,
               created_at: Utc::now(),
           }
       }
   }
   ```

2. **Implement Value Objects**
   ```rust
   // src/dom/values/identifiers.rs
   use uuid::Uuid;
   
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct UserId(Uuid);
   
   impl UserId {
       pub fn new() -> Self {
           Self(Uuid::new_v4())
       }
       
       pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
           Ok(Self(Uuid::parse_str(s)?))
       }
   }
   ```

**Validation:**
- [ ] Domain entities compile without external dependencies
- [ ] Value objects enforce invariants
- [ ] Business rules are in domain layer

### Day 2-3: Infrastructure Layer Implementation

**Tasks:**
1. **Setup PostgreSQL Connection**
   ```rust
   // src/infra/db/postgres/mod.rs
   use sqlx::{PgPool, postgres::PgPoolOptions};
   
   pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
       PgPoolOptions::new()
           .max_connections(20)
           .connect(database_url)
           .await
   }
   ```

2. **Implement Repository Pattern**
   ```rust
   // src/infra/repos/user_repo.rs
   use sqlx::PgPool;
   use async_trait::async_trait;
   
   pub struct PostgresUserRepository {
       pool: PgPool,
   }
   
   #[async_trait]
   impl UserRepository for PostgresUserRepository {
       async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError> {
           let row = sqlx::query!(
               "SELECT id, firebase_uid, email, created_at FROM users WHERE firebase_uid = $1",
               firebase_uid
           )
           .fetch_optional(&self.pool)
           .await?;
           
           match row {
               Some(row) => Ok(Some(User {
                   id: UserId::from_str(&row.id.to_string())?,
                   firebase_uid: row.firebase_uid,
                   email: row.email,
                   roles: vec![], // Load separately
                   subscription: None, // Load separately
                   created_at: row.created_at,
               })),
               None => Ok(None),
           }
       }
   }
   ```

3. **Setup Database Migrations**
   ```sql
   -- migrations/001_initial_schema.sql
   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       firebase_uid VARCHAR(128) UNIQUE NOT NULL,
       email VARCHAR(255) NOT NULL,
       created_at TIMESTAMP DEFAULT NOW(),
       updated_at TIMESTAMP DEFAULT NOW()
   );
   
   CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
   ```

**Validation:**
- [ ] Database connection established
- [ ] Migrations run successfully
- [ ] Repository pattern implemented

### Day 4: Application Layer & Web Layer

**Tasks:**
1. **Implement Use Cases**
   ```rust
   // src/app/use_cases/auth.rs
   use crate::dom::entities::User;
   use crate::app::ports::UserRepository;
   
   pub struct AuthUseCase {
       user_repo: Arc<dyn UserRepository>,
       firebase_auth: Arc<dyn FirebaseAuthService>,
   }
   
   impl AuthUseCase {
       pub async fn authenticate_user(&self, token: &str) -> Result<User, AuthError> {
           // 1. Verify Firebase token
           let firebase_user = self.firebase_auth.verify_token(token).await?;
           
           // 2. Load user from PostgreSQL
           let user = self.user_repo
               .find_by_firebase_uid(&firebase_user.uid)
               .await?
               .ok_or(AuthError::UserNotFound)?;
               
           Ok(user)
       }
   }
   ```

2. **Setup Web Handlers**
   ```rust
   // src/web/auth/handlers.rs
   use axum::{Json, Extension};
   
   #[axum::debug_handler]
   pub async fn get_current_user(
       Extension(user): Extension<User>,
   ) -> Result<Json<UserResponse>, ApiError> {
       Ok(Json(UserResponse {
           id: user.id.to_string(),
           email: user.email,
           roles: user.roles.iter().map(|r| r.name.clone()).collect(),
           subscription: user.subscription.map(|s| s.to_string()),
       }))
   }
   ```

**Validation:**
- [ ] Use cases implement business logic
- [ ] Web handlers return proper responses
- [ ] Error handling works correctly

## Phase 3: Template System Implementation (3 days)

### Day 1: Template Engine Core

**Tasks:**
1. **Create Template Domain Model**
   ```rust
   // src/dom/entities/template.rs
   #[derive(Debug, Clone)]
   pub struct FeatureTemplate {
       pub id: TemplateId,
       pub name: String,
       pub category: TemplateCategory,
       pub modules: Vec<FeatureModule>,
       pub variables: Vec<TemplateVariable>,
       pub conditions: Vec<TemplateCondition>,
       pub auto_assignment_rules: Vec<AutoAssignmentRule>,
       pub status: TemplateStatus,
   }
   ```

2. **Implement Template Engine**
   ```rust
   // src/dom/services/template_engine.rs
   pub struct TemplateEngine {
       template_repo: Arc<dyn TemplateRepository>,
       condition_evaluator: Arc<ConditionEvaluator>,
   }
   
   impl TemplateEngine {
       pub async fn apply_template(
           &self,
           user_id: &UserId,
           template_id: &TemplateId,
           variables: HashMap<String, Value>
       ) -> Result<FeatureActivationResult> {
           // Implementation from IAM guide
       }
   }
   ```

**Validation:**
- [ ] Templates can be loaded and validated
- [ ] Variable substitution works
- [ ] Conditions evaluate correctly

### Day 2: Auto-Assignment System

**Tasks:**
1. **Implement Auto-Assignment Engine**
   ```rust
   // src/dom/services/auto_assignment.rs
   impl AutoAssignmentEngine {
       pub async fn process_registration(
           &self,
           user_id: &UserId,
           package_tier: &PackageTier,
           context: &RegistrationContext
       ) -> Result<AssignmentResults> {
           // Implementation from IAM guide
       }
   }
   ```

2. **Update Registration Flow**
   ```typescript
   // app/actions/auth.ts - Enhanced registration
   export async function registerAction(formData: FormData) {
     'use server'
     
     // Create user account
     const response = await fetch('/api/v1/authentication/register', {
       method: 'POST',
       body: JSON.stringify({
         email: formData.get('email'),
         password: formData.get('password'),
         package_tier: formData.get('package_tier'),
         source: 'web_registration'
       })
     });
     
     if (response.ok) {
       const result = await response.json();
       // Set auth cookie and redirect
       cookies().set('auth-token', result.token, { /* options */ });
       redirect('/dashboard?welcome=true&features=' + result.features_unlocked);
     }
   }
   ```

**Validation:**
- [ ] Registration assigns templates automatically
- [ ] Package tiers map to correct templates
- [ ] Welcome flow shows unlocked features

### Day 3: Admin Assignment System

**Tasks:**
1. **Create Admin Assignment Service**
   ```rust
   // src/app/use_cases/admin_assignment.rs
   impl AdminAssignmentService {
       pub async fn assign_template_directly(
           &self,
           admin_id: &UserId,
           request: &DirectAssignmentRequest
       ) -> Result<DirectAssignmentResult> {
           // Implementation from IAM guide
       }
   }
   ```

2. **Build Admin Dashboard Components**
   ```typescript
   // apps/admin-frontend/components/AssignmentDashboard.tsx
   export function AssignmentDashboard() {
     const handleAssignment = async (data: AssignmentData) => {
       await adminAssignTemplateAction({
         user_id: data.userId,
         template_id: data.templateId,
         assignment_type: 'promotional',
         reason: data.reason,
         expires_at: data.expiresAt
       });
     };
     
     return (
       <div>
         <UserSelector />
         <TemplateSelector />
         <AssignmentForm onSubmit={handleAssignment} />
       </div>
     );
   }
   ```

**Validation:**
- [ ] Admins can assign templates directly
- [ ] Bulk assignments work correctly
- [ ] Assignment audit trail is recorded

## Phase 4: Database Migration (2 days)

### Day 1: Schema Migration

**Tasks:**
1. **Create Complete PostgreSQL Schema**
   ```sql
   -- migrations/002_iam_tables.sql
   CREATE TABLE feature_templates (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name VARCHAR(255) NOT NULL,
       description TEXT,
       category VARCHAR(50) NOT NULL,
       version VARCHAR(20) NOT NULL,
       status VARCHAR(50) DEFAULT 'active',
       template_data JSONB NOT NULL,
       auto_assignment_rules JSONB,
       created_at TIMESTAMP DEFAULT NOW()
   );
   
   CREATE TABLE user_features (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       user_id UUID REFERENCES users(id),
       template_id UUID REFERENCES feature_templates(id),
       feature_id VARCHAR(255) NOT NULL,
       status VARCHAR(50) DEFAULT 'active',
       configuration JSONB,
       activated_at TIMESTAMP DEFAULT NOW(),
       expires_at TIMESTAMP
   );
   
   -- Additional tables from schema
   ```

2. **Run Database Migrations**
   ```bash
   # Development
   sqlx migrate run --database-url postgresql://localhost/epsx_dev
   
   # Production
   sqlx migrate run --database-url $DATABASE_URL
   ```

**Validation:**
- [ ] All tables created successfully
- [ ] Indexes are properly configured
- [ ] Foreign key constraints work

### Day 2: Data Migration

**Tasks:**
1. **Export Existing Firestore Data**
   ```typescript
   // scripts/export-firestore.ts
   import { admin } from 'firebase-admin';
   
   async function exportUsers() {
     const users = await admin.firestore().collection('users').get();
     const userData = users.docs.map(doc => ({
       id: doc.id,
       ...doc.data()
     }));
     
     fs.writeFileSync('users-export.json', JSON.stringify(userData, null, 2));
   }
   ```

2. **Import to PostgreSQL**
   ```rust
   // scripts/import-users.rs
   use sqlx::PgPool;
   use serde_json::Value;
   
   async fn import_users(pool: &PgPool, users_json: &str) -> Result<()> {
     let users: Vec<Value> = serde_json::from_str(users_json)?;
     
     for user in users {
       sqlx::query!(
         "INSERT INTO users (firebase_uid, email) VALUES ($1, $2)",
         user["firebase_uid"].as_str().unwrap(),
         user["email"].as_str().unwrap()
       )
       .execute(pool)
       .await?;
     }
     
     Ok(())
   }
   ```

**Validation:**
- [ ] All user data migrated correctly
- [ ] Data integrity maintained
- [ ] No data loss occurred

## Phase 5: Testing & Deployment (2 days)

### Day 1: Comprehensive Testing

**Tasks:**
1. **Run Test Suites**
   ```bash
   # Backend tests
   cd apps/backend
   cargo test
   
   # Frontend tests
   cd apps/frontend
   pnpm test
   pnpm test:e2e
   
   # Admin frontend tests
   cd apps/admin-frontend
   pnpm test
   ```

2. **Manual Testing Checklist**
   - [ ] User registration with template assignment
   - [ ] Login/logout flow
   - [ ] Dashboard displays correctly
   - [ ] Admin template assignment
   - [ ] Crypto payment integration
   - [ ] API endpoints respond correctly

**Validation:**
- [ ] All automated tests pass
- [ ] Manual testing complete
- [ ] Performance targets met

### Day 2: Production Deployment

**Tasks:**
1. **Deploy Backend Services**
   ```bash
   # Build Rust backend
   cargo build --release
   
   # Deploy to Vercel
   vercel deploy --prod
   ```

2. **Deploy Frontend Applications**
   ```bash
   # Deploy user frontend
   cd apps/frontend
   vercel deploy --prod
   
   # Deploy admin frontend
   cd apps/admin-frontend
   vercel deploy --prod
   ```

3. **Run Production Migrations**
   ```bash
   sqlx migrate run --database-url $PRODUCTION_DATABASE_URL
   ```

**Validation:**
- [ ] All services deployed successfully
- [ ] Database migrations completed
- [ ] Production monitoring active
- [ ] Health checks passing

## Post-Migration Checklist

### Functionality Verification
- [ ] User authentication works end-to-end
- [ ] Registration assigns templates automatically
- [ ] Admin assignment system functional
- [ ] Payment integration operational
- [ ] Real-time features working
- [ ] Mobile responsiveness maintained

### Performance Verification  
- [ ] Page load times <2 seconds
- [ ] API response times <500ms
- [ ] Database queries optimized
- [ ] SSR rendering performance
- [ ] Memory usage within limits

### Security Verification
- [ ] Authentication tokens secure
- [ ] Authorization rules enforced
- [ ] Data encryption enabled
- [ ] Audit logging functional
- [ ] CORS policies configured
- [ ] Rate limiting active

### Monitoring Setup
- [ ] Error tracking configured
- [ ] Performance monitoring active
- [ ] Database health monitoring
- [ ] User activity tracking
- [ ] Business metrics collection

## Rollback Strategy

### Emergency Rollback Plan
1. **Immediate Actions** (< 15 minutes)
   - Revert DNS to previous environment
   - Disable new feature flags
   - Restore previous deployment

2. **Data Rollback** (< 1 hour)
   - Restore database from backup
   - Re-import critical data
   - Verify data integrity

3. **Communication Plan**
   - Notify stakeholders immediately
   - Prepare user communications
   - Document issues and lessons learned

### Rollback Triggers
- Authentication failure rate > 5%
- API error rate > 10%
- Database connection issues
- Critical functionality broken
- Performance degradation > 50%

---

**Document Version**: 2.1  
**Last Updated**: 2025-01-24  
**Status**: Complete Migration Guide  
**Total Timeline**: 16 development days  
**Risk Level**: Medium (comprehensive testing and rollback plans mitigate risks)