use std::{env};
use rusqlite::{params, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use std::convert::Infallible;
use warp::{
    http::StatusCode,
    Filter, Rejection, Reply,
};
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
    println!("Query: SELECT id, path FROM files");
    let conn = SQLite::connect();
    let mut stmt = conn.prepare("SELECT id, path, file_name, file_ext, album, artist, title FROM files")?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            file_name: row.get(2)?,
            file_ext: row.get(3)?,
            album: row.get(4)?,
            artist: row.get(5)?,
            title: row.get(6)?,
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

fn search_result_to_file(id: i64, path: String, file_name: String, file_ext: String, title: String, artist: String, album: String) -> SQLiteResult<File> {
    let mut file = File {
        id: id,
        path: path,
        file_name: file_name,
        file_ext: file_ext,
        album: album,
        artist: artist,
        title: title,
    };

    file.get_unique_id();

    Ok(file)
}

fn search_db(input: String) -> SQLiteResult<Vec<File>> {
    let query = "SELECT * FROM `search` WHERE `search` MATCH :input;";
    println!("{}", query);
    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;
    //let mut stmt = conn.prepare("SELECT * FROM `files` where `artist` LIKE :query;")?;

    let rows = stmt.query_and_then_named(&[(":input", &input)], |row| {
        search_result_to_file(
            row.get(1)?, // id
            row.get(2)?, // path
            row.get(3)?, // filename
            row.get(4)?, // ext
            row.get(5)?, // title
            row.get(6)?, // artist
            row.get(7)?, // album
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
}

fn random_song() -> SQLiteResult<Vec<File>> {
    let query = "SELECT id, path, file_name, file_ext, title, artist, album FROM `files` WHERE `file_ext` = 'mp3' AND _ROWID_ >= (abs(random()) % (SELECT max(_ROWID_) FROM `files`)) LIMIT 1;";
    println!("{}", query);
    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_map(params![], |row| {
        search_result_to_file(
            row.get(0)?, // id
            row.get(1)?, // path
            row.get(2)?, // filename
            row.get(3)?, // ext
            row.get(4)?, // title
            row.get(5)?, // artist
            row.get(6)?, // album
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
}

fn find_song_by_hash(input: String) -> SQLiteResult<Vec<File>> {
    let query = "SELECT id, path, file_name, file_ext, title, artist, album FROM `files` WHERE `id` IN (SELECT file FROM plays WHERE hash = :input) LIMIT 0, 1;";
    println!("{}", query);
    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_and_then_named(&[(":input", &input)], |row| {
        search_result_to_file(
            row.get(0)?, // id
            row.get(1)?, // path
            row.get(2)?, // filename
            row.get(3)?, // ext
            row.get(4)?, // title
            row.get(5)?, // artist
            row.get(6)?, // album
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
}

fn first_song_from_vec(files: Vec<File>) -> Result<File, &'static str> {
    if files.len() == 1 {
        // Restore connection from db file
        let file = files.into_iter().nth(0).unwrap();

        return Ok(file)
    }

    Err("No files in supplied vector")
}

fn get_file_from_hash(hash: String) -> Result<File, &'static str> {
    let files = match find_song_by_hash(hash) {
        Ok(files) => files,
        Err(error) => panic!("Problem with search: {:?}", error),
    };

    let file = match first_song_from_vec(files.clone()) {
        Ok(file) => file,
        Err(error) => panic!("Problem with search: {:?}", error),
    };

    Ok(file)
}

fn get_mime_from_hash(hash: String) -> String {
    let file = match get_file_from_hash(hash.to_string()) {
        Ok(file) => file,
        Err(error) => panic!("Problem with file get: {:?}", error),
    };

    let guess = mime_guess::from_ext(&file.file_ext).first().unwrap();
    let mime = guess.essence_str();

    mime.to_string()
}

fn get_path_from_hash(hash: String) -> String {
    let file = match get_file_from_hash(hash.to_string()) {
        Ok(file) => file,
        Err(error) => panic!("Problem with file get: {:?}", error),
    };

    file.path
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

    // domain.tld/stream/[anything]
    let stream = warp::path!("stream" / String)
        .and(filter_range())
        .and_then(move |hash: String, range_header: String| get_range(range_header, hash))
        .map(with_partial_content_status);

    let cors = warp::cors()
        //.allow_any_origin()
        .allow_origins(vec!["https://randomsound.uk", "http://localhost:1338"])
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers"]);

    let gets = warp::get().and(root.or(search).or(random).or(stream)).with(cors).recover(handle_rejection);

    warp::serve(gets)
        .run(([127, 0, 0, 1], 1337))
        .await;
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::NOT_FOUND,
            "Not Found".to_string(),
        )
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}

// borrowed from warp-range
use async_stream::stream;
use std::{
    cmp::min, io::SeekFrom, num::ParseIntError
};
use tokio::io::{
    AsyncReadExt, AsyncSeekExt
};
use warp::{
    http::HeaderValue, hyper::HeaderMap, reply::WithStatus
};

/// This function filters and extracts the "Range"-Header
pub fn filter_range() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header::<String>("Range")
}

/// This function retrives the range of bytes requested by the web client
pub async fn get_range(range_header: String, hash: String) -> Result<impl warp::Reply, Rejection> {
    internal_get_range(range_header, hash).await.map_err(|e| {
        println!("Error in get_range: {}", e.message);
        warp::reject()
    })
}

/// This function adds the "206 Partial Content" header
pub fn with_partial_content_status<T: Reply>(reply: T) -> WithStatus<T> {
    warp::reply::with_status(reply, StatusCode::PARTIAL_CONTENT) 
}

fn get_range_params(range: &str, size: u64)->Result<(u64, u64), Error> {
    let range: Vec<String> = range
        .replace("bytes=", "")
        .split("-")
        .filter_map(|n| if n.len() > 0 {Some(n.to_string())} else {None})
        .collect();
    let start = if range.len() > 0 { 
        range[0].parse::<u64>()? 
    } else { 
        0 
    };
    let end = if range.len() > 1 {
        range[1].parse::<u64>()?
    } else {
        size-1 
    };
    Ok((start, end))
}

#[derive(Debug)]
struct Error {
    message: String
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error { message: err.to_string() }
    }
}
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error { message: err.to_string() }
    }
}

async fn internal_get_range(range_header: String, hash: String) -> Result<impl warp::Reply, Error> {
    
    let path = get_path_from_hash(hash.clone());
    let mime = get_mime_from_hash(hash);

    let mut file = tokio::fs::File::open(path).await?;
    let metadata = file.metadata().await?;
    let size = metadata.len();
    let (start_range, end_range) = get_range_params(&range_header, size)?;
    let byte_count = end_range - start_range + 1;
    file.seek(SeekFrom::Start(start_range)).await?;

    let stream = stream! {
        let bufsize = 16384;
        let cycles = byte_count / bufsize as u64 + 1;
        let mut sent_bytes: u64 = 0;
        for _ in 0..cycles {
            let mut buffer: Vec<u8> = vec![0; min(byte_count - sent_bytes, bufsize) as usize];
            let bytes_read = file.read_exact(&mut buffer).await.unwrap();
            sent_bytes += bytes_read as u64;
            yield Ok(buffer) as Result<Vec<u8>, hyper::Error>;
        }
    };
    let body = hyper::Body::wrap_stream(stream);
    let mut response = warp::reply::Response::new(body);
    
    let headers = response.headers_mut();
    let mut header_map = HeaderMap::new();
    header_map.insert("Content-Type", HeaderValue::from_str(&mime).unwrap());
    header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
    header_map.insert("Content-Range", HeaderValue::from_str(&format!("bytes {}-{}/{}", start_range, end_range, size)).unwrap());
    header_map.insert("Content-Length", HeaderValue::from(byte_count));
    headers.extend(header_map);
    Ok (response)
}