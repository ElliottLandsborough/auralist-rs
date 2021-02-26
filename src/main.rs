use std::{env};
use rusqlite::{params, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use warp::Filter;
use serde::{Serialize, Deserialize};

mod database;
mod dbbackup;
use crate::database::SQLite;
mod config;
use crate::config::{ConfigFile, Settings};
mod music;
use crate::music::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = ConfigFile::new();

    if (args.len()) > 1 {
        let command = &args[1];

        match command.as_str() {
            "init" => config.create(),
            "index" => index(),
            "serve" => serve(),
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

        let path = entry.path();

        if !path.is_dir() {
            let f = File::populate_from_path(&path);
            f.save_to_database();
        }
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
            path_hash: row.get(2)?,
            file_name: "".to_string(),
            album: "".to_string(),
            artist: "".to_string(),
            title: "".to_string(),
        })
    })?;

    for file in file_iter {
        match file {
            Ok(file) => println!("Found file id {:?}", file.id),
            Err(error) => println!("ERROR: {:?}", error),
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct EmptyResponse {
    pub status: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileResponse {
    pub status: i32,
    pub message: String,
    pub data: Vec<File>,
}

#[tokio::main]
async fn serve() {
    // domain.tld
    let root = warp::path::end()
        .map(|| {
            let response = EmptyResponse {
                status: 200,
                message: "OK".to_string(),
            };

            warp::reply::json(&response)
        });
    
    // domain.tld/search/[anything]
    let search = warp::path!("search" / String)
        .map(|name| format!("Searching for, {}!", name));

    let routes = root
        .or(search);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}