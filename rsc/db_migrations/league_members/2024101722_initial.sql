CREATE TABLE NewEmailUser (
                              Id INTEGER PRIMARY KEY ,
                              Email TEXT NOT NULL,
                              Token BLOB NOT NULL,
                              CreationTimeStamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE User (
    Id INTEGER PRIMARY KEY,
    Name TEXT NOT NULL,
    PermissionLevel INTEGER NOT NULL DEFAULT 0,
    SETTINGS BLOB
);

CREATE TABLE Email (
    Id INTEGER PRIMARY KEY ,
    User INTEGER NOT NULL,
    Email TEXT NOT NULL,
    IsVerified INTEGER NOT NULL,
    Password BLOB
);

CREATE TABLE Steam (
    Id INTEGER PRIMARY KEY,
    User INTEGER NOT NULL,
    Steam64GUID NOT NULL,
    LastDriverName TEXT,
    CreationTimeStamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
