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

    let directory_to_index = Settings::get("Indexer", "directory_to_index");

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
    println!("Query: SELECT id, path, path_hash FROM files");
    let conn = SQLite::connect();
    let mut stmt = conn.prepare("SELECT id, path, path_hash, file_name, file_ext, album, artist, title FROM files")?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            path_hash: row.get(2)?,
            file_name: row.get(3)?,
            file_ext: row.get(4)?,
            album: row.get(5)?,
            artist: row.get(6)?,
            title: row.get(7)?,
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

fn search_result_to_file(path: String, file_name: String, file_ext: String, title: String, artist: String, album: String) -> SQLiteResult<File> {
    Ok(File {
        id: 0, // fix this
        path_hash: "123".to_string(), // fix this
        path: path,
        file_name: file_name,
        file_ext: file_ext,
        album: album,
        artist: artist,
        title: title,
    })
}

fn search_db(input: String) -> SQLiteResult<Vec<File>> {
    let query = "SELECT * FROM `search` WHERE `search` MATCH :input;";
    println!("{}", query);
    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;
    //let mut stmt = conn.prepare("SELECT * FROM `files` where `artist` LIKE :query;")?;

    let rows = stmt.query_and_then_named(&[(":input", &input)], |row| {
        search_result_to_file(
            row.get(0)?, // path
            row.get(1)?, // filename
            row.get(2)?, // ext
            row.get(3)?, // title
            row.get(4)?, // artist
            row.get(5)?, // album
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
}

fn random_song() -> SQLiteResult<Vec<File>> {
    let query = "SELECT path, file_name, file_ext, title, artist, album FROM `files` WHERE `file_ext` = 'mp3' AND _ROWID_ >= (abs(random()) % (SELECT max(_ROWID_) FROM `files`)) LIMIT 1;";
    println!("{}", query);
    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_map(params![], |row| {
        search_result_to_file(
            row.get(0)?, // path
            row.get(1)?, // filename
            row.get(2)?, // ext
            row.get(3)?, // title
            row.get(4)?, // artist
            row.get(5)?, // album
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
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
    pub count: usize,
    pub data: Vec<File>,
}

#[tokio::main]
async fn serve() {
    let _conn = SQLite::initialize();

    // Restore connection from db file
    match SQLite::restore_from_gz() {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

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
        .map(|query| {
            let files = match search_db(query) {
                Ok(files) => files,
                Err(error) => panic!("Problem with search: {:?}", error),
            };

            let response = FileResponse {
                status: 200,
                message: "OK".to_string(),
                count: files.len(),
                data: files
            };

            warp::reply::json(&response)
        });

    // domain.tld/random
    let random = warp::path!("random")
        .map(|| {
            let files = match random_song() {
                Ok(files) => files,
                Err(error) => panic!("Problem with search: {:?}", error),
            };

            let response = FileResponse {
                status: 200,
                message: "OK".to_string(),
                count: files.len(),
                data: files
            };

            warp::reply::json(&response)
        });

    let routes = root
        .or(search)
        .or(random);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 1337))
        .await;
}