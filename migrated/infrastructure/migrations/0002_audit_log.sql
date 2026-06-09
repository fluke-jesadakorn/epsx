-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(66),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50),
    resource_id VARCHAR(66),
    metadata JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log (action, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_log (resource_type, resource_id);

-- Edit sessions for collaborative editing
CREATE TABLE IF NOT EXISTS edit_sessions (
    id VARCHAR(66) PRIMARY KEY,
    page_id VARCHAR(66) NOT NULL,
    user_id VARCHAR(66) NOT NULL,
    role VARCHAR(50) NOT NULL,
    active BOOLEAN DEFAULT true,
    locked_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sessions_page ON edit_sessions (page_id, active);

-- Page versions (for history)
CREATE TABLE IF NOT EXISTS page_versions (
    id BIGSERIAL PRIMARY KEY,
    page_id VARCHAR(66) NOT NULL,
    version INTEGER NOT NULL,
    title TEXT,
    blocks_json TEXT,
    seo_json TEXT,
    author_id VARCHAR(66),
    commit_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(page_id, version)
);
CREATE INDEX IF NOT EXISTS idx_versions_page ON page_versions (page_id, version DESC);
