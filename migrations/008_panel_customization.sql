-- Add button style to panel_categories
ALTER TABLE panel_categories ADD COLUMN IF NOT EXISTS button_style VARCHAR(20) DEFAULT 'primary';

-- Add embed customization to ticket_panel
ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS embed_color INTEGER DEFAULT 5865202;
ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS embed_image_url TEXT;
ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS embed_thumbnail_url TEXT;
ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS embed_footer_text TEXT;
ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS embed_footer_icon_url TEXT;

-- Add premium features to guilds table
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS max_tickets_per_user INTEGER DEFAULT 1;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS max_open_tickets INTEGER;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS ticket_name_format VARCHAR(100) DEFAULT 'ticket-{number}';
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS welcome_message TEXT;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS close_confirmation BOOLEAN DEFAULT TRUE;
ALTER TABLE guilds ADD COLUMN IF NOT EXISTS ticket_logs_enabled BOOLEAN DEFAULT TRUE;
