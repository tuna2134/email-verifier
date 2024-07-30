-- Add migration script here
CREATE TABLE IF NOT EXISTS email_verify (
    guild_id BIGINT NOT NULL PRIMARY KEY,
    email_pattern TEXT NOT NULL,
    role_id BIGINT NOT NULL
);