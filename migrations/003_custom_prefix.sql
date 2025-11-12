ALTER TABLE guilds ADD COLUMN IF NOT EXISTS prefix VARCHAR(10) DEFAULT '!';

CREATE INDEX IF NOT EXISTS idx_guilds_prefix ON guilds(guild_id, prefix);
