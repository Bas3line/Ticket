CREATE INDEX IF NOT EXISTS idx_tickets_channel_id ON tickets(channel_id);
CREATE INDEX IF NOT EXISTS idx_tickets_owner_id_status ON tickets(owner_id, status) WHERE status = 'open';
CREATE INDEX IF NOT EXISTS idx_tickets_created_at ON tickets(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ticket_notes_ticket_id_created ON ticket_notes(ticket_id, created_at);
CREATE INDEX IF NOT EXISTS idx_blacklist_target_type ON blacklist(target_id, target_type);
CREATE INDEX IF NOT EXISTS idx_guilds_guild_id ON guilds(guild_id) WHERE ticket_category_id IS NOT NULL;

ANALYZE tickets;
ANALYZE ticket_notes;
ANALYZE ticket_messages;
ANALYZE support_roles;
ANALYZE guilds;
