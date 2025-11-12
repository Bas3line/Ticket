CREATE TABLE IF NOT EXISTS premium (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL UNIQUE,
    max_servers INT NOT NULL DEFAULT 1,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT NOT NULL
);

CREATE INDEX idx_premium_guild ON premium(guild_id);
CREATE INDEX idx_premium_expires ON premium(expires_at);

ALTER TABLE guilds ADD COLUMN IF NOT EXISTS claim_buttons_enabled BOOLEAN DEFAULT TRUE;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS auto_close_hours INT DEFAULT NULL;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS ticket_limit_per_user INT DEFAULT 1;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS ticket_cooldown_seconds INT DEFAULT 0;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS dm_on_create BOOLEAN DEFAULT TRUE;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS embed_color INT DEFAULT 5865714;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS embed_title VARCHAR(256) DEFAULT 'Support Ticket';
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS embed_description TEXT DEFAULT 'Click the button below to create a ticket';
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS embed_footer TEXT DEFAULT NULL;

CREATE INDEX idx_guilds_settings ON guilds(guild_id, claim_buttons_enabled);
