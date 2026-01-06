-- Force replace the trigger function to ensure it uses NULL for system actions
-- This resolves the "valid_performed_by" check constraint violation
CREATE OR REPLACE FUNCTION log_payment_status_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO payment_audit_log (
            payment_id,
            action,
            old_status,
            new_status,
            reason,
            performed_by,
            metadata
        ) VALUES (
            NEW.id,
            'status_change',
            OLD.status,
            NEW.status,
            'Automatic status change',
            NULL, -- MUST be NULL to pass check constraint
            jsonb_build_object(
                'old_record', to_jsonb(OLD),
                'new_record', to_jsonb(NEW)
            )
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
