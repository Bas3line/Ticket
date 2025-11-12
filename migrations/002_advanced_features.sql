ALTER TABLE tickets ADD COLUMN priority VARCHAR(20) DEFAULT 'normal';
ALTER TABLE tickets ADD COLUMN rating INTEGER CHECK (rating >= 1 AND rating <= 5);
ALTER TABLE tickets ADD COLUMN last_activity TIMESTAMPTZ DEFAULT NOW();

CREATE INDEX idx_tickets_priority ON tickets(priority);
CREATE INDEX idx_tickets_last_activity ON tickets(last_activity);

CREATE TABLE ticket_notes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    author_id BIGINT NOT NULL,
    note TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ticket_notes_ticket_id ON ticket_notes(ticket_id);

CREATE TABLE blacklist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    reason TEXT,
    blacklisted_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(guild_id, user_id)
);

CREATE INDEX idx_blacklist_guild_user ON blacklist(guild_id, user_id);
