-- Add backup category support to ticket_categories table
-- Each category can have backup categories when the primary reaches 50 channels (Discord limit)
-- Premium guilds can have up to 4 backup categories per ticket category

CREATE TABLE IF NOT EXISTS category_backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES ticket_categories(id) ON DELETE CASCADE,
    discord_category_id BIGINT NOT NULL,
    backup_number INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(category_id, backup_number),
    CHECK (backup_number >= 1 AND backup_number <= 4)
);

CREATE INDEX IF NOT EXISTS idx_category_backups_category_id ON category_backups(category_id);
CREATE INDEX IF NOT EXISTS idx_category_backups_discord_category_id ON category_backups(discord_category_id);

-- Add primary discord category ID to ticket_categories
ALTER TABLE ticket_categories ADD COLUMN IF NOT EXISTS discord_category_id BIGINT DEFAULT NULL;
