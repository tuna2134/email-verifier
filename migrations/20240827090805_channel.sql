-- Add migration script here
ALTER TABLE email_verify ADD COLUMN channel_id BIGINT NOT NULL DEFAULT 0;