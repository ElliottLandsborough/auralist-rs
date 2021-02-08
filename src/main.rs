use std::{env};
use rusqlite::{params, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
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

    match SQLite::backup_to_gz() {
        Ok(file) => file,
        Err(err) => println!("{}", err),
    };

    // Restore connection from db file
    match SQLite::restore_from_gz() {
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

    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        let path = entry.path().to_str().unwrap().to_string();
        let path_hash = xxh3::hash64(path.as_bytes()).to_string();

        let f = File::new(
            &path,
            &path_hash
        );

        f.save_to_database();
    }

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