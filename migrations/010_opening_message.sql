ALTER TABLE tickets ADD COLUMN IF NOT EXISTS opening_message_id BIGINT;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS default_button_color VARCHAR(20) DEFAULT 'primary';
