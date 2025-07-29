# Database Entity Relationship Diagram

## ER Diagram

```mermaid
erDiagram
    users {
        uuid id PK
        varchar firebase_uid UK
        varchar email
        varchar role
        timestamptz created_at
        timestamptz updated_at
    }

    sessions {
        uuid id PK
        uuid user_id FK
        text access_token
        text refresh_token
        timestamptz expires_at
        timestamptz created_at
        boolean is_active
    }

    permission_profiles {
        uuid id PK
        varchar name UK
        text description
        varchar category
        varchar version
        varchar status
        jsonb profile_data
        jsonb pricing_tier
        jsonb auto_assignment_rules
        jsonb api_endpoints
        jsonb frontend_routes
        varchar compliance_level
        timestamptz created_at
        timestamptz updated_at
        uuid created_by FK
    }

    admin_permission_profile_assignments {
        uuid id PK
        uuid user_id FK
        uuid permission_profile_id FK
        uuid assigned_by FK
        varchar assignment_type
        text assignment_reason
        timestamptz expires_at
        jsonb variables
        boolean override_pricing
        jsonb notification_settings
        varchar status
        timestamptz created_at
    }

    assignment_audit_log {
        uuid id PK
        uuid assignment_id FK
        varchar action
        uuid performed_by FK
        jsonb details
        timestamptz timestamp
    }

    audit_logs {
        uuid id PK
        uuid actor_id FK
        uuid user_id FK
        varchar action
        varchar resource_type
        varchar resource_id
        varchar result
        varchar event_category
        varchar severity
        boolean success
        jsonb details
        jsonb metadata
        inet client_ip
        inet ip_address
        text user_agent
        timestamptz timestamp
        uuid session_id FK
    }

    %% Relationships
    users ||--o{ sessions : "has"
    users ||--o{ permission_profiles : "creates"
    users ||--o{ admin_permission_profile_assignments : "receives"
    users ||--o{ admin_permission_profile_assignments : "assigns"
    users ||--o{ audit_logs : "performs_as_actor"
    users ||--o{ audit_logs : "target_of_action"
    users ||--o{ assignment_audit_log : "performs"

    permission_profiles ||--o{ admin_permission_profile_assignments : "assigned_via"
    
    admin_permission_profile_assignments ||--o{ assignment_audit_log : "tracked_in"
    
    sessions ||--o{ audit_logs : "generates"
```

## Entity Descriptions

### Core Entities

**users**
- Primary user table with Firebase authentication
- Contains user role (user, premium, moderator, admin, super_admin)
- Base entity for all user-related operations

**sessions** 
- User authentication sessions
- Links to Firebase tokens and expiration
- Tracks active user sessions

**permission_profiles**
- Predefined permission templates (Bronze, Silver, Admin Dashboard, etc.)
- Contains JSONB configuration for features, API endpoints, and pricing
- Created by admin users for assignment to others

**admin_permission_profile_assignments**
- Junction table linking users to permission profiles
- Contains assignment metadata (reason, expiration, variables)
- Tracks who assigned what to whom

### Audit & Compliance

**audit_logs**
- Comprehensive audit trail for all system actions
- Tracks actor, target, action type, and results
- Includes client information and session correlation

**assignment_audit_log**
- Dedicated audit trail for permission assignments
- Tracks changes to user permission assignments
- Links to main audit system

## Key Relationships

1. **User Management**
   - Users authenticate via sessions
   - Users can be assigned multiple permission profiles
   - All user actions generate audit logs

2. **Permission System**
   - Admins create permission profiles with specific capabilities
   - Profiles are assigned to users through assignment table
   - Assignments can have expiration and custom variables

3. **Audit Trail**
   - All actions tracked in audit_logs
   - Permission assignments have dedicated audit log
   - Session activities correlated with audit entries

## Data Flow

1. User authenticates → creates session
2. Admin creates permission profiles → stored with configuration
3. Admin assigns profiles to users → tracked in assignments table
4. All operations logged → comprehensive audit trail
5. Session activities → correlated audit entries

## Security Features

- Firebase UID integration for authentication
- Role-based access control
- Comprehensive audit logging
- Session tracking and management
- Assignment reason tracking for compliance
- IP and user agent logging for security