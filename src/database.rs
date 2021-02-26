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
        CREATE TABLE file (
            id        INTEGER PRIMARY KEY,
            path_hash TEXT NOT NULL,
            path      TEXT NOT NULL,
            file_name TEXT NOT NULL,
            title     TEXT NOT NULL,
            artist    TEXT NOT NULL,
            album     TEXT NOT NULL
        );

        CREATE UNIQUE INDEX path_hash ON file (path_hash);

        CREATE VIRTUAL TABLE file_search
        USING FTS5(path_hash, file_name, title, artist, album);
        ";
    
        match conn.execute_batch(sql) {
            Ok(_) => println!("Success."),
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

        println!("Success.");

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

        println!("Success.");

        Ok(())
    }

    fn db_backup_progress(p: backup::Progress) {
        let pagecount = f64::from(p.pagecount);
        let remaining = f64::from(p.remaining);
    
        let remaining = ((pagecount - remaining) / pagecount) * 100.0;
    
        println!("Progress: {}%", remaining.round());
    }
}