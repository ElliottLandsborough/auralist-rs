use std::{env};
use rusqlite::{params, Connection, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use flate2::Compression;
use flate2::bufread::GzEncoder;
use flate2::bufread::GzDecoder;
use std::fs::File as FsFile;
use std::io::BufReader;
use twox_hash::xxh3;

mod database;
use crate::database::SQLite;
mod files;
use crate::files::File;
mod config;
use crate::config::{ConfigFile, Settings};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = ConfigFile::new();

    if (args.len()) > 1 {
        let command = &args[1];

        match command.as_str() {
            "init" => config.create(),
            "index" => index(),
            _ => println!("Please choose a command e.g 'init' or 'index'")
        }
    }
}

fn index() {
    let _conn = SQLite::initialize();
    let conf_file = "./conf.ini";

    if !Path::new(conf_file).exists() {
        println!("Config file `{:?}` missing. Please run `init` first", conf_file);

        return;
    }

    let directory_to_index = Settings::get("Files", "directory_to_index");

    if !Path::new(&directory_to_index).exists() {
        println!("Directory set in `conf.ini` missing: `{:?}`", &directory_to_index);

        return;
    }

    match get_files(String::from(&directory_to_index)) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    let db_file = Settings::get("System", "db_file");
    let db_backup_file = db_file.to_owned() + ".gz";

    match SQLite::backup_db_to_file(&db_file) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    compress_file(&db_file, &db_backup_file);

    decompress_file(&db_backup_file, &db_file);

    // Restore connection from db file
    match SQLite::restore_db_from_file(db_file) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    // Try to query restored db
    match test_db() {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }
}

fn get_files(directory: std::string::String) -> Result<(), walkdir::Error> {
    println!("Saving files to db...");
    let conn = SQLite::connect();

    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        let full_path = entry.path().to_str().unwrap();

        match save_file_in_db(String::from(full_path), &conn) {
            Ok(_) => println!("."),
            Err(err) => println!("Update failed: {}", err),
        }
    }

    Ok(())
}

fn save_file_in_db(path: std::string::String, conn: &Connection) -> SQLiteResult<()> {
    let path_hash = xxh3::hash64(path.as_bytes()).to_string();
    
    let f = File {
        id: 0,
        path: path,
        path_hash: path_hash
    };

    conn.execute(
        "INSERT INTO file (path, path_hash) VALUES (?1, ?2)",
        params![
            f.path,
            f.path_hash,
        ],
    )?;

    Ok(())
}

fn test_db() -> SQLiteResult<()> {
    println!("Query: SELECT id, path, path_hash FROM file");
    let conn = SQLite::connect();
    let mut stmt = conn.prepare("SELECT id, path, path_hash FROM file")?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            path_hash: row.get(2)?
        })
    })?;

    for file in file_iter {
        println!("Found file {:?}", file.unwrap());
    }

    Ok(())
}

fn compress_file(source: &str, destination: &str) {
    println!("Compressing file...");
    let f = FsFile::open(source);
    let b = BufReader::new(f.unwrap());
    let mut gz = GzEncoder::new(b, Compression::default());

    // Write contents to disk.
    let mut f = FsFile::create(destination).expect("Unable to create file");
    std::io::copy(&mut gz, &mut f).expect("Unable to copy data");
}

fn decompress_file(source: &str, destination: &str) {
    println!("Decompressing file...");
    let f = FsFile::open(source);
    let b = BufReader::new(f.unwrap());
    let mut gz = GzDecoder::new(b);

    // Write contents to disk.
    let mut f = FsFile::create(destination).expect("Unable to create file");
    std::io::copy(&mut gz, &mut f).expect("Unable to copy data");
}