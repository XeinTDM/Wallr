CREATE TABLE IF NOT EXISTS user_verifications (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token VARCHAR,
    token_expires_at TIMESTAMPTZ,
    last_resent_at TIMESTAMPTZ
);
