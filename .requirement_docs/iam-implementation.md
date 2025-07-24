# EPSX IAM Implementation Guide

*Complete Identity & Access Management system with dynamic template assignments*

## System Overview

### Core Architecture
- **Modular Design**: Standalone IAM system with clear domain boundaries
- **Firebase Auth Only**: Email/password authentication, PostgreSQL for all business data
- **Dynamic Templates**: Admin-created templates for flexible feature management
- **Clean Architecture**: Domain-driven design with hexagonal patterns
- **Educational Compliance**: Built-in disclaimers and audit trails

### Key Components
```typescript
interface User {
  id: string;
  firebase_uid: string;     // Links to Firebase Auth
  email: string;
  roles: Role[];
  permissions: Permission[];
  subscription?: SubscriptionLevel;
  compliance: ComplianceProfile;
  created_at: Date;
}

interface DynamicFeatureTemplate {
  id: string;
  name: string;
  category: 'module' | 'feature' | 'integration';
  modules: FeatureModule[];
  permissions: AnalyticsPermission[];
  variables: TemplateVariable[];
  pricing_tier: PricingTier;
  auto_assignment_rules: AutoAssignmentRule[];
  compliance_level: ComplianceLevel;
}
```

## Authentication System

### Firebase Auth + PostgreSQL Integration
```rust
// Backend authentication flow
pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = extract_bearer_token(&headers)?;
    
    // 1. Verify Firebase token (authentication only)
    let firebase_user = auth_service.verify_firebase_token(token).await?;
    
    // 2. Load complete user data from PostgreSQL
    let user = auth_service.load_user_from_postgres(&firebase_user.uid).await?;
    
    // 3. Inject user into request context
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
```

### SSR Authentication (Frontend)
```typescript
// Server-side auth with HTTP-only cookies
export async function getServerSideAuth(): Promise<User | null> {
  const cookieStore = cookies();
  const token = cookieStore.get('auth-token')?.value;
  
  if (!token) return null;
  
  // Verify token and load user data from backend
  const user = await fetch('/api/v1/authentication/me', {
    headers: { Authorization: `Bearer ${token}` }
  });
  
  return user.ok ? await user.json() : null;
}

// Server Action for login
export async function loginAction(formData: FormData) {
  'use server'
  
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;
  
  // Authenticate with backend
  const response = await fetch('/api/v1/authentication/login', {
    method: 'POST',
    body: JSON.stringify({ email, password })
  });
  
  if (response.ok) {
    const { token } = await response.json();
    
    // Set HTTP-only cookie
    cookies().set('auth-token', token, {
      httpOnly: true,
      secure: true,
      sameSite: 'strict',
      maxAge: 86400 // 24 hours
    });
    
    redirect('/dashboard');
  }
}
```

## Dynamic Template System

### Template Architecture
```rust
// Core template engine
pub struct TemplateEngine {
    template_repo: Arc<dyn TemplateRepository>,
    user_repo: Arc<dyn UserRepository>,
    condition_evaluator: Arc<ConditionEvaluator>,
}

impl TemplateEngine {
    pub async fn apply_template_to_user(
        &self, 
        user_id: &UserId, 
        template_id: &TemplateId,
        variables: HashMap<String, Value>
    ) -> Result<FeatureActivationResult> {
        // 1. Load template and validate conditions
        let template = self.template_repo.find_by_id(template_id).await?;
        let user = self.user_repo.find_by_id(user_id).await?;
        
        // 2. Evaluate template conditions
        let conditions_met = self.condition_evaluator
            .evaluate_conditions(&template.conditions, &user, &variables)
            .await?;
            
        if !conditions_met {
            return Err(TemplateError::ConditionsNotMet);
        }
        
        // 3. Apply template with variable substitution
        let features_unlocked = self.process_template_modules(
            &template.modules, 
            &variables
        ).await?;
        
        // 4. Create user feature assignments
        for feature in &features_unlocked {
            self.assign_feature_to_user(user_id, feature).await?;
        }
        
        // 5. Record activation for audit
        self.record_template_activation(user_id, template_id, &features_unlocked).await?;
        
        Ok(FeatureActivationResult {
            activation_id: generate_id(),
            template_id: template_id.clone(),
            user_id: user_id.clone(),
            features_unlocked,
            activation_status: ActivationStatus::Active,
        })
    }
}
```

### Auto-Assignment During Registration
```rust
// Auto-assignment engine for new user registration
pub struct AutoAssignmentEngine {
    template_repo: Arc<dyn TemplateRepository>,
    assignment_repo: Arc<dyn AssignmentRepository>,
}

impl AutoAssignmentEngine {
    pub async fn process_registration(
        &self, 
        user_id: &UserId, 
        package_tier: &PackageTier,
        context: &RegistrationContext
    ) -> Result<AssignmentResults> {
        // 1. Get auto-assignment rules for package tier
        let rules = self.get_auto_assignment_rules(package_tier).await?;
        
        // 2. Evaluate registration triggers (email domain, referral, etc.)
        let triggered_templates = self.evaluate_registration_triggers(context).await?;
        
        // 3. Merge and prioritize templates
        let templates_to_assign = self.merge_template_assignments(rules, triggered_templates)?;
        
        // 4. Apply templates with appropriate variables
        let mut results = Vec::new();
        for assignment in templates_to_assign {
            let result = self.assign_template_to_user(user_id, &assignment).await?;
            results.push(result);
        }
        
        // 5. Record registration event for analytics
        self.record_registration_event(user_id, package_tier, context, &results).await?;
        
        Ok(AssignmentResults { assignments: results })
    }
}
```

## Admin Assignment System

### Direct Template Assignment
```typescript
// Admin dashboard for direct template assignment
export function AdminAssignmentDashboard() {
  const handleDirectAssignment = async (assignmentData: DirectAssignmentData) => {
    const request: DirectAssignmentRequest = {
      user_id: assignmentData.userId,
      template_id: assignmentData.templateId,
      assignment_type: 'promotional',
      reason: assignmentData.reason,
      expires_at: assignmentData.expiresAt,
      variables: assignmentData.variables,
      notification_settings: {
        notify_user: true,
        include_tutorial: true,
        email_template: 'admin_assignment_welcome',
      },
    };
    
    // Server Action for admin assignment
    const result = await adminAssignTemplateAction(request);
    showAssignmentSuccess(result);
  };
  
  return (
    <div className="admin-assignment-dashboard">
      <TemplateSelector />
      <UserSelector />
      <AssignmentConfiguration onAssign={handleDirectAssignment} />
      <AssignmentAnalytics />
    </div>
  );
}

// Server Action for admin assignment
export async function adminAssignTemplateAction(request: DirectAssignmentRequest) {
  'use server'
  
  const admin = await getServerSideAuth();
  if (!admin || !hasAdminPermission(admin, 'template_assignment')) {
    throw new Error('Unauthorized');
  }
  
  const response = await fetch('/api/admin/user-management/assign-template', {
    method: 'POST',
    headers: { 
      'Authorization': `Bearer ${await getAuthToken()}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(request)
  });
  
  if (!response.ok) {
    throw new Error('Assignment failed');
  }
  
  const result = await response.json();
  
  // Revalidate relevant pages
  revalidatePath('/admin/assignments');
  revalidatePath(`/admin/users/${request.user_id}`);
  
  return result;
}
```

### Bulk Assignment Operations
```rust
// Backend bulk assignment service
impl AdminAssignmentService {
    pub async fn bulk_assign_template(
        &self, 
        admin_id: &UserId, 
        request: &BulkAssignmentRequest
    ) -> Result<BulkAssignmentResult> {
        // 1. Validate admin permissions
        self.validate_bulk_assignment_permission(admin_id, &request.template_id).await?;
        
        // 2. Process assignments in parallel (with rate limiting)
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        
        let semaphore = Semaphore::new(10); // Limit concurrent operations
        let tasks = request.user_ids.iter().map(|user_id| {
            let semaphore = &semaphore;
            let assignment_service = self.clone();
            let individual_request = DirectAssignmentRequest {
                user_id: user_id.clone(),
                template_id: request.template_id.clone(),
                assignment_type: request.assignment_type.clone(),
                reason: request.reason.clone(),
                expires_at: request.expires_at,
                variables: request.variables.clone(),
                notification_settings: request.notification_settings.clone(),
            };
            
            async move {
                let _permit = semaphore.acquire().await?;
                assignment_service.assign_template_directly(admin_id, &individual_request).await
            }
        });
        
        let results = join_all(tasks).await;
        
        // 3. Categorize results
        for (idx, result) in results.into_iter().enumerate() {
            match result {
                Ok(assignment_result) => successful.push(assignment_result.assignment_id),
                Err(e) => failed.push(FailedAssignment {
                    user_id: request.user_ids[idx].clone(),
                    error: e.to_string(),
                    reason: classify_assignment_error(&e),
                }),
            }
        }
        
        // 4. Record bulk operation for audit
        self.record_bulk_assignment_operation(admin_id, request, &successful, &failed).await?;
        
        Ok(BulkAssignmentResult {
            total_users: request.user_ids.len(),
            successful_assignments: successful.len(),
            failed_assignments: failed,
            assignment_ids: successful,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}
```

## Crypto Payment Integration

### Payment-Based Feature Activation
```typescript
// Crypto payment flow for feature unlocking
interface CryptoPaymentRequest {
  template_id: string;
  user_id: string;
  payment_method: {
    currency: 'USDT' | 'USDC' | 'ETH' | 'BTC';
    network: 'TRC20' | 'BSC' | 'ERC20' | 'Arbitrum' | 'Polygon';
  };
}

export async function initiateCryptoPaymentAction(request: CryptoPaymentRequest) {
  'use server'
  
  // 1. Get dynamic pricing quote
  const priceQuote = await fetch('/api/v1/payments/crypto-quote', {
    method: 'POST',
    body: JSON.stringify({
      template_id: request.template_id,
      currency: request.payment_method.currency,
      network: request.payment_method.network
    })
  });
  
  // 2. Create payment through existing MusePay service
  const payment = await fetch('/api/v1/payments/crypto-payments/initiate', {
    method: 'POST',
    body: JSON.stringify({
      ...request,
      quote_id: priceQuote.quote_id,
      metadata: {
        template_id: request.template_id,
        feature_unlock: true
      }
    })
  });
  
  const paymentResult = await payment.json();
  
  return {
    payment_id: paymentResult.id,
    qr_code: paymentResult.qr_code,
    payment_address: paymentResult.address,
    amount: paymentResult.amount,
    expires_at: paymentResult.expires_at
  };
}
```

### Webhook Processing for Feature Activation
```rust
// Webhook handler for automatic feature activation
#[axum::debug_handler]
pub async fn process_payment_webhook(
    State(service): State<PaymentWebhookService>,
    Json(webhook_data): Json<PaymentWebhookData>,
) -> Result<Json<WebhookResponse>, ApiError> {
    // 1. Verify webhook signature
    service.verify_webhook_signature(&webhook_data).await?;
    
    if webhook_data.status == PaymentStatus::Completed 
        && webhook_data.metadata.get("feature_unlock").is_some() {
        
        // 2. Extract template information
        let template_id = webhook_data.metadata
            .get("template_id")
            .ok_or(ApiError::BadRequest("Missing template_id".to_string()))?;
            
        // 3. Activate features for the user
        let activation_result = service.template_engine
            .activate_feature_on_payment(&webhook_data.payment_id, template_id)
            .await?;
            
        // 4. Send activation notification
        service.notification_service
            .send_feature_activation_notification(
                &webhook_data.user_id,
                &activation_result.features_unlocked
            )
            .await?;
            
        // 5. Record payment and activation for analytics
        service.record_payment_activation(&webhook_data, &activation_result).await?;
        
        Ok(Json(WebhookResponse {
            payment_processed: true,
            features_activated: true,
            activation_details: Some(activation_result),
        }))
    } else {
        Ok(Json(WebhookResponse {
            payment_processed: true,
            features_activated: false,
            activation_details: None,
        }))
    }
}
```

## Database Schema

### Core IAM Tables
```sql
-- Users linked to Firebase UID
users (
  id UUID PRIMARY KEY,
  firebase_uid VARCHAR(128) UNIQUE NOT NULL,
  email VARCHAR(255) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Dynamic feature templates
feature_templates (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  category VARCHAR(50) NOT NULL,
  version VARCHAR(20) NOT NULL,
  status VARCHAR(50) DEFAULT 'active',
  template_data JSONB NOT NULL,
  pricing_tier JSONB,
  auto_assignment_rules JSONB,
  compliance_level VARCHAR(50) DEFAULT 'educational'
);

-- Template variables
template_variables (
  id UUID PRIMARY KEY,
  template_id UUID REFERENCES feature_templates(id),
  name VARCHAR(100) NOT NULL,
  type VARCHAR(50) NOT NULL,
  required BOOLEAN DEFAULT FALSE,
  default_value JSONB,
  user_configurable BOOLEAN DEFAULT FALSE
);

-- User feature assignments
user_features (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  template_id UUID REFERENCES feature_templates(id),
  feature_id VARCHAR(255) NOT NULL,
  status VARCHAR(50) DEFAULT 'active',
  configuration JSONB,
  activated_at TIMESTAMP DEFAULT NOW(),
  expires_at TIMESTAMP
);

-- Admin template assignments
admin_template_assignments (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  template_id UUID REFERENCES feature_templates(id),
  assigned_by UUID REFERENCES users(id),
  assignment_type VARCHAR(50) NOT NULL,
  assignment_reason TEXT NOT NULL,
  expires_at TIMESTAMP,
  variables JSONB,
  override_pricing BOOLEAN DEFAULT FALSE,
  internal_notes TEXT,
  status VARCHAR(50) DEFAULT 'active'
);

-- Assignment audit trail
assignment_audit_log (
  id UUID PRIMARY KEY,
  assignment_id UUID REFERENCES admin_template_assignments(id),
  action VARCHAR(50) NOT NULL,
  performed_by UUID REFERENCES users(id),
  details JSONB NOT NULL,
  timestamp TIMESTAMP DEFAULT NOW()
);

-- Crypto payment integration
feature_payments (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  template_id UUID REFERENCES feature_templates(id),
  payment_id UUID NOT NULL, -- References existing payment system
  features_unlocked JSONB NOT NULL,
  activation_status VARCHAR(50) DEFAULT 'pending',
  activated_at TIMESTAMP,
  expires_at TIMESTAMP
);
```

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_template_auto_assignment() {
    let service = setup_test_service().await;
    
    // Setup: Create Bronze tier template
    let template = create_test_template("bronze_analytics").await;
    setup_auto_assignment_rule(&template.id, PackageTier::Bronze).await;
    
    // Test: Register user with Bronze package
    let user = register_test_user(PackageTier::Bronze).await;
    
    // Verify: User has template assigned automatically
    let assignments = service.get_user_template_assignments(&user.id).await?;
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].template_id, template.id);
}

#[tokio::test]
async fn test_admin_direct_assignment() {
    let service = setup_admin_service().await;
    let admin = create_test_admin().await;
    let user = create_test_user().await;
    let template = create_test_template("premium_analytics").await;
    
    let request = DirectAssignmentRequest {
        user_id: user.id,
        template_id: template.id,
        assignment_type: AssignmentType::Trial,
        reason: "Beta program access".to_string(),
        expires_at: Some(Utc::now() + Duration::days(30)),
        variables: HashMap::new(),
    };
    
    let result = service.assign_template_directly(&admin.id, &request).await?;
    
    // Verify assignment and audit trail
    assert_eq!(result.template_id, template.id);
    assert_eq!(result.status, "active");
    
    let audit_entries = service.get_assignment_audit(&result.assignment_id).await?;
    assert!(!audit_entries.is_empty());
}
```

### Integration Tests
```typescript
// E2E test for registration with auto-assignment
describe('User Registration with Template Assignment', () => {
  test('should auto-assign templates based on package tier', async ({ page }) => {
    await page.goto('/register');
    
    // Select Bronze package tier
    await page.selectOption('[data-testid="package-tier"]', 'Bronze');
    
    // Complete registration
    await page.fill('[data-testid="email"]', 'test@example.com');
    await page.fill('[data-testid="password"]', 'SecurePass123!');
    await page.click('[data-testid="register-button"]');
    
    // Verify features were unlocked
    await expect(page.locator('[data-testid="features-unlocked"]')).toBeVisible();
    await expect(page.locator('[data-testid="template-count"]')).toContainText('3');
    
    // Verify dashboard shows assigned features
    await page.goto('/dashboard');
    await expect(page.locator('[data-testid="bronze-analytics"]')).toBeVisible();
  });
});
```

## Performance Considerations

### Optimization Strategies
- **Template Caching**: Redis-based template and condition caching
- **Bulk Operations**: Parallel processing with rate limiting
- **Database Indexing**: Optimized queries for user features and assignments
- **Condition Evaluation**: Compiled condition evaluation for performance

### Monitoring & Metrics
- **Assignment Performance**: Track assignment completion times
- **Template Usage**: Monitor template adoption and usage patterns
- **Admin Efficiency**: Measure admin operation success rates
- **Payment Integration**: Track crypto payment and activation success

---

**Document Version**: 2.1  
**Last Updated**: 2025-01-24  
**Status**: Complete IAM Implementation Guide  
**Implementation Effort**: 7-10 development days  
**Dependencies**: PostgreSQL setup, Firebase Auth, payment system integration