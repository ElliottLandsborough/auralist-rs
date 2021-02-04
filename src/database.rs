use rusqlite::Connection as RuConnection;
use rusqlite::OpenFlags;

pub struct SQLite;

impl SQLite {
    pub fn connect() -> RuConnection {
        match RuConnection::open("file:blah?mode=memory&cache=shared") {
            Ok(conn) => conn,
            Err(error) => panic!("Cannot connect to SQLite: {}", error)
        }
    }
}