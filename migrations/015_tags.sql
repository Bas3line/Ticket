CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    name VARCHAR(100) NOT NULL,
    content TEXT NOT NULL,
    creator_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uses INTEGER NOT NULL DEFAULT 0,
    UNIQUE(guild_id, name)
);

CREATE INDEX idx_tags_guild_id ON tags(guild_id);
CREATE INDEX idx_tags_name ON tags(guild_id, name);
CREATE INDEX idx_tags_creator ON tags(creator_id);
