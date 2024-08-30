-- Add migration script here
ALTER TABLE email_verify ADD COLUMN enable_check_mail BOOLEAN NOT NULL DEFAULT 0;