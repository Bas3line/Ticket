ALTER TABLE guilds ADD COLUMN IF NOT EXISTS channel_name_template VARCHAR(100) DEFAULT 'ticket-$ticket_number';
