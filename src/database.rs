use rusqlite::{backup, Connection as RuConnection, Result as SQLiteResult};
use std::time::Duration;

use crate::config::Settings;
use crate::dbbackup::BackupFile;

pub struct SQLite;

impl SQLite {
    pub fn initialize() -> RuConnection {
        let persist = SQLite::connect();
        SQLite::migrate();
        persist
    }

    pub fn connect() -> RuConnection {
        match RuConnection::open("file:blah?mode=memory&cache=shared") {
            Ok(conn) => conn,
            Err(error) => panic!("Cannot connect to SQLite: {}", error)
        }
    }

    pub fn migrate() {
        println!("Initializing DB...");
    
        let conn = SQLite::connect();
        
        let sql = "
        CREATE TABLE files (
            id        INTEGER PRIMARY KEY,
            path      TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_ext TEXT NOT NULL,
            title     TEXT NOT NULL,
            artist    TEXT NOT NULL,
            album     TEXT NOT NULL,
            time      TEXT NOT NULL
        );

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

    pub fn backup_to_gz() -> SQLiteResult<()> {
        println!("Backing up db to file...");
        let db_file = Settings::get("System", "db_file");
        let file_to_compress = db_file.clone();
        let src = SQLite::connect();
        let mut dst = RuConnection::open(db_file)?;
        let backup = backup::Backup::new(&src, &mut dst)?;
        backup.run_to_completion(5, Duration::from_millis(0), Some(SQLite::db_backup_progress))?;

        let f = BackupFile::new(&file_to_compress);
        f.compress_to_gz();

        println!("Backup finished.");

        Ok(())
    }

    pub fn restore_from_gz() -> SQLiteResult<()> {
        println!("Restoring db from file...");
        let db_file = Settings::get("System", "db_file");

        let file_to_decompress = db_file.clone();
        let f = BackupFile::new(&file_to_decompress);
        f.decompress_from_gz();

        let src = RuConnection::open(db_file)?;
        let mut dst = SQLite::connect();
        let backup = backup::Backup::new(&src, &mut dst)?;
        backup.run_to_completion(5, Duration::from_millis(0), Some(SQLite::db_backup_progress))?;

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