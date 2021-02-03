
use lazy_static::lazy_static;
use rusqlite::{Connection};
use std::sync::{Mutex};

lazy_static! {
    pub static ref DB: Mutex<Connection> = {
        let conn = Connection::open_in_memory().expect("Failed to open sqlite connection!");
        Mutex::new(conn)
    };
}