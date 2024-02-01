use rusqlite::{backup, Connection as RuConnection, Result as SQLiteResult};
use std::time::Duration;

use crate::config::Settings;

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
        CREATE TABLE files (
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

        CREATE INDEX duration ON files (duration);

        CREATE TABLE plays (
            id        INTEGER PRIMARY KEY,
            hash      TEXT,
            time      INTEGER,
            file      INTEGER
        );

        CREATE INDEX hash ON plays (hash);
        CREATE INDEX time ON plays (time);
        CREATE INDEX file ON plays (file);
        ";

        match conn.execute_batch(sql) {
            Ok(_) => println!("Successfully created files table."),
            Err(err) => println!("update failed: {}", err),
        }

        let conn = SQLite::connect();

        let sql = "
        CREATE VIRTUAL TABLE search
        USING FTS5(path, file_name, file_ext, title, artist, album);
        ";

        match conn.execute_batch(sql) {
            Ok(_) => println!("Successfully created search table."),
            Err(err) => println!("update failed: {}", err),
        }
    }

    pub fn restore() -> SQLiteResult<()> {
        let db_file = Settings::get("System", "db_file");
        let src = RuConnection::open(db_file)?;
        let mut dst = SQLite::connect();
        let backup = backup::Backup::new(&src, &mut dst)?;
        backup.run_to_completion(
            5,
            Duration::from_millis(0),
            Some(SQLite::db_backup_progress),
        )?;

        println!("Restored.");

        Ok(())
    }

    fn db_backup_progress(p: backup::Progress) {
        let pagecount = f64::from(p.pagecount);
        let remaining = f64::from(p.remaining);

        let remaining = ((pagecount - remaining) / pagecount) * 100.0;

        println!("Progress: {}%", remaining.round());
    }
}
