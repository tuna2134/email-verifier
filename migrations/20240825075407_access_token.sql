-- Add migration script here
ALTER TABLE token ADD COLUMN access_token TEXT NOT NULL DEFAULT '';