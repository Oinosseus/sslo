CREATE TABLE users (
    name TEXT NOT NULL,
    permission INTEGER NOT NULL DEFAULT 0
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
