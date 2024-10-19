CREATE TABLE user (
    name TEXT NOT NULL,
    permission INTEGER NOT NULL DEFAULT 0,
    settings BLOB
);

CREATE TABLE email (
    email TEXT NOT NULL UNIQUE,
    token BLOB UNIQUE,
    token_creation_time TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user INTEGER DEFAULT NULL,
);
