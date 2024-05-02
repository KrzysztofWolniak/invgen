-- Your SQL goes here
CREATE TABLE users (
                       id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                       name VARCHAR NOT NULL,
                       email VARCHAR NOT NULL UNIQUE,
                       schema INTEGER NOT NULL
);