use rusqlite::{backup, Connection as RuConnection, Result as SQLiteResult};
use std::time::Duration;

pub struct SQLite;

impl SQLite {
    pub fn initialize() -> RuConnection {
        let persist = SQLite::connect();
        SQLite::migrate();
        persist
    }

    pub fn connect() -> RuConnection {
        match RuConnection::open("file:auralist.sqlite?cache=shared") {
            Ok(conn) => conn,
            Err(error) => panic!("Cannot connect to SQLite: {}", error),
        }
    }

    pub fn migrate() {
        println!("Initializing DB...");

        let conn = SQLite::connect();

        let sql = "
        CREATE TABLE IF NOT EXISTS files (
            id            INTEGER PRIMARY KEY,
            path          TEXT NOT NULL,
            file_name     TEXT NOT NULL,
            file_ext      TEXT NOT NULL,
            file_size     INTEGER,
            file_modified INTEGER,
            title         TEXT NOT NULL,
            artist        TEXT NOT NULL,
            album         TEXT NOT NULL,
            duration      INTEGER,
            indexed_at    INTEGER
        );

        CREATE INDEX IF NOT EXISTS duration ON files (duration);

        CREATE TABLE IF NOT EXISTS plays (
            id        INTEGER PRIMARY KEY,
            hash      TEXT,
            time      INTEGER,
            file      INTEGER
        );

        CREATE INDEX IF NOT EXISTS hash ON plays (hash);
        CREATE INDEX IF NOT EXISTS time ON plays (time);
        CREATE INDEX IF NOT EXISTS file ON plays (file);
        ";

        match conn.execute_batch(sql) {
            Ok(_) => println!("Successfully created files table."),
            Err(err) => println!("update failed: migration 1: {}", err),
        }

        let conn = SQLite::connect();

        let sql = "
        CREATE VIRTUAL TABLE IF NOT EXISTS search
        USING FTS5(path, file_name, file_ext, title, artist, album);
        ";

        match conn.execute_batch(sql) {
            Ok(_) => println!("Successfully created search table."),
            Err(err) => println!("update failed: migration 2: {}", err),
        }
    }
}
