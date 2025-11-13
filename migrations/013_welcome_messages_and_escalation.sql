ALTER TABLE ticket_categories ADD COLUMN IF NOT EXISTS custom_welcome_message TEXT;
ALTER TABLE ticket_categories ADD COLUMN IF NOT EXISTS use_custom_welcome BOOLEAN DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS ticket_escalations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    escalated_by BIGINT NOT NULL,
    escalated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_ping_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(ticket_id)
);

CREATE INDEX IF NOT EXISTS idx_ticket_escalations_ticket_id ON ticket_escalations(ticket_id);
CREATE INDEX IF NOT EXISTS idx_ticket_escalations_active ON ticket_escalations(is_active) WHERE is_active = TRUE;

ALTER TABLE tickets ADD COLUMN IF NOT EXISTS has_messages BOOLEAN DEFAULT FALSE;
