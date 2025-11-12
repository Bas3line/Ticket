-- Add compound index for fast user ticket lookups per guild
-- This speeds up queries like: SELECT * FROM tickets WHERE guild_id = $1 AND owner_id = $2 AND status = 'open'
CREATE INDEX IF NOT EXISTS idx_tickets_guild_owner_status ON tickets(guild_id, owner_id, status);

-- Add index on ticket_panel guild_id for faster panel lookups
CREATE INDEX IF NOT EXISTS idx_ticket_panel_guild_id ON ticket_panel(guild_id);
