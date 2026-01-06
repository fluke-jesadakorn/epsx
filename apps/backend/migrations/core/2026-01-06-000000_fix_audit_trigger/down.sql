-- Revert to 'system' (though it was broken, this restores state)
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
            'system',
            jsonb_build_object(
                'old_record', to_jsonb(OLD),
                'new_record', to_jsonb(NEW)
            )
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
