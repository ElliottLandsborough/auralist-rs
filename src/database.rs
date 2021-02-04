use rusqlite::Connection as RuConnection;

pub struct SQLite;

impl SQLite {
    pub fn initialize() -> RuConnection {
        let persist = SQLite::connect();
        SQLite::migrate();

        return persist;
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
}