// @generated automatically by Diesel CLI.

diesel::table! {
    admin_module_permissions (id) {
        id -> Uuid,
        module_code -> Varchar,
        api_endpoints -> Array<Text>,
        frontend_routes -> Array<Text>,
        permissions -> Array<Text>,
        resource_patterns -> Array<Text>,
        access_level -> Nullable<Varchar>,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    admin_modules (id) {
        id -> Uuid,
        module_code -> Varchar,
        module_name -> Varchar,
        description -> Text,
        category -> Varchar,
        icon -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        sort_order -> Nullable<Int4>,
        is_active -> Nullable<Bool>,
        requires_modules -> Array<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    admin_role_audit (id) {
        id -> Uuid,
        firebase_uid -> Varchar,
        module_code -> Varchar,
        action -> Varchar,
        old_access_level -> Nullable<Varchar>,
        new_access_level -> Nullable<Varchar>,
        performed_by -> Varchar,
        reason -> Nullable<Text>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        action -> Varchar,
        resource_type -> Nullable<Varchar>,
        resource_id -> Nullable<Varchar>,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    eps_growth_rankings (id) {
        id -> Uuid,
        symbol -> Varchar,
        company_name -> Varchar,
        current_eps -> Numeric,
        previous_eps -> Numeric,
        eps_growth_rate -> Numeric,
        market_cap -> Nullable<Numeric>,
        sector -> Nullable<Varchar>,
        industry -> Nullable<Varchar>,
        rank_position -> Int4,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    iam_groups (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        policies -> Array<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    iam_policies (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        version -> Varchar,
        document -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    iam_roles (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        policies -> Array<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    level_history (id) {
        id -> Uuid,
        user_id -> Uuid,
        old_level -> Varchar,
        new_level -> Varchar,
        change_reason -> Text,
        changed_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payments (id) {
        id -> Uuid,
        user_id -> Uuid,
        amount -> Numeric,
        currency -> Varchar,
        status -> Varchar,
        payment_method -> Nullable<Varchar>,
        transaction_id -> Nullable<Varchar>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    permission_profiles (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Text,
        category -> Varchar,
        permissions -> Array<Text>,
        prerequisites -> Array<Uuid>,
        auto_assign -> Bool,
        auto_assign_conditions -> Nullable<Jsonb>,
        expires_after_days -> Nullable<Int4>,
        max_assignments -> Nullable<Int4>,
        is_active -> Bool,
        created_by -> Uuid,
        updated_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        access_token -> Text,
        expires_at -> Timestamptz,
        provider -> Nullable<Varchar>,
        provider_account_id -> Nullable<Text>,
        session_token -> Nullable<Text>,
        jwt_token -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Inet>,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    stocks (symbol) {
        symbol -> Varchar,
        name -> Varchar,
        market -> Varchar,
        price -> Numeric,
        volume -> Int8,
        market_cap -> Nullable<Numeric>,
        sector -> Nullable<Varchar>,
        industry -> Nullable<Varchar>,
        last_updated -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    sub_modules (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Text,
        is_active -> Bool,
        quota_limits -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    temporary_permissions (id) {
        id -> Uuid,
        user_id -> Uuid,
        permission -> Varchar,
        resource -> Nullable<Varchar>,
        action -> Varchar,
        status -> Varchar,
        expires_at -> Timestamptz,
        granted_by -> Uuid,
        reason -> Text,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_admin_roles (id) {
        id -> Uuid,
        firebase_uid -> Varchar,
        module_code -> Varchar,
        granted_by -> Nullable<Varchar>,
        granted_reason -> Nullable<Text>,
        expires_at -> Nullable<Timestamptz>,
        is_active -> Nullable<Bool>,
        assignment_metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_module_assignments (id) {
        id -> Uuid,
        user_id -> Uuid,
        module_id -> Uuid,
        access_level -> Varchar,
        expires_at -> Nullable<Timestamptz>,
        assigned_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_permission_overrides (user_id) {
        user_id -> Uuid,
        allowed_permissions -> Array<Text>,
        denied_permissions -> Array<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        firebase_uid -> Varchar,
        email -> Varchar,
        display_name -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        avatar_url -> Nullable<Text>,
        package_tier -> Nullable<Varchar>,
        permissions -> Array<Text>,
        is_active -> Nullable<Bool>,
        last_login_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

// Note: These joinable macros should match foreign key relationships in the database
// Commenting out problematic joins that need schema review
// diesel::joinable!(admin_module_permissions -> admin_modules (module_code));
// diesel::joinable!(admin_role_audit -> admin_modules (module_code));
diesel::joinable!(level_history -> users (user_id));
diesel::joinable!(payments -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(temporary_permissions -> users (user_id));
// diesel::joinable!(user_admin_roles -> admin_modules (module_code));
diesel::joinable!(user_module_assignments -> sub_modules (module_id));
diesel::joinable!(user_module_assignments -> users (user_id));
diesel::joinable!(user_permission_overrides -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_module_permissions,
    admin_modules,
    admin_role_audit,
    audit_logs,
    eps_growth_rankings,
    iam_groups,
    iam_policies,
    iam_roles,
    level_history,
    payments,
    permission_profiles,
    sessions,
    stocks,
    sub_modules,
    temporary_permissions,
    user_admin_roles,
    user_module_assignments,
    user_permission_overrides,
    users,
);