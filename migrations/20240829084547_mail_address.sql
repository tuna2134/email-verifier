-- Add migration script here
CREATE TABLE mail_address (
    guild_id BIGINT NOT NULL REFERENCES email_verify(guild_id) ON DELETE CASCADE,
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL
)