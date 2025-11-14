CREATE TABLE IF NOT EXISTS ignored_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(guild_id, channel_id)
);

CREATE INDEX IF NOT EXISTS idx_ignored_channels_guild ON ignored_channels(guild_id);
CREATE INDEX IF NOT EXISTS idx_ignored_channels_lookup ON ignored_channels(guild_id, channel_id);
