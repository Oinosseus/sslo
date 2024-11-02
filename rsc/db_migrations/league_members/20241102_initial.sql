CREATE TABLE users (
    name TEXT NOT NULL,
    promotion INTEGER NOT NULL DEFAULT 0,
    promotion_authority INTEGER NOT NULL DEFAULT 0,
    last_lap TEXT,
    email TEXT UNIQUE,
    email_token BLOB UNIQUE,
    email_token_creation TEXT,
    email_token_consumption TEXT,
    password BLOB,
    password_last_usage TEXT,
    password_last_user_agent BLOB
);

CREATE TABLE cookie_logins (
    user INTEGER NOT NULL,
    token BLOB NOT NULL UNIQUE,
    creation TEXT NOT NULL,
    last_user_agent BLOB,
    last_usage TEXT DEFAULT CURRENT_TIMESTAMP
);
