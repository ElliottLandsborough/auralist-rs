use rusqlite::{params, Result as SQLiteResult};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::path::Path;
use walkdir::WalkDir;
use warp::{http::Method, http::StatusCode, Filter, Rejection, Reply};

mod database;
mod dbbackup;
use crate::database::SQLite;
mod config;
use crate::config::{ConfigFile, Settings};
mod music;
use crate::music::File;

use std::{
    fs::File as StdFsFile,
    io::{prelude::*, BufReader},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = ConfigFile::new();

    if (args.len()) > 1 {
        let command = &args[1];

        match command.as_str() {
            "init" => config.create(),
            "index" => index(),
            "serve" => serve(),
            _ => println!("Please choose a command e.g 'init' or 'index'"),
        }
    }
}

fn index() {
    let _conn = SQLite::initialize();

    let conf_file = "./conf.ini";

    if !Path::new(conf_file).exists() {
        println!(
            "Config file `{:?}` missing. Please run `init` first",
            conf_file
        );

        return;
    }

    let directory_to_index = Settings::get("Indexer", "directory_to_index");

    if !Path::new(&directory_to_index).exists() {
        println!(
            "Directory set in `conf.ini` missing: `{:?}`",
            &directory_to_index
        );

        return;
    }

    let directory_exclusions_file_path = Settings::get("Indexer", "directory_exclusions");

    if directory_exclusions_file_path.len() > 0
        && !Path::new(&directory_exclusions_file_path).exists()
    {
        println!(
            "Exclusions file is missing: `{:?}`",
            &directory_exclusions_file_path
        );

        return;
    }

    let directory_exclusions = lines_from_file(directory_exclusions_file_path);

    let binding = Settings::get("Indexer", "extensions_to_index");
    let extensions_to_index: Vec<&str> = binding.split(",").collect();

    match get_files(
        String::from(&directory_to_index),
        directory_exclusions,
        extensions_to_index,
    ) {
        Ok(_) => println!("Finished getting files."),
        Err(err) => println!("{}", err),
    }

    // Try to query restored db
    match test_db() {
        Ok(_) => println!("Success."),
        Err(err) => println!("Test error: {}", err),
    }
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = StdFsFile::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

fn get_files(
    directory: std::string::String,
    exclusions: Vec<std::string::String>,
    extensions: Vec<&str>,
) -> Result<(), walkdir::Error> {
    println!("Saving files to db...");

    'entries: for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        let path = entry.path();

        println!("+ PATH: `{:?}`", &path);

        for exclusion in &exclusions {
            if path.starts_with(exclusion) {
                println!("Excluding: `{:?}`", &path);
                println!("Based on rule: `{:?}`", &exclusion);
                continue 'entries;
            }
        }

        if !path.is_dir() {
            let f = File::populate_from_path(&path, extensions.clone());
            // todo: add all extensions that lofty supports, exts are now specified in conf file
            if f.file_ext == "mp3" || f.file_ext == "flac" {
                // renember to not use wav, its too big!
                f.save_to_database();
            }
        }
    }

    Ok(())
}

fn test_db() -> SQLiteResult<()> {
    let query = "SELECT id, path, file_name, file_ext, file_size, file_modified, album, artist, title, duration, indexed_at FROM files LIMIT 0, 5";

    let conn = SQLite::connect();
    let mut stmt = conn.prepare(query)?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            file_name: row.get(2)?,
            file_ext: row.get(3)?,
            file_size: row.get(4)?,
            file_modified: row.get(5)?,
            album: row.get(6)?,
            artist: row.get(7)?,
            title: row.get(8)?,
            duration: row.get(9)?,
            indexed_at: row.get(10)?,
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

fn search_result_to_file(
    id: i64,
    path: String,
    file_name: String,
    file_ext: String,
    file_size: u64,
    file_modified: u64,
    title: String,
    artist: String,
    album: String,
    duration: u64,
    indexed_at: u64,
) -> SQLiteResult<File> {
    let file = File {
        id,
        path,
        file_name,
        file_ext,
        file_size,
        file_modified,
        title,
        artist,
        album,
        duration: duration,
        indexed_at,
    };

    Ok(file)
}

fn search_db(input: String) -> SQLiteResult<Vec<File>> {
    let query = "SELECT * FROM `search` WHERE `search` MATCH :input;";

    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;
    //let mut stmt = conn.prepare("SELECT * FROM `files` where `artist` LIKE :query;")?;

    let rows = stmt.query_and_then(&[(":input", &input)], |row| {
        search_result_to_file(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for result in rows {
        let mut file = result.unwrap();
        file.get_unique_id();
        files.push(file);
    }

    Ok(files)
}

fn random_song() -> SQLiteResult<Vec<File>> {
    let query = "SELECT id, path, file_name, file_ext, file_size, file_modified, title, artist, album, duration, indexed_at FROM `files` WHERE `file_ext` IN ('mp3', 'flac') AND _ROWID_ >= (abs(random()) % (SELECT max(_ROWID_) FROM `files`)) LIMIT 1;";

    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_map(params![], |row| {
        search_result_to_file(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for result in rows {
        let mut file = result.unwrap();
        file.get_unique_id();
        files.push(file);
    }

    Ok(files)
}

fn find_song_by_hash(input: String) -> SQLiteResult<Vec<File>> {
    let query = "SELECT id, path, file_name, file_ext, title, artist, album, duration FROM `files` WHERE `id` IN (SELECT file FROM plays WHERE hash = :input) LIMIT 0, 1;";

    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_and_then(&[(":input", &input)], |row| {
        search_result_to_file(
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
            row.get(11)?,
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

        return Ok(file);
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
    match SQLite::restore() {
        Ok(_) => println!("Success."),
        Err(err) => println!("{}", err),
    }

    // default e.g https://domain.tld
    let default = warp::path::end().and(warp::fs::file("static/index.html"));

    // domain.tld/bundle.js
    let bundle = warp::path!("bundle.js").and(warp::fs::file("static/bundle.js"));

    // domain.tld/favicon.svg
    let favicon = warp::path!("favicon.svg").and(warp::fs::file("static/favicon.svg"));

    // domain.tld/search/[anything]
    let search = warp::path!("search" / String).map(|query| {
        let files = match search_db(query) {
            Ok(files) => files,
            Err(error) => panic!("Problem with search: {:?}", error),
        };

        let response = FileResponse {
            status: 200,
            message: "OK".to_string(),
            count: files.len(),
            data: files,
        };

        warp::reply::json(&response)
    });

    // domain.tld/random
    let random = warp::path!("random").map(|| {
        let files = match random_song() {
            Ok(files) => files,
            Err(error) => panic!("Problem with search: {:?}", error),
        };

        let response = FileResponse {
            status: 200,
            message: "OK".to_string(),
            count: files.len(),
            data: files,
        };

        warp::reply::json(&response)
    });

    // domain.tld/stream/[anything] (parses range headers)
    let stream = warp::path!("stream" / String)
        .and(filter_range())
        .and_then(move |hash: String, range_header: String| get_range(range_header, hash))
        .map(with_partial_content_status);

    // domain.tld/stream/[anything] (when stream headers are missing)
    let download = warp::path!("stream" / String)
        .and_then(move |hash: String| get_range("".to_string(), hash));

    let cors = warp::cors()
        .allow_origins(vec!["https://randomsound.uk", "http://localhost:1338", "http://localhost:1337", "http://192.168.2.41:1337"])
        .allow_methods(&[Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(vec!["Authorization", "Content-Type", "User-Agent"]);
    //.allow_headers(vec!["Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers"]);

    let gets = warp::get()
        .and(
            default
                .or(favicon)
                .or(bundle)
                .or(search)
                .or(random)
                .or(stream)
                .or(download),
        )
        .with(cors)
        .recover(handle_rejection);

    warp::serve(gets).run(([0, 0, 0, 0], 1337)).await;
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!(
            "unhandled error: {:?}",
            err
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}

// borrowed from warp-range
use async_stream::stream;
use std::{cmp::min, io::SeekFrom, num::ParseIntError};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use warp::{http::HeaderValue, hyper::Body, hyper::HeaderMap, reply::WithStatus};

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

fn get_range_params(range: &str, size: u64) -> Result<(u64, u64), Error> {
    let range: Vec<String> = range
        .replace("bytes=", "")
        .split("-")
        .filter_map(|n| {
            if n.len() > 0 {
                Some(n.to_string())
            } else {
                None
            }
        })
        .collect();
    let start = if range.len() > 0 {
        range[0].parse::<u64>()?
    } else {
        0
    };
    let end = if range.len() > 1 {
        range[1].parse::<u64>()?
    } else {
        size - 1
    };
    Ok((start, end))
}

#[derive(Debug)]
struct Error {
    message: String,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}

async fn internal_get_range(range_header: String, hash: String) -> Result<impl warp::Reply, Error> {
    let path = get_path_from_hash(hash.clone());
    let mime = get_mime_from_hash(hash);
    println!("RANGE: {}", path);
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
    let body = Body::wrap_stream(stream);
    let mut response = warp::reply::Response::new(body);

    let headers = response.headers_mut();
    let mut header_map = HeaderMap::new();
    header_map.insert("Content-Type", HeaderValue::from_str(&mime).unwrap());
    header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
    header_map.insert(
        "Content-Range",
        HeaderValue::from_str(&format!("bytes {}-{}/{}", start_range, end_range, size)).unwrap(),
    );
    header_map.insert("Content-Length", HeaderValue::from(byte_count));
    headers.extend(header_map);
    Ok(response)
}
