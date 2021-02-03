use std::{env};
use rusqlite::{backup, params, Connection, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use std::time::Duration;
use ini::Ini;
use std::io::prelude::*;
use flate2::Compression;
use flate2::bufread::GzEncoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Result as ioResult;

#[derive(Debug)]
struct IndexedFile {
    id: i32,
    path: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let conn = Connection::open_in_memory().unwrap();

    if (args.len()) > 1 {
        let command = &args[1];

        if command == "init" {
            init();
            return;
        }

        if command == "index" {
            index(conn);
            return;
        }
    }

    println!("Please choose a command e.g 'init' or 'index'");
}

fn init() {
    let configPath = "conf.ini";

    if Path::new(configPath).exists() {
        println!("Could not create `conf.ini` as it already exists.");

        return;
    }

    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("encoding", "utf-8");
    conf.with_section(Some("Files"))
        .set("directory_to_index", "~/Music")
        .set("extensions_to_index", "*");
    conf.write_to_file(configPath).unwrap();
}

fn index(conn: Connection) {
    let conf = Ini::load_from_file("conf.ini").unwrap();

    let section = conf.section(Some("Files")).unwrap();
    let directory = section.get("directory_to_index").unwrap();

    if !Path::new(directory).exists() {
        println!("Directory set in `conf.ini` missing: `{:?}`", directory);

        return;
    }

    initialize_db(&conn);

    get_files(String::from(directory), &conn);

    test_db(&conn);

    backup_db_to_file(&conn, "./auralist.sqlite3", db_backup_progress);

    compress_file("./auralist.sqlite3");

    //decompress_file("./auralist.sqlite3");

    //restore_db();
}

fn db_backup_progress(progress: backup::Progress) {
    // todo: the progress...
    println!("Backing up...");
}

fn initialize_db(conn: &Connection) {
    conn.execute(
        "CREATE TABLE file (
                  id              INTEGER PRIMARY KEY,
                  path            TEXT NOT NULL
                  )",
        params![],
    );
}

fn get_files(directory: std::string::String, conn: &Connection) -> Result<i32, walkdir::Error> {
    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        println!("{}", entry.path().display());

        let full_path = entry.path().to_str().unwrap();

        save_file_in_db(String::from(full_path), &conn);
    }

    Ok(0)
}

fn save_file_in_db(path: std::string::String, conn: &Connection) -> SQLiteResult<()> {    
    let f = IndexedFile {
        id: 0,
        path: path,
    };

    conn.execute(
        "INSERT INTO file (path) VALUES (?1)",
        params![f.path],
    )?;

    Ok(())
}

fn test_db(conn: &Connection) -> SQLiteResult<()> {
    let mut stmt = conn.prepare("SELECT id, path FROM file")?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(IndexedFile {
            id: row.get(0)?,
            path: row.get(1)?,
        })
    })?;

    println!("Found file {:?}", "RESULT:");

    for file in file_iter {
        println!("Found file {:?}", file.unwrap());
    }

    Ok(())
}

fn backup_db_to_file<P: AsRef<Path>>(
    src: &Connection,
    dst: P,
    progress: fn(backup::Progress),
) -> SQLiteResult<()> {
    let mut dst = Connection::open(dst)?;
    let backup = backup::Backup::new(src, &mut dst)?;
    backup.run_to_completion(5, Duration::from_millis(250), Some(progress))
}

fn compress_file<P: AsRef<Path>>(path: P) /*-> ioResult<Vec<u8>>*/ {
    let f = File::open(path);
    let b = BufReader::new(f.unwrap());
    let gz = GzEncoder::new(b, Compression::fast());
    //let mut buffer = Vec::new();
    //gz.read_to_end(&mut buffer)?;
    //Ok(buffer)

    for byte in gz.bytes() {
        println!("{}", byte.unwrap());
        // todo: output every 50kb to file
        // todo: decompress file
        // todo: restore db from file
    }
}