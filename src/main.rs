// Local Packages
mod database;
use crate::database::SQLite;
mod file;
use crate::file::File;

// Remote Packages
use std::{env};
use rusqlite::{backup, params, Connection, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use std::time::Duration;
use ini::Ini;
use flate2::Compression;
use flate2::bufread::GzEncoder;
use flate2::bufread::GzDecoder;
use std::fs::File as FsFile;
use std::io::BufReader;
use twox_hash::xxh3;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _conn = SQLite::initialize();

    if (args.len()) > 1 {
        let command = &args[1];

        match command.as_str() {
            "init" => init(),
            "index" => index(),
            _ => println!("Please choose a command e.g 'init' or 'index'")
        }
    }
}

fn init() {
    let config_path = "conf.ini";

    if Path::new(config_path).exists() {
        println!("Could not create `conf.ini` as it already exists.");

        return;
    }

    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("encoding", "utf-8");
    conf.with_section(Some("Files"))
        .set("directory_to_index", "~/Music")
        .set("extensions_to_index", "*");
    conf.write_to_file(config_path).unwrap();
}

fn index() {
    let conf_file = "conf.ini";
    let db_file = "./auralist.sqlite3";
    let db_backup_file = db_file.to_owned() + ".gz";

    if !Path::new(conf_file).exists() {
        println!("Config file `{:?}` missing. Please run `init` first", conf_file);

        return;
    }

    let conf = Ini::load_from_file(conf_file).unwrap();

    let section = conf.section(Some("Files")).unwrap();
    let directory = section.get("directory_to_index").unwrap();

    if !Path::new(directory).exists() {
        println!("Directory set in `conf.ini` missing: `{:?}`", directory);

        return;
    }

    match get_files(String::from(directory)) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    match backup_db_to_file(db_file, db_backup_progress) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    compress_file(db_file, &db_backup_file);

    decompress_file(&db_backup_file, db_file);

    // Restore connection from db file
    match restore_db_from_file(db_file, db_backup_progress) {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    // Try to query restored db
    match test_db() {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }
}

fn db_backup_progress(p: backup::Progress) {
    let pagecount = f64::from(p.pagecount);
    let remaining = f64::from(p.remaining);

    let remaining = ((pagecount - remaining) / pagecount) * 100.0;

    println!("Progress: {}%", remaining.round());
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

fn backup_db_to_file<P: AsRef<Path>>(
    dst: P,
    progress: fn(backup::Progress),
) -> SQLiteResult<()> {
    println!("Backing up db to file...");
    let src = SQLite::connect();
    let mut dst = Connection::open(dst)?;
    let backup = backup::Backup::new(&src, &mut dst)?;
    backup.run_to_completion(5, Duration::from_millis(0), Some(progress))
}

fn restore_db_from_file<P: AsRef<Path>>(
    src: P,
    progress: fn(backup::Progress),
) -> SQLiteResult<()> {
    println!("Restoring db from file...");
    let src = Connection::open(src)?;
    let mut dst = SQLite::connect();
    let backup = backup::Backup::new(&src, &mut dst)?;
    backup.run_to_completion(5, Duration::from_millis(0), Some(progress))
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