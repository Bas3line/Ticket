ALTER TABLE tickets ADD COLUMN IF NOT EXISTS assigned_to BIGINT DEFAULT NULL;

ALTER TABLE guilds ADD COLUMN IF NOT EXISTS autoclose_enabled BOOLEAN DEFAULT FALSE;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS autoclose_minutes INT DEFAULT NULL;

ALTER TABLE tickets ADD COLUMN IF NOT EXISTS last_message_at TIMESTAMPTZ DEFAULT NULL;

CREATE INDEX IF NOT EXISTS idx_tickets_assigned_to ON tickets(assigned_to);
CREATE INDEX IF NOT EXISTS idx_tickets_last_message_at ON tickets(last_message_at);
