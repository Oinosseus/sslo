CREATE TABLE users (
    rowid INTEGER PRIMARY KEY,
    name TEXT NOT NULL
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
