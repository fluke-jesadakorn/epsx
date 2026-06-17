// @generated automatically by Diesel CLI — wave12(track-b) hand-edit.
// Originally generated as `pub mod infra_logs { diesel::table! { ... } }`
// (the schema-qualified wrap that diesel emits when you pass --schema).
// We unwrap the inner `pub mod` and re-prefix every `diesel::table!` with
// `infra_logs.` so the Rust module path stays `crate::schemas::infra_logs::X`
// (no double-qualifier). The `infra_logs.` on the table! macro is the SQL
// identifier — it tells diesel to query the table in the `infra_logs`
// schema. Doc comments keep the schema-qualified name for clarity.

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.aggregate_snapshots` table.
    infra_logs.aggregate_snapshots (aggregate_id) {
        /// The `aggregate_id` column of the `infra_logs.aggregate_snapshots` table.
        #[max_length = 255]
        aggregate_id -> Varchar,
        /// The `aggregate_type` column of the `infra_logs.aggregate_snapshots` table.
        #[max_length = 100]
        aggregate_type -> Varchar,
        /// The `aggregate_version` column of the `infra_logs.aggregate_snapshots` table.
        aggregate_version -> Int8,
        /// The `snapshot_data` column of the `infra_logs.aggregate_snapshots` table.
        snapshot_data -> Jsonb,
        /// The `created_at` column of the `infra_logs.aggregate_snapshots` table.
        created_at -> Timestamptz,
        /// The `event_count_at_snapshot` column of the `infra_logs.aggregate_snapshots` table.
        event_count_at_snapshot -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.analytics_events` table.
    infra_logs.analytics_events (id) {
        /// The `id` column of the `infra_logs.analytics_events` table.
        id -> Uuid,
        /// The `event_type` column of the `infra_logs.analytics_events` table.
        #[max_length = 50]
        event_type -> Varchar,
        /// The `wallet_address` column of the `infra_logs.analytics_events` table.
        #[max_length = 42]
        wallet_address -> Nullable<Varchar>,
        /// The `resource_path` column of the `infra_logs.analytics_events` table.
        #[max_length = 255]
        resource_path -> Varchar,
        /// The `method` column of the `infra_logs.analytics_events` table.
        #[max_length = 10]
        method -> Varchar,
        /// The `status_code` column of the `infra_logs.analytics_events` table.
        status_code -> Int4,
        /// The `duration_ms` column of the `infra_logs.analytics_events` table.
        duration_ms -> Int4,
        /// The `metadata` column of the `infra_logs.analytics_events` table.
        metadata -> Nullable<Jsonb>,
        /// The `created_at` column of the `infra_logs.analytics_events` table.
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs` table.
    infra_logs.api_key_usage_logs (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs_2025_11` partition.
    infra_logs.api_key_usage_logs_2025_11 (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs_2025_12` partition.
    infra_logs.api_key_usage_logs_2025_12 (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs_2026_01` partition.
    infra_logs.api_key_usage_logs_2026_01 (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs_2026_02` partition.
    infra_logs.api_key_usage_logs_2026_02 (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Representation of the `infra_logs.api_key_usage_logs_2026_03` partition.
    infra_logs.api_key_usage_logs_2026_03 (id, request_at) {
        id -> Uuid,
        api_key_id -> Uuid,
        module_id -> Nullable<Uuid>,
        endpoint -> Varchar,
        method -> Varchar,
        response_status -> Nullable<Int4>,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        request_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Audit trail for subscription/assignment modifications
    infra_logs.assignment_audit_log (id) {
        id -> Uuid,
        assignment_id -> Uuid,
        action -> Varchar,
        old_value -> Nullable<Text>,
        new_value -> Nullable<Text>,
        reason -> Nullable<Text>,
        performed_by -> Varchar,
        performed_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// General audit log for all actions
    infra_logs.audit_logs (id) {
        id -> Uuid,
        wallet_address -> Nullable<Varchar>,
        action -> Varchar,
        resource_type -> Varchar,
        resource_id -> Nullable<Varchar>,
        result -> Varchar,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        details -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Immutable event log for event sourcing
    infra_logs.event_store (event_id) {
        event_id -> Uuid,
        aggregate_id -> Varchar,
        aggregate_type -> Varchar,
        aggregate_version -> Int8,
        event_type -> Varchar,
        event_data -> Jsonb,
        metadata -> Jsonb,
        occurred_at -> Timestamptz,
        causation_id -> Nullable<Uuid>,
        correlation_id -> Nullable<Uuid>,
        user_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Transactional outbox for reliable event publishing
    infra_logs.outbox_events (id) {
        id -> Int8,
        event_id -> Uuid,
        aggregate_id -> Varchar,
        aggregate_type -> Varchar,
        event_type -> Varchar,
        event_payload -> Jsonb,
        processed -> Bool,
        processed_at -> Nullable<Timestamptz>,
        retry_count -> Int4,
        last_error -> Nullable<Text>,
        next_retry_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        sequence_number -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Audit trail for payment status changes
    infra_logs.payment_audit_log (id) {
        id -> Uuid,
        payment_id -> Uuid,
        action -> Varchar,
        old_status -> Nullable<Varchar>,
        new_status -> Varchar,
        reason -> Nullable<Text>,
        performed_by -> Nullable<Varchar>,
        performed_at -> Timestamptz,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Audit trail for permission changes
    infra_logs.permission_audit_log (id) {
        id -> Uuid,
        event_type -> Varchar,
        event_timestamp -> Timestamptz,
        event_source -> Varchar,
        wallet_address -> Varchar,
        permission_string -> Nullable<Varchar>,
        permission_id -> Nullable<Uuid>,
        group_id -> Nullable<Uuid>,
        group_name -> Nullable<Varchar>,
        performed_by -> Nullable<Varchar>,
        performed_by_name -> Nullable<Varchar>,
        reason -> Nullable<Text>,
        request_id -> Nullable<Varchar>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        previous_state -> Nullable<Jsonb>,
        new_state -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamptz>,
        valid_from -> Nullable<Timestamptz>,
        valid_until -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Unified audit log for all system actions
    infra_logs.unified_audit_log (id) {
        id -> Uuid,
        #[max_length = 42]
        actor -> Nullable<Varchar>,
        #[max_length = 20]
        actor_type -> Varchar,
        created_at -> Timestamptz,
        #[max_length = 50]
        resource_type -> Varchar,
        #[max_length = 255]
        resource_id -> Nullable<Varchar>,
        #[max_length = 50]
        action -> Varchar,
        #[max_length = 20]
        effect -> Varchar,
        before_state -> Nullable<Jsonb>,
        after_state -> Nullable<Jsonb>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        #[max_length = 30]
        category -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    /// Wallet activity tracking
    infra_logs.wallet_activity_logs (id) {
        id -> Uuid,
        wallet_address -> Varchar,
        event_type -> Varchar,
        description -> Text,
        performed_by -> Nullable<Varchar>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aggregate_snapshots,
    analytics_events,
    api_key_usage_logs,
    api_key_usage_logs_2025_11,
    api_key_usage_logs_2025_12,
    api_key_usage_logs_2026_01,
    api_key_usage_logs_2026_02,
    api_key_usage_logs_2026_03,
    assignment_audit_log,
    audit_logs,
    event_store,
    outbox_events,
    payment_audit_log,
    permission_audit_log,
    unified_audit_log,
    wallet_activity_logs,
);
