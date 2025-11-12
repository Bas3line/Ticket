DROP TABLE IF EXISTS blacklist;

CREATE TABLE blacklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_id BIGINT NOT NULL UNIQUE,
    target_type VARCHAR(10) NOT NULL CHECK (target_type IN ('user', 'guild')),
    reason TEXT,
    blacklisted_by BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_blacklist_target ON blacklist(target_id, target_type);
CREATE INDEX IF NOT EXISTS idx_blacklist_type ON blacklist(target_type);
