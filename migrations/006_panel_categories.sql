ALTER TABLE ticket_panel ADD COLUMN IF NOT EXISTS selection_type VARCHAR(20) DEFAULT 'button';

CREATE TABLE IF NOT EXISTS panel_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    panel_id UUID NOT NULL REFERENCES ticket_panel(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES ticket_categories(id) ON DELETE CASCADE,
    button_label VARCHAR(80) NOT NULL,
    button_emoji VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(panel_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_panel_categories_panel_id ON panel_categories(panel_id);
CREATE INDEX IF NOT EXISTS idx_panel_categories_category_id ON panel_categories(category_id);
