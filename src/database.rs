use rusqlite::{backup, Connection as RuConnection, Result as SQLiteResult};
use std::path::Path;
use std::time::Duration;

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
        
        let sql = "CREATE TABLE file (
            id      INTEGER PRIMARY KEY,
            path    TEXT NOT NULL,
            path_hash TEXT NOT NULL
        );
        
        CREATE UNIQUE INDEX path_hash ON file (path_hash);
        ";
    
        match conn.execute_batch(sql) {
            Ok(_) => println!("Success."),
            Err(err) => println!("update failed: {}", err),
        }
    }

    pub fn backup_db_to_file<P: AsRef<Path>>(
        dst: P,
        progress: fn(backup::Progress),
    ) -> SQLiteResult<()> {
        println!("Backing up db to file...");
        let src = SQLite::connect();
        let mut dst = RuConnection::open(dst)?;
        let backup = backup::Backup::new(&src, &mut dst)?;
        backup.run_to_completion(5, Duration::from_millis(0), Some(progress))
    }

    pub fn restore_db_from_file<P: AsRef<Path>>(
        src: P,
        progress: fn(backup::Progress),
    ) -> SQLiteResult<()> {
        println!("Restoring db from file...");
        let src = RuConnection::open(src)?;
        let mut dst = SQLite::connect();
        let backup = backup::Backup::new(&src, &mut dst)?;
        backup.run_to_completion(5, Duration::from_millis(0), Some(progress))
    }
}