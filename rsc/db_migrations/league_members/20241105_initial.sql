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
    password_last_useragent BLOB
);

CREATE TABLE cookie_logins (
    user INTEGER NOT NULL,
    token BLOB NOT NULL UNIQUE,
    creation TEXT NOT NULL,
    last_useragent BLOB,
    last_usage TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE steam_users (
    steam_id TEXT NOT NULL UNIQUE,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user INTEGER,
    last_login_timestamp TEXT,
    last_login_useragent BLOB
)