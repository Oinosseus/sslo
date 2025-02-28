CREATE TABLE users (
    rowid INTEGER PRIMARY KEY,
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
    rowid INTEGER PRIMARY KEY,
    user INTEGER NOT NULL,
    token BLOB NOT NULL UNIQUE,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_useragent BLOB,
    last_usage TEXT,
    FOREIGN KEY(user) REFERENCES users(rowid)
);

CREATE TABLE steam_accounts (
    rowid INTEGER PRIMARY KEY,
    steam_id TEXT NOT NULL UNIQUE,
    user INTEGER,
    creation TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TEXT,
    FOREIGN KEY(user) REFERENCES users(rowid)
);

CREATE TABLE email_accounts (
    rowid INTEGER PRIMARY KEY,
    user INTEGER,
    email TEXT UNIQUE,
    token BLOB UNIQUE,
    token_creation TEXT,
    token_consumption TEXT,
    FOREIGN KEY(user) REFERENCES users(rowid)
)
