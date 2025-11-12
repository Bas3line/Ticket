CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE guilds (
    guild_id BIGINT PRIMARY KEY,
    ticket_category_id BIGINT,
    log_channel_id BIGINT,
    transcript_channel_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ticket_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    emoji VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE support_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(guild_id, role_id)
);

CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    channel_id BIGINT NOT NULL UNIQUE,
    ticket_number INTEGER NOT NULL,
    owner_id BIGINT NOT NULL,
    category_id UUID REFERENCES ticket_categories(id) ON DELETE SET NULL,
    claimed_by BIGINT,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    UNIQUE(guild_id, ticket_number)
);

CREATE TABLE ticket_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    message_id BIGINT NOT NULL,
    author_id BIGINT NOT NULL,
    author_name VARCHAR(255) NOT NULL,
    author_discriminator VARCHAR(10),
    author_avatar_url TEXT,
    content TEXT NOT NULL,
    attachments JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ticket_panel (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    channel_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(guild_id, message_id)
);

CREATE INDEX idx_tickets_guild_id ON tickets(guild_id);
CREATE INDEX idx_tickets_owner_id ON tickets(owner_id);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_ticket_messages_ticket_id ON ticket_messages(ticket_id);
CREATE INDEX idx_ticket_categories_guild_id ON ticket_categories(guild_id);
CREATE INDEX idx_support_roles_guild_id ON support_roles(guild_id);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_guilds_updated_at BEFORE UPDATE ON guilds
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
