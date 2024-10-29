CREATE TABLE users (
    name TEXT NOT NULL,
    promotion INTEGER NOT NULL DEFAULT 0,
    last_lap TEXT,
    last_login TEXT
);

CREATE TABLE emails (
    email TEXT NOT NULL UNIQUE,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    token BLOB UNIQUE,
    token_creation TEXT,
    token_last_usage TEXT,
    password BLOB UNIQUE,
    password_creation TEXT,
    password_last_usage TEXT,
    user INTEGER DEFAULT NULL
);

CREATE TABLE cookie_logins (
    user INTEGER NOT NULL,
    token BLOB NOT NULL UNIQUE,
    creation TEXT NOT NULL,
    last_user_agent BLOB,
    last_usage TEXT DEFAULT CURRENT_TIMESTAMP
);