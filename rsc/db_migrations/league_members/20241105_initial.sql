CREATE TABLE users (
    name TEXT NOT NULL,
    promotion_level INTEGER NOT NULL DEFAULT 0,
    promotion_authority INTEGER NOT NULL DEFAULT 0,
    last_lap TEXT,
    last_login TEXT,
    password BLOB,
    password_last_usage TEXT,
    password_last_useragent BLOB
);

CREATE TABLE cookie_logins (
    user INTEGER NOT NULL,
    token BLOB NOT NULL UNIQUE,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_useragent BLOB,
    last_usage TEXT
);

CREATE TABLE steam_users (
    steam_id TEXT NOT NULL UNIQUE,
    user INTEGER,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_timestamp TEXT,
    last_login_useragent BLOB
);

CREATE TABLE email_accounts (
    user INTEGER,
    email TEXT UNIQUE,
    token BLOB UNIQUE,
    token_creation TEXT,
    token_consumption TEXT
)
