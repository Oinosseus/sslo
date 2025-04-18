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

    -- since when the last token verification process successfully finished
    verified_since TEXT,

    -- the token that shall be sent via email and verified back
    token BLOB UNIQUE,

    -- This defines the user, that shall be assigned after successful token verification
    token_user INTEGER,

    -- when the token was created
    token_creation TEXT,

    -- when the token was verified
    token_consumption TEXT,

    FOREIGN KEY(user) REFERENCES users(rowid),
    FOREIGN KEY(token_user) REFERENCES users(rowid)
)
