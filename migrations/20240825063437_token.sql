-- Add migration script here
CREATE TABLE token (
    user_id BIGINT NOT NULL PRIMARY KEY,
    nonce TEXT NOT NULL
)