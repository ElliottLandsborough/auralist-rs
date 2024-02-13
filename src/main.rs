use rand::seq::SliceRandom;

use murmurhash32::murmurhash3;
use rusqlite::{params, Result as SQLiteResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use tantivy::doc;
use walkdir::WalkDir;
use warp::{http::Method, http::StatusCode, Filter, Rejection, Reply};

use std::thread::sleep;
use std::time::{Duration, Instant};

mod database;
use crate::database::SQLite;
mod music;
use crate::music::File;
use crate::music::FileHashed;
mod search;
use std::sync::{Arc, Mutex};

use std::{
    fs::File as StdFsFile,
    io::{prelude::*, BufReader},
};

fn main() {
    let files: HashMap<u32, File> = HashMap::new();
    let files_mutex = Arc::new(Mutex::new(files));

    let plays: HashMap<String, File> = HashMap::new();
    let plays_mutex = Arc::new(Mutex::new(plays));

    // murmurs / durations
    let durations: HashMap<u32, u64> = HashMap::new();
    let durations_mutex = Arc::new(Mutex::new(durations));

    // murmurs
    let to_be_indexed: Vec<u32> = Vec::new();
    let to_be_indexed_mutex = Arc::new(Mutex::new(to_be_indexed));

    // murmurs
    let have_been_indexed: Vec<u32> = Vec::new();
    let have_been_indexed_mutex = Arc::new(Mutex::new(have_been_indexed));

    SQLite::initialize();

    load_old_data(files_mutex.clone(), have_been_indexed_mutex.clone());

    thread::scope(|s| {
        s.spawn(|| {
            println!("Logging queues...");
            log_queues(
                files_mutex.clone(),
                plays_mutex.clone(),
                to_be_indexed_mutex.clone(),
                have_been_indexed_mutex.clone(),
            );
        });
        s.spawn(|| {
            println!("Indexing basic file information...");
            index(
                files_mutex.clone(),
                to_be_indexed_mutex.clone(),
                have_been_indexed_mutex.clone(),
            );
        });
        s.spawn(|| {
            println!("Warming database with more file info...");
            warm(
                files_mutex.clone(),
                durations_mutex.clone(),
                to_be_indexed_mutex.clone(),
                have_been_indexed_mutex.clone(),
            );
        });
        s.spawn(|| {
            println!("Starting periodic cleanup tasks...");
            cleanup(plays_mutex.clone());
        });
        s.spawn(|| {
            println!("Starting web server...");
            serve(
                files_mutex.clone(),
                plays_mutex.clone(),
                have_been_indexed_mutex.clone(),
            );
        });
        println!("Hello from the main... \\m/");
    });
}

#[tokio::main]
async fn log_queues(
    files_mutex: Arc<std::sync::Mutex<std::collections::HashMap<u32, music::File>>>,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
    to_be_indexed_mutex: Arc<Mutex<Vec<u32>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) {
    loop {
        let files_mutex = files_mutex.lock().unwrap();
        let plays_mutex = plays_mutex.lock().unwrap();
        let to_be_indexed_mutex = to_be_indexed_mutex.lock().unwrap();
        let have_been_indexed_mutex = have_been_indexed_mutex.lock().unwrap();

        println!("Files: {:?}", files_mutex.len());
        println!("Plays: {:?}", plays_mutex.len());
        println!("To be indexed: {:?}", to_be_indexed_mutex.len());
        println!("Have been indexed: {:?}", have_been_indexed_mutex.len());

        drop(files_mutex);
        drop(plays_mutex);
        drop(to_be_indexed_mutex);
        drop(have_been_indexed_mutex);

        println!("Sleeping for 10 seconds (log_queues)...");
        thread::sleep(time::Duration::from_secs(10));
    }
}

#[tokio::main]
async fn warm(
    files_mutex: Arc<std::sync::Mutex<std::collections::HashMap<u32, music::File>>>,
    durations_mutex: Arc<std::sync::Mutex<std::collections::HashMap<u32, u64>>>,
    to_be_indexed_mutex: Arc<Mutex<Vec<u32>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) {
    let mut i = 0;
    loop {
        let mut to_be_indexed = to_be_indexed_mutex.lock().unwrap();
        let hash_to_be_indexed = to_be_indexed.pop();
        drop(to_be_indexed);

        if !hash_to_be_indexed.is_none() {
            println!("Locking files (warm)...");
            let mut files = files_mutex.lock().unwrap();

            let file = match files.get(&hash_to_be_indexed.unwrap()) {
                Some(file) => Some(file.clone()),
                None => None,
            };

            if !file.is_none() {
                let mut f = file.unwrap();
                f.indexed_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                f = index_and_commit_to_db(&mut f).clone();

                // add to durations map
                let mut durations = durations_mutex.lock().unwrap();
                durations.insert(f.id, f.duration);
                drop(durations);

                if f.clone().parse_fail {
                    files.remove(&hash_to_be_indexed.unwrap());
                } else {
                    *files.get_mut(&hash_to_be_indexed.unwrap()).unwrap() = f;

                    // todo: update search
                    //search::write_index(f)
                }

                println!("Locking have_been_indexed (warm)...");
                let mut have_been_indexed = have_been_indexed_mutex.lock().unwrap();
                have_been_indexed.push(hash_to_be_indexed.unwrap());
                have_been_indexed.dedup();
                println!("Unlocking have_been_indexed (warm)...");
                drop(have_been_indexed);
            }

            println!("Unlocking files (warm)...");
            drop(files);
        } else {
            i = i + 1;
            if i == 10 {
                println!("Sleeping for 20 seconds (warm)...");
                thread::sleep(time::Duration::from_secs(20));
            }
        }
    }
}

fn load_old_data(
    files_mutex: Arc<std::sync::Mutex<HashMap<u32, File>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) {
    // Grab all files from the sqlite database if possible
    println!("+ Loading old data");

    for file in get_all_db_files() {
        println!("+");
        println!("Locking files (load_old_data)...");
        let mut files = files_mutex.lock().unwrap();
        // Add it to our in memory list
        files.insert(file.clone().id, file.clone());
        println!("Unlocking files (load_old_data)...");
        drop(files);

        // Add them all to have been indexed list
        println!("Locking have_been_indexed (load_old_data)...");
        let mut have_been_indexed = have_been_indexed_mutex.lock().unwrap();
        have_been_indexed.push(file.id);
        have_been_indexed.dedup();
        println!("Unlocking have_been_indexed (load_old_data)...");
        drop(have_been_indexed);
    }
}

// todo: where does this belong?
fn get_all_db_files() -> Vec<File> {
    let conn = SQLite::connect();
    let mut stmt = conn
        .prepare("SELECT * FROM files")
        .expect("SQL Statement prepare fail");

    let file_iter = stmt
        .query_map(params![], |row| {
            Ok(File {
                id: row.get(0)?,
                path: row.get(1)?,
                file_name: row.get(2)?,
                file_ext: row.get(3)?,
                file_size: row.get(4)?,
                file_modified: row.get(5)?,
                title: row.get(6)?,
                artist: row.get(7)?,
                album: row.get(8)?,
                duration: row.get(9)?,
                indexed_at: row.get(10)?,
                accessed_at: row.get(11)?,
                parse_fail: row.get(12)?,
            })
        })
        .expect("Error during get_all_db_files query/iteration.");

    let mut files: Vec<File> = Vec::new();

    for file in file_iter {
        match file {
            Ok(file) => files.push(file),
            Err(err) => println!("Could not get file from db into memory: {:?}", err),
        }
    }

    files
}

#[tokio::main]
async fn cleanup(plays_mutex: Arc<Mutex<HashMap<String, File>>>) {
    let interval = Duration::from_secs(5);
    let mut next_time = Instant::now() + interval;

    loop {
        clear_plays(plays_mutex.clone());
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}

fn clear_plays(plays_mutex: Arc<Mutex<HashMap<String, File>>>) {
    // Acquire and drop mutex
    println!("Locking plays (clear_plays)...");
    let plays = plays_mutex.lock().unwrap();
    let iter = plays.clone().into_iter();
    println!("Unlocking plays (clear_plays)...");
    drop(plays);

    for (hash, file) in iter {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let accessed_at = file.accessed_at;
        let duration = file.duration;

        // if enough time has passed for the song to have played 4 times...
        if now - accessed_at > duration * 4 {
            println!("Locking plays (clear_plays)...");
            let mut plays = plays_mutex.lock().unwrap();
            // the url won't work anymore
            println!("Removing a play...");
            println!("Now: {:?}", now);
            println!("Accesed at: {:?}", accessed_at);
            println!("Duration: {:?}", duration);
            println!("Duration*4: {:?}", duration * 4);

            plays.remove(&hash);
            println!("Unlocking plays (clear_plays)...");
            drop(plays);
        }
    }
}

#[tokio::main]
async fn index(
    files_mutex: Arc<std::sync::Mutex<HashMap<u32, File>>>,
    to_be_indexed_mutex: Arc<Mutex<Vec<u32>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) {
    // todo: make this a command line arg
    let directory_to_index = "./files";

    if !Path::new(&directory_to_index).exists() {
        println!(
            "Cannot index files, directory `{:?}` does not exist",
            &directory_to_index
        );

        return;
    }

    let directory_exclusions_file_path = "./exclusions.txt";

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

    match get_files(
        directory_to_index.to_string(),
        directory_exclusions,
        files_mutex,
        to_be_indexed_mutex,
        have_been_indexed_mutex,
    ) {
        Ok(_) => println!("Finished getting files."),
        Err(err) => println!("{}", err),
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
    files_mutex: Arc<std::sync::Mutex<HashMap<u32, File>>>,
    to_be_indexed_mutex: Arc<Mutex<Vec<u32>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) -> Result<(), walkdir::Error> {
    println!("Walking files and saving to vector...");

    let cloned_files_mutex = files_mutex.clone();

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
            // todo: make this a command line arg
            let binding = "flac,mp3";

            let extensions_to_index: Vec<&str> = binding.split(",").collect();

            println!("Run populate_from_path (get_files)...");
            let f = File::populate_from_path(&path);

            if extensions_to_index.contains(&&f.file_ext.as_str()) {
                let file_hash = murmurhash3(f.path.as_bytes());

                let mut index_the_file = false;

                let have_been_indexed = have_been_indexed_mutex.lock().unwrap();
                let current_file_has_been_indexed =
                    match have_been_indexed.binary_search(&file_hash) {
                        Ok(_u) => true,
                        Err(_e) => false,
                    };
                drop(have_been_indexed);

                // The file is not marked as indexed
                if !current_file_has_been_indexed {
                    println!("Locking files (get_files1)...");
                    let mut files = cloned_files_mutex.lock().unwrap();
                    files.insert(file_hash.clone(), f.clone());
                    println!("Unlocking files (get_files1)...");
                    drop(files);
                    index_the_file = true;
                }

                println!("Locking files (get_files2)...");
                let files_mutex = files_mutex.clone();
                let files = files_mutex.lock().unwrap();

                let current_file_in_memory_result = match files.get(&file_hash) {
                    Some(file) => Some(file.clone()),
                    _ => None,
                };

                drop(files);

                if !current_file_in_memory_result.is_none() {
                    let current_file_in_memory = current_file_in_memory_result.unwrap();
                    // File size has changed, index it
                    if f.clone().file_size != current_file_in_memory.file_size {
                        index_the_file = true;
                    }

                    // File modified has changed, index it
                    if f.clone().file_modified != current_file_in_memory.file_modified {
                        index_the_file = true;
                    }

                    if index_the_file == true {
                        println!("Locking to_be_indexed (get_files)...");
                        let mut to_be_indexed = to_be_indexed_mutex.lock().unwrap();
                        to_be_indexed.push(file_hash);
                        to_be_indexed.dedup();
                        println!("Unlocking to_be_indexed (get_files)...");
                        drop(to_be_indexed);
                    }
                }

                println!("Unlocking files (get_files2)...");
            }
        }
        println!("END (get_files)...");
    }

    Ok(())
}

fn index_and_commit_to_db(f: &mut File) -> &mut File {
    // https://docs.rs/lofty/latest/lofty/#supported-formats
    if f.file_ext == "mp3" || f.file_ext == "flac" {
        f.populate_lofty();
    }

    if !f.parse_fail {
        f.save_to_database();
    }

    f
}

fn test_db() -> SQLiteResult<()> {
    let query = "SELECT id, path, file_name, file_ext, file_size, file_modified, title, artist, album, duration, indexed_at, accessed_at FROM files LIMIT 0, 5";

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
            title: row.get(6)?,
            artist: row.get(7)?,
            album: row.get(8)?,
            duration: row.get(9)?,
            indexed_at: row.get(10)?,
            accessed_at: row.get(11)?,
            parse_fail: row.get(12)?,
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
    id: u32,
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
    accessed_at: u64,
    parse_fail: bool,
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
        indexed_at: indexed_at,
        accessed_at: accessed_at,
        parse_fail: parse_fail,
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
            row.get(11)?,
            row.get(12)?,
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
    let query = "SELECT id, path, file_name, file_ext, title, artist, album, duration, indexed_at, accessed_at, parse_fail FROM `files` WHERE `id` IN (SELECT file FROM plays WHERE hash = :input) LIMIT 0, 1;";

    let conn = SQLite::connect();

    let mut stmt = conn.prepare(query)?;

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
            row.get(11)?,
            row.get(12)?,
        )
    })?;

    let mut files: Vec<File> = Vec::new();

    for file in rows {
        files.push(file?);
    }

    Ok(files)
}

fn get_file_from_hash(
    hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Option<music::File> {
    println!("Locking plays (get_file_from_hash)...");
    let plays = plays_mutex.lock().unwrap();
    let result = match plays.get(&hash) {
        Some(file) => Some(file.clone()),
        None => None,
    };
    println!("Unocking plays (get_file_from_hash)...");
    drop(plays);

    println!("END (get_file_from_hash)...");
    return result;
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
    pub data: Vec<FileHashed>,
}

#[tokio::main]
async fn serve(
    files_mutex: Arc<std::sync::Mutex<std::collections::HashMap<u32, music::File>>>,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
    have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>,
) {
    println!("SERVING");
    let plays_mutex_1 = plays_mutex.clone();
    let plays_mutex_2 = plays_mutex.clone();
    let plays_mutex_3 = plays_mutex.clone();
    let have_been_indexed_mutex = have_been_indexed_mutex.clone();

    // default e.g https://domain.tld
    let default = warp::path::end().and(warp::fs::file("static/index.html"));

    // domain.tld/favicon.svg
    let favicon = warp::path("favicon.svg").and(warp::fs::file("static/favicon.svg"));

    // domain.tld/js/*
    let js = warp::path("js").and(warp::fs::dir("static/js"));

    // domain.tld/svg/*
    let svg = warp::path("svg").and(warp::fs::dir("static/svg"));

    /*
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
    */

    // domain.tld/random
    let random = warp::path!("random").and(warp::path::end()).map(move || {
        let m = Arc::clone(&have_been_indexed_mutex);

        let random_hash = random_hash(m);

        if random_hash == 0 {
            let response = EmptyResponse {
                status: 404,
                message: "No files have been indexed (yet...)".to_string(),
            };

            return warp::reply::json(&response);
        }

        let files_mutex = files_mutex.clone();
        let files = files_mutex.lock().unwrap();

        let mut random_files: Vec<File> = Vec::new();

        match files.get(&random_hash) {
            Some(file) => random_files.push(file.clone()),
            _ => println!("Hash is missing from db"),
        };

        drop(files);

        let mut random_files_hashed: Vec<FileHashed> = Vec::new();
        for file in random_files {
            let file_hashed = file.clone().to_response();
            random_files_hashed.push(file_hashed.clone());

            let plays_mutex = plays_mutex_1.clone();

            // Acquire and drop mutex
            println!("Locking plays (serve)...");
            let mut plays = plays_mutex.lock().unwrap();
            let mut file = file.clone();
            file.accessed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            plays.insert(file_hashed.path.clone(), file);
            println!("Unlocking plays (serve)...");
            drop(plays);
        }

        let response = FileResponse {
            status: 200,
            message: "OK".to_string(),
            count: random_files_hashed.len(),
            data: random_files_hashed,
        };

        println!("END (route:random)...");
        return warp::reply::json(&response);
    });

    // domain.tld/stream/[anything] (parses range headers)
    let stream = warp::path!("stream" / String)
        .and(filter_range())
        .and_then(move |hash: String, range_header: String| {
            let plays_mutex = plays_mutex_2.clone();
            get_range(range_header, hash, plays_mutex)
        })
        .map(with_partial_content_status);

    // domain.tld/stream/[anything] (when stream headers are missing)
    let download = warp::path!("stream" / String).and_then(move |hash: String| {
        let plays_mutex = plays_mutex_3.clone();
        get_range("".to_string(), hash, plays_mutex)
    });

    let cors = warp::cors()
        .allow_origins(vec![
            "https://randomsound.uk",
            "http://localhost:1338",
            "http://localhost:1337",
            "http://192.168.2.41:1337",
        ])
        .allow_methods(&[Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(vec!["Authorization", "Content-Type", "User-Agent"]);
    //.allow_headers(vec!["Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers"]);

    let gets = warp::get()
        .and(
            default
                .or(favicon)
                //.or(search)
                .or(random)
                .or(stream)
                .or(download)
                .or(js)
                .or(svg),
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
use std::{cmp::min, io::SeekFrom, num::ParseIntError};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use warp::{http::HeaderValue, hyper::Body, hyper::HeaderMap, reply::WithStatus};

/// This function filters and extracts the "Range"-Header
pub fn filter_range() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header::<String>("Range")
}

/// This function retrives the range of bytes requested by the web client
pub async fn get_range(
    range_header: String,
    hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Result<impl warp::Reply, Rejection> {
    internal_get_range(range_header, hash, plays_mutex)
        .await
        .map_err(|e| {
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

async fn internal_get_range(
    range_header: String,
    hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Result<impl warp::Reply, Error> {
    let file_option = get_file_from_hash(hash, plays_mutex);

    if file_option.is_none() {
        // Todo: return 404 here instead of 500
        return Err(Error {
            message: "Could not range. Hash not found.".to_string(),
        });
    }

    let file = file_option.unwrap();

    let path = &file.path;
    let guess = mime_guess::from_ext(&file.file_ext).first().unwrap();
    let mime = guess.essence_str();
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

fn random_hash(have_been_indexed_mutex: Arc<Mutex<Vec<u32>>>) -> u32 {
    println!("Locking have_been_indexed_mutex (random_hash)...");
    let have_been_indexed = have_been_indexed_mutex.lock().unwrap();
    let random_hash_opt = have_been_indexed.choose(&mut rand::thread_rng());

    let hash = match random_hash_opt {
        Some(random_hash_opt) => u32::from(random_hash_opt.clone()),
        None => 0,
    };

    return hash;
}
