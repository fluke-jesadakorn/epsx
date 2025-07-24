# IAM Enhanced Template System Migration Plan

## Executive Summary

This migration plan outlines the implementation of enhanced IAM template system features, focusing on automatic template assignment during user registration and admin direct assignment capabilities. The current system already has excellent foundations - this plan extends those capabilities.

## Current State Analysis

### ✅ Already Implemented (Strengths)
- **Complete IAM Domain Model**: AWS-inspired IAM with roles, policies, groups
- **Template System Foundation**: Dynamic feature templates with variables and conditions
- **Clean Architecture**: Proper domain/app/infra separation
- **Database Schema**: PostgreSQL with comprehensive IAM tables
- **User Registration Flow**: Email validation and role assignment
- **Admin Management**: Template creation and user management
- **Crypto Payment Integration**: MusePay integration for feature purchases

### 🔄 Enhancement Areas (This Migration)
- **Auto-Assignment During Registration**: Package tier → template activation
- **Admin Direct Assignment**: Promotional and trial access without payment
- **Registration Flow Enhancement**: Seamless feature activation
- **Template Assignment Analytics**: Performance tracking and metrics

## Migration Phases

### Phase 1: Database Schema Extensions (2 days)

#### 1.1 Auto-Assignment Tables
```sql
-- Auto-assignment rules for registration flow
CREATE TABLE template_auto_assignment_rules (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id UUID REFERENCES feature_templates(id) NOT NULL,
  trigger_type VARCHAR(50) NOT NULL DEFAULT 'registration',
  package_tiers TEXT[] NOT NULL, -- Array of package tiers
  conditions JSONB,
  template_variables JSONB,
  expires_after_days INTEGER,
  priority INTEGER DEFAULT 0,
  active BOOLEAN DEFAULT TRUE,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Registration triggers for specific conditions
CREATE TABLE registration_triggers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id UUID REFERENCES feature_templates(id) NOT NULL,
  conditions JSONB NOT NULL, -- email domains, sources, referrals, etc.
  assignment_type VARCHAR(50) NOT NULL DEFAULT 'permanent',
  trial_duration_days INTEGER,
  variables JSONB,
  active BOOLEAN DEFAULT TRUE,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Package tier mapping for templates
CREATE TABLE package_tier_templates (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  package_tier VARCHAR(50) NOT NULL,
  template_id UUID REFERENCES feature_templates(id) NOT NULL,
  template_variables JSONB,
  feature_overrides JSONB,
  usage_quotas JSONB,
  auto_assign BOOLEAN DEFAULT TRUE,
  created_at TIMESTAMP DEFAULT NOW(),
  
  CONSTRAINT unique_package_template UNIQUE (package_tier, template_id)
);

-- Enhanced user registration tracking
CREATE TABLE user_registration_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) NOT NULL,
  package_tier VARCHAR(50) NOT NULL,
  registration_source VARCHAR(100),
  referral_code VARCHAR(50),
  email_domain VARCHAR(100),
  geographic_region VARCHAR(50),
  templates_assigned JSONB,
  assignment_results JSONB,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_auto_assignment_template ON template_auto_assignment_rules(template_id);
CREATE INDEX idx_auto_assignment_trigger ON template_auto_assignment_rules(trigger_type);
CREATE INDEX idx_registration_triggers_template ON registration_triggers(template_id);
CREATE INDEX idx_package_tier_templates_tier ON package_tier_templates(package_tier);
CREATE INDEX idx_user_registration_user ON user_registration_events(user_id);
```

#### 1.2 Analytics Extensions
```sql
-- Template assignment analytics
CREATE TABLE template_assignment_analytics (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id UUID REFERENCES feature_templates(id) NOT NULL,
  assignment_type VARCHAR(50) NOT NULL, -- 'auto', 'admin', 'payment'
  package_tier VARCHAR(50),
  assignment_date DATE NOT NULL,
  assignments_count INTEGER DEFAULT 0,
  activations_count INTEGER DEFAULT 0,
  usage_count INTEGER DEFAULT 0,
  revenue_generated DECIMAL(10,2) DEFAULT 0,
  created_at TIMESTAMP DEFAULT NOW(),
  
  CONSTRAINT unique_template_assignment_date UNIQUE (template_id, assignment_type, package_tier, assignment_date)
);

CREATE INDEX idx_template_analytics_date ON template_assignment_analytics(assignment_date);
CREATE INDEX idx_template_analytics_type ON template_assignment_analytics(assignment_type);
```

### Phase 2: Backend Implementation (3 days)

#### 2.1 Auto-Assignment Engine
```rust
// src/dom/services/auto_assignment.rs
pub struct AutoAssignmentEngine {
    template_repo: Arc<dyn TemplateRepository>,
    assignment_repo: Arc<dyn AssignmentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl AutoAssignmentEngine {
    pub async fn process_registration(&self, user_id: &UserId, package_tier: &PackageTier, context: &RegistrationContext) -> Result<AssignmentResults> {
        // 1. Get auto-assignment rules for package tier
        let rules = self.get_auto_assignment_rules(package_tier).await?;
        
        // 2. Evaluate registration triggers
        let triggered_templates = self.evaluate_registration_triggers(context).await?;
        
        // 3. Merge and prioritize templates
        let templates_to_assign = self.merge_template_assignments(rules, triggered_templates)?;
        
        // 4. Apply templates with variables
        let mut results = Vec::new();
        for assignment in templates_to_assign {
            let result = self.assign_template_to_user(user_id, &assignment).await?;
            results.push(result);
        }
        
        // 5. Record registration event
        self.record_registration_event(user_id, package_tier, context, &results).await?;
        
        Ok(AssignmentResults { assignments: results })
    }
    
    async fn evaluate_registration_triggers(&self, context: &RegistrationContext) -> Result<Vec<TriggeredTemplate>> {
        let triggers = self.assignment_repo.get_registration_triggers().await?;
        let mut triggered = Vec::new();
        
        for trigger in triggers {
            if self.matches_trigger_conditions(&trigger, context)? {
                triggered.push(TriggeredTemplate {
                    template_id: trigger.template_id,
                    assignment_type: trigger.assignment_type,
                    trial_duration: trigger.trial_duration_days,
                    variables: trigger.variables,
                });
            }
        }
        
        Ok(triggered)
    }
}
```

#### 2.2 Registration Service Enhancement
```rust
// src/app/use_cases/auth.rs - Enhanced registration
impl AuthUseCase {
    pub async fn register_user_with_templates(&self, request: &RegisterRequest) -> Result<RegisterResponse> {
        // 1. Validate email and create user (existing logic)
        let user = self.create_user_account(request).await?;
        
        // 2. Assign role based on package tier (existing logic)
        let role = self.assign_package_tier_role(&user.id, &request.package_tier).await?;
        
        // 3. NEW: Auto-assign templates based on package tier
        let registration_context = RegistrationContext {
            email_domain: extract_domain(&request.email),
            registration_source: request.source.clone(),
            referral_code: request.referral_code.clone(),
            geographic_region: request.region.clone(),
        };
        
        let assignment_results = self.auto_assignment_engine
            .process_registration(&user.id, &request.package_tier, &registration_context)
            .await?;
        
        // 4. Send welcome email with feature summary
        self.send_welcome_email_with_features(&user, &assignment_results).await?;
        
        Ok(RegisterResponse {
            user,
            role,
            assigned_templates: assignment_results.assignments,
            features_unlocked: assignment_results.total_features_unlocked(),
        })
    }
}
```

#### 2.3 Admin Assignment Service Enhancement
```rust
// src/app/use_cases/admin_assignment.rs - Direct assignment capabilities
pub struct AdminAssignmentService {
    assignment_engine: Arc<AutoAssignmentEngine>,
    permission_service: Arc<PermissionService>,
    notification_service: Arc<NotificationService>,
}

impl AdminAssignmentService {
    pub async fn assign_template_directly(&self, admin_id: &UserId, request: &DirectAssignmentRequest) -> Result<DirectAssignmentResult> {
        // 1. Validate admin permissions
        self.validate_admin_assignment_permission(admin_id, &request.template_id).await?;
        
        // 2. Check assignment conflicts
        let conflicts = self.check_assignment_conflicts(&request.user_id, &request.template_id).await?;
        
        // 3. Create direct assignment
        let assignment = AdminTemplateAssignment {
            assignment_type: request.assignment_type.clone(),
            assigned_by: admin_id.clone(),
            assignment_reason: request.reason.clone(),
            expires_at: request.expires_at,
            variables: request.variables.clone(),
            override_pricing: true, // Direct assignments bypass payment
            internal_notes: request.internal_notes.clone(),
            notification_settings: request.notification_settings.clone(),
        };
        
        // 4. Apply assignment
        let result = self.assignment_engine
            .apply_admin_assignment(&request.user_id, &request.template_id, &assignment)
            .await?;
        
        // 5. Send notifications
        if assignment.notification_settings.notify_user {
            self.send_assignment_notification(&request.user_id, &result).await?;
        }
        
        // 6. Record admin action
        self.record_admin_assignment_action(admin_id, &request, &result).await?;
        
        Ok(result)
    }
    
    pub async fn bulk_assign_template(&self, admin_id: &UserId, request: &BulkAssignmentRequest) -> Result<BulkAssignmentResult> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        
        for user_id in &request.user_ids {
            let individual_request = DirectAssignmentRequest {
                user_id: user_id.clone(),
                template_id: request.template_id.clone(),
                assignment_type: request.assignment_type.clone(),
                reason: request.reason.clone(),
                expires_at: request.expires_at,
                variables: request.variables.clone(),
                internal_notes: request.internal_notes.clone(),
                notification_settings: request.notification_settings.clone(),
            };
            
            match self.assign_template_directly(admin_id, &individual_request).await {
                Ok(result) => successful.push(result.assignment_id),
                Err(e) => failed.push(FailedAssignment {
                    user_id: user_id.clone(),
                    error: e.to_string(),
                    reason: classify_assignment_error(&e),
                }),
            }
        }
        
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

### Phase 3: Frontend Implementation (2 days)

#### 3.1 Enhanced Registration Flow
```typescript
// apps/frontend/components/auth/EnhancedRegistrationForm.tsx
export function EnhancedRegistrationForm() {
  const [packageTier, setPackageTier] = useState<PackageTier>('Bronze');
  const [availableTemplates, setAvailableTemplates] = useState<TemplatePreview[]>([]);
  
  // Load template preview based on package tier
  useEffect(() => {
    async function loadTemplatePreview() {
      const templates = await getTemplatePreviewForPackageTier(packageTier);
      setAvailableTemplates(templates);
    }
    loadTemplatePreview();
  }, [packageTier]);
  
  const handleRegistration = async (formData: RegistrationData) => {
    const registrationRequest = {
      ...formData,
      package_tier: packageTier,
      source: 'web_registration',
      referral_code: searchParams.get('ref'),
      region: await getUserRegion(),
    };
    
    const result = await registerUserWithTemplates(registrationRequest);
    
    // Show success with features unlocked
    showRegistrationSuccess({
      user: result.user,
      featuresUnlocked: result.features_unlocked,
      templatesAssigned: result.assigned_templates,
    });
  };
  
  return (
    <div className="registration-form">
      <PackageTierSelector 
        selected={packageTier}
        onChange={setPackageTier}
        templatePreviews={availableTemplates}
      />
      
      <RegistrationFields onSubmit={handleRegistration} />
      
      <FeaturePreviewSection templates={availableTemplates} />
    </div>
  );
}
```

#### 3.2 Admin Assignment Dashboard
```typescript
// apps/admin-frontend/components/template/AdminAssignmentDashboard.tsx
export function AdminAssignmentDashboard() {
  const [assignmentMode, setAssignmentMode] = useState<'single' | 'bulk'>('single');
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  
  const handleDirectAssignment = async (assignmentData: DirectAssignmentData) => {
    const request: DirectAssignmentRequest = {
      user_id: assignmentData.userId,
      template_id: selectedTemplate,
      assignment_type: 'promotional',
      reason: assignmentData.reason,
      expires_at: assignmentData.expiresAt,
      variables: assignmentData.variables,
      internal_notes: assignmentData.notes,
      notification_settings: {
        notify_user: true,
        include_tutorial: true,
        email_template: 'admin_assignment_welcome',
      },
    };
    
    const result = await adminAssignTemplate(request);
    showAssignmentSuccess(result);
  };
  
  const handleBulkAssignment = async (bulkData: BulkAssignmentData) => {
    const request: BulkAssignmentRequest = {
      user_ids: selectedUsers,
      template_id: selectedTemplate,
      assignment_type: 'temporary',
      reason: bulkData.reason,
      expires_at: bulkData.expiresAt,
      variables: bulkData.variables,
      notification_settings: {
        notify_user: true,
        include_tutorial: true,
      },
    };
    
    const result = await bulkAssignTemplate(request);
    showBulkAssignmentResults(result);
  };
  
  return (
    <div className="admin-assignment-dashboard">
      <AssignmentModeSelector 
        mode={assignmentMode}
        onChange={setAssignmentMode}
      />
      
      <TemplateSelector 
        selected={selectedTemplate}
        onChange={setSelectedTemplate}
      />
      
      {assignmentMode === 'single' ? (
        <SingleAssignmentPanel onAssign={handleDirectAssignment} />
      ) : (
        <BulkAssignmentPanel 
          selectedUsers={selectedUsers}
          onUsersChange={setSelectedUsers}
          onAssign={handleBulkAssignment}
        />
      )}
      
      <AssignmentAnalytics />
      <RecentAssignments />
    </div>
  );
}
```

### Phase 4: API Endpoints Implementation (1 day)

#### 4.1 Auto-Assignment APIs
```rust
// Enhanced registration endpoint
#[axum::debug_handler]
pub async fn register_user_enhanced(
    State(service): State<AuthService>,
    Json(request): Json<EnhancedRegisterRequest>,
) -> Result<Json<EnhancedRegisterResponse>, ApiError> {
    let result = service
        .register_user_with_templates(&request)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    
    Ok(Json(result))
}

// Admin direct assignment endpoints
#[axum::debug_handler]
pub async fn admin_assign_template(
    State(service): State<AdminAssignmentService>,
    Extension(admin): Extension<User>,
    Json(request): Json<DirectAssignmentRequest>,
) -> Result<Json<DirectAssignmentResult>, ApiError> {
    let result = service
        .assign_template_directly(&admin.id, &request)
        .await
        .map_err(|e| ApiError::Forbidden(e.to_string()))?;
    
    Ok(Json(result))
}

#[axum::debug_handler]
pub async fn admin_bulk_assign_template(
    State(service): State<AdminAssignmentService>,
    Extension(admin): Extension<User>,
    Json(request): Json<BulkAssignmentRequest>,
) -> Result<Json<BulkAssignmentResult>, ApiError> {
    let result = service
        .bulk_assign_template(&admin.id, &request)
        .await
        .map_err(|e| ApiError::Forbidden(e.to_string()))?;
    
    Ok(Json(result))
}
```

### Phase 5: Testing & Validation (1 day)

#### 5.1 Auto-Assignment Testing
```rust
#[tokio::test]
async fn test_registration_auto_assignment() {
    let service = setup_test_service().await;
    
    // Setup: Create templates for Bronze tier
    let bronze_template = create_test_template("bronze_analytics", "Bronze").await;
    setup_auto_assignment_rule(&bronze_template.id, "Bronze").await;
    
    // Test: Register user with Bronze package
    let request = EnhancedRegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        package_tier: PackageTier::Bronze,
        source: Some("web_registration".to_string()),
        referral_code: None,
        region: Some("US".to_string()),
    };
    
    let result = service.register_user_with_templates(&request).await.unwrap();
    
    // Verify: User has template assigned
    assert_eq!(result.assigned_templates.len(), 1);
    assert_eq!(result.assigned_templates[0].template_id, bronze_template.id);
    assert!(result.features_unlocked > 0);
}

#[tokio::test]
async fn test_admin_direct_assignment() {
    let service = setup_test_admin_service().await;
    let admin = create_test_admin().await;
    let user = create_test_user().await;
    let template = create_test_template("premium_analytics", "Premium").await;
    
    let request = DirectAssignmentRequest {
        user_id: user.id,
        template_id: template.id,
        assignment_type: AssignmentType::Promotional,
        reason: "Beta tester access".to_string(),
        expires_at: Some(Utc::now() + Duration::days(30)),
        variables: HashMap::new(),
        internal_notes: Some("Selected for beta program".to_string()),
        notification_settings: NotificationSettings {
            notify_user: true,
            include_tutorial: true,
            email_template: Some("beta_welcome".to_string()),
        },
    };
    
    let result = service.assign_template_directly(&admin.id, &request).await.unwrap();
    
    // Verify assignment
    assert_eq!(result.template_id, template.id);
    assert_eq!(result.user_id, user.id);
    assert_eq!(result.status, "active");
    assert!(result.features_unlocked.len() > 0);
}
```

#### 5.2 Frontend Integration Testing
```typescript
// E2E tests for enhanced registration
describe('Enhanced Registration Flow', () => {
  test('should auto-assign templates based on package tier', async ({ page }) => {
    await page.goto('/register');
    
    // Select Bronze package
    await page.click('[data-testid="package-bronze"]');
    
    // Verify template preview shows
    await expect(page.locator('[data-testid="template-preview"]')).toBeVisible();
    
    // Complete registration
    await page.fill('[data-testid="email"]', 'test@example.com');
    await page.fill('[data-testid="password"]', 'password123');
    await page.click('[data-testid="register-button"]');
    
    // Verify success page shows assigned features
    await expect(page.locator('[data-testid="features-unlocked"]')).toBeVisible();
    await expect(page.locator('[data-testid="template-count"]')).toContainText('1');
  });
});

describe('Admin Assignment Dashboard', () => {
  test('should allow direct template assignment', async ({ page }) => {
    await loginAsAdmin(page);
    await page.goto('/admin/assignments');
    
    // Select template and user
    await page.selectOption('[data-testid="template-select"]', 'premium_analytics');
    await page.fill('[data-testid="user-search"]', 'test@example.com');
    await page.click('[data-testid="user-select-first"]');
    
    // Configure assignment
    await page.fill('[data-testid="assignment-reason"]', 'Premium trial access');
    await page.selectOption('[data-testid="assignment-type"]', 'trial');
    
    // Submit assignment
    await page.click('[data-testid="assign-button"]');
    
    // Verify success
    await expect(page.locator('[data-testid="assignment-success"]')).toBeVisible();
  });
});
```

## Migration Timeline

| Phase | Duration | Dependencies | Deliverables |
|-------|----------|--------------|--------------|
| **Phase 1: Database** | 2 days | None | Schema extensions, indexes |
| **Phase 2: Backend** | 3 days | Phase 1 | Auto-assignment engine, admin services |
| **Phase 3: Frontend** | 2 days | Phase 2 | Enhanced registration, admin dashboard |
| **Phase 4: APIs** | 1 day | Phase 2 | REST endpoints, documentation |
| **Phase 5: Testing** | 1 day | All phases | Test suite, validation |

**Total Duration: 9 days**

## Risk Mitigation

### Technical Risks
1. **Database Performance**: Extensive indexing and query optimization
2. **Assignment Conflicts**: Conflict detection and resolution logic
3. **Template Dependencies**: Dependency validation before assignment
4. **Auto-Assignment Logic**: Comprehensive testing of assignment rules

### Business Risks
1. **User Experience**: Gradual rollout with feature flags
2. **Admin Training**: Documentation and training materials
3. **Performance Impact**: Load testing with realistic user volumes
4. **Data Integrity**: Comprehensive audit logging and rollback capabilities

## Success Metrics

### Technical Metrics
- **Registration Performance**: <2s end-to-end registration with template assignment
- **Assignment Success Rate**: >99% successful auto-assignments
- **Admin Operations**: <1s single assignments, <5s bulk operations
- **System Reliability**: No performance degradation during peak usage

### Business Metrics
- **User Onboarding**: Immediate feature access upon registration
- **Admin Efficiency**: 90% reduction in manual template assignment time
- **Feature Adoption**: 50% increase in feature usage post-registration
- **User Satisfaction**: Improved onboarding experience scores

## Post-Migration Validation

### Functional Validation
1. **Registration Flow**: Verify all package tiers auto-assign correctly
2. **Admin Operations**: Test single and bulk assignment workflows
3. **Template Management**: Validate template creation and approval flows
4. **Analytics**: Confirm assignment analytics and reporting

### Performance Validation
1. **Load Testing**: 1000+ concurrent registrations with template assignment
2. **Database Performance**: Query performance under load
3. **Memory Usage**: Monitor memory consumption during bulk operations
4. **Response Times**: Verify all operations meet SLA requirements

## Documentation Updates

### Admin Documentation
- **Template Assignment Guide**: Step-by-step admin procedures
- **Bulk Operations Manual**: Best practices for bulk assignments
- **Analytics Dashboard Guide**: Using assignment analytics and reporting
- **Troubleshooting Guide**: Common issues and resolutions

### Developer Documentation
- **API Documentation**: Updated endpoints and examples
- **Integration Guide**: Frontend integration patterns
- **Database Schema**: New tables and relationships
- **Testing Guide**: Automated testing procedures

---

**Document Version**: 1.1  
**Created**: 2025-01-24  
**Last Updated**: 2025-01-24  
**Status**: Ready for Implementation - Detailed Plan Available  
**Cross-References**: iam-design.md v2.1 (template system design), architecture-design.md v2.1 (IAM architecture)  
**Dependencies**: Current IAM system, PostgreSQL database, existing template framework  
**Approval Required**: Technical lead, Product owner  
**Estimated Effort**: 9 development days  
**Risk Level**: Low (extends existing system without breaking changes)