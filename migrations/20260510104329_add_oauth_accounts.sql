CREATE TABLE IF NOT EXISTS user_oauth_accounts (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    PRIMARY KEY (provider, provider_user_id)
);
