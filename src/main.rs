use serde::{Deserialize, Serialize};
use std::thread;
mod music;
use crate::music::File;
use crate::music::FileHashed;
use parking_lot::{Mutex, MutexGuard};
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use warp::{http::Method, http::StatusCode, Filter, Rejection, Reply};

#[derive(Debug)]
struct InvalidParameter;

impl warp::reject::Reject for InvalidParameter {}

#[derive(Clone)]
struct IndexedFiles {
    pub files: HashMap<u32, File>,
    pub mixes: Vec<u32>,
    pub tunes: Vec<u32>,
    pub all: Vec<u32>,
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

fn main() {
    let indexed_files = synchronous_file_scan();

    // random ids that need to be sought
    let plays: HashMap<String, File> = HashMap::new();
    let plays_mutex = Arc::new(Mutex::new(plays));

    thread::scope(|s| {
        s.spawn(|| {
            println!("Starting web server...");
            serve(indexed_files, plays_mutex);
        });
        println!("Hello from the main... \\m/");
    });
}

#[tokio::main]
async fn serve(indexed_files: IndexedFiles, plays_mutex: Arc<Mutex<HashMap<String, File>>>) {
    println!("SERVING");

    let plays_mutex_1 = Arc::clone(&plays_mutex);
    let plays_mutex_2 = Arc::clone(&plays_mutex);
    let plays_mutex_3 = Arc::clone(&plays_mutex);

    // default e.g https://domain.tld
    let default = warp::path::end().and(warp::fs::file("static/index.html"));

    // domain.tld/js/*
    let js = warp::path("js")
        .and(warp::fs::dir("static/js"))
        .map(|res: warp::fs::File| {
            // cache for 23 days
            warp::reply::with_header(
                res,
                "cache-control",
                "Cache-Control: public, max-age 1987200, s-maxage 1987200, immutable",
            )
        });

    // domain.tld/random
    let random = warp::path!("random" / String).map(move |selection: String| {
        println!("START (route:random)...");
        let random_hash = random_hash(selection.to_string(), indexed_files.clone());
        let duration = Duration::new(0, 500_000_000);
        thread::sleep(duration);
        let response = generate_random_response(random_hash, indexed_files.clone(), &plays_mutex_1);
        println!("END (route:random)...");
        response
    });

    // domain.tld/stream/[anything] (parses range headers)
    let stream = warp::path!("stream" / String)
        .and(filter_range())
        .and_then(move |hash: String, range_header: String| {
            println!("START (stream/[anything])...");

            // hash e.g 1f768ac1-6e83-4f12-a4c3-ad37f6d93844
            let sliced_hash = hash[0..36].to_string();

            get_range(range_header, sliced_hash, Arc::clone(&plays_mutex_2))
        })
        .map(with_partial_content_status);

    // domain.tld/stream/[anything] (when stream headers are missing)
    let download = warp::path!("stream" / String).and_then(move |hash: String| {
        // hash e.g 1f768ac1-6e83-4f12-a4c3-ad37f6d93844
        let sliced_hash = hash[0..36].to_string();
        get_range("".to_string(), sliced_hash, Arc::clone(&plays_mutex_3))
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
        .and(default.or(random).or(stream).or(download).or(js))
        .with(cors)
        .recover(handle_rejection);

    warp::serve(gets).run(([0, 0, 0, 0], 1337)).await;
}

fn synchronous_file_scan() -> IndexedFiles {
    let directory_to_index = "./files";

    // murmurs with their file counterparts
    let files = get_files(directory_to_index.to_string());

    // murmurs of mixes
    let mut mixes: Vec<u32> = Vec::new();

    // murmurs of tunes
    let mut tunes: Vec<u32> = Vec::new();

    // murmurs of all
    let mut all: Vec<u32> = Vec::new();

    for (_, f) in files.iter() {
        let mix_threshold = 23 * 60;

        all.push(f.id);

        if f.duration > mix_threshold {
            mixes.push(f.id);
        } else {
            tunes.push(f.id);
        }
    }

    return IndexedFiles {
        files,
        mixes,
        tunes,
        all,
    };
}

fn get_files(directory: std::string::String) -> HashMap<u32, File> {
    println!("Walking files...");

    let mut files: HashMap<u32, File> = HashMap::new();

    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        let path = entry.path();

        println!("+ PATH: `{:?}`", &path);

        if !path.is_dir() {
            let mut f = File::new_empty_file_from_path(&path);
            f.populate_from_path();
            f.indexed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // get info from tags if possible
            // https://docs.rs/lofty/latest/lofty/#supported-formats
            if f.file_ext == "mp3" || f.file_ext == "flac" {
                println!("---------- TRIGGER A LOFTY POPULATE...");
                f.populate_lofty();
            }

            // Skip files that couldn't be parsed by id3
            if f.parse_fail {
                continue;
            }

            // Skip files longer than 12000 seconds
            if f.duration > 12000 {
                continue;
            }

            files.insert(f.id, f);
        }
    }

    return files;
}

fn random_hash(mode: String, state: IndexedFiles) -> u32 {
    let mut selection = Vec::new();

    if mode.len() == 0 {
        println!("WARN: Mode is empty, defaulting to 'mixes'");
    }

    if mode.len() > 100 {
        println!("WARN: Mode too huge, defaulting to 'mixes'");
    }

    if mode == "mixes" {
        selection = state.mixes.clone();
    }

    if mode == "tunes" {
        selection = state.tunes.clone();
    }

    if mode == "all" {
        selection = state.all.clone();
    }

    if mode != "mixes" && mode != "tunes" && mode != "all" {
        println!("WARN: Mode not recognized, defaulting to 'mixes'");
        selection = state.mixes.clone();
    }

    let ten_random_hashes: Vec<&u32> = selection
        .choose_multiple(&mut rand::thread_rng(), 30)
        .collect();

    let random_hash_opt: Option<&u32>;

    let answer: u32;

    if ten_random_hashes.len() == 0 {
        println!("WARN: Fall back to 'old random'");
        random_hash_opt = selection.choose(&mut rand::thread_rng());

        answer = match random_hash_opt {
            Some(random_hash_opt) => u32::from(random_hash_opt.clone()),
            None => 0,
        };
    } else {
        println!("OK: Picked a tune'");
        let borrowerd_random_hash = ten_random_hashes
            .choose(&mut rand::thread_rng())
            .clone()
            .unwrap();
        let random_hash = *borrowerd_random_hash;
        answer = *random_hash;
    }

    return answer;
}

fn generate_random_response(
    random_hash: u32,
    indexed_files: IndexedFiles,
    plays_mutex: &Arc<Mutex<HashMap<String, File>>>,
) -> warp::reply::Json {
    let mut random_files: Vec<File> = Vec::new();

    match indexed_files.files.get(&random_hash) {
        Some(file) => random_files.push(file.clone()),
        _ => println!("ERROR: Hash is missing from db"),
    };

    let mut random_files_hashed: Vec<FileHashed> = Vec::new();
    for file in random_files {
        let file_hashed = file.clone().to_response();
        random_files_hashed.push(file_hashed.clone());

        let mut file = file.clone();
        file.accessed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!("Locking plays (generate_random_response)...");
        let mut plays = plays_mutex.lock();
        plays.insert(file_hashed.path.clone(), file);
        MutexGuard::unlock_fair(plays);
        println!("Unlocked plays (generate_random_response)...");
    }

    let response = FileResponse {
        status: 200,
        message: "OK".to_string(),
        count: random_files_hashed.len(),
        data: random_files_hashed,
    };

    warp::reply::json(&response)
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

// This function filters and extracts the "Range"-Header
fn filter_range() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header::<String>("Range")
}

// This function adds the "206 Partial Content" header
fn with_partial_content_status<T: Reply>(reply: T) -> WithStatus<T> {
    warp::reply::with_status(reply, StatusCode::PARTIAL_CONTENT)
}

// This function retrives the range of bytes requested by the web client
pub async fn get_range(
    range_header: String,
    hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Result<impl warp::Reply, Rejection> {
    let file_option = get_file_from_hash_old(hash.clone(), plays_mutex);

    if file_option.is_none() {
        println!(
            "Error in internal_get_range: get_file_from_hash returned None for hash: `{:?}`",
            hash.to_string()
        );
        return Err(warp::reject::custom(InvalidParameter));
    }

    let file = file_option.unwrap();

    return internal_get_range(file, range_header).await.map_err(|e| {
        println!("Error in get_range: {}", e.message);
        warp::reject()
    });
}

fn get_file_from_hash_old(
    hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Option<music::File> {
    println!("Locking plays (get_file_from_hash)...");
    let plays = plays_mutex.lock();
    let result = match plays.get(&hash) {
        Some(file) => Some(file.clone()),
        None => None,
    };
    MutexGuard::unlock_fair(plays);
    println!("Unlocked plays (get_file_from_hash)...");

    println!("END (get_file_from_hash)...");
    return result;
}

/*
// This function retrives the range of bytes requested by the web client
async fn get_range_new(
    range_header: String,
    sliced_hash: String,
    plays_mutex: Arc<Mutex<HashMap<String, File>>>,
) -> Result<impl warp::Reply, Rejection> {
    // Acquire and drop mutex
    println!("Locking plays (get_range)...");
    let plays = plays_mutex.lock().unwrap();
    // can we get the hash from the list?
    let file_option = match plays.get(&sliced_hash) {
        Some(file) => Some(file.clone()),
        None => None,
    };
    println!("Unlocking plays (get_range)...");
    drop(plays);

    // If no, panic
    if file_option.is_none() {
        println!(
            "Error in internal_get_range: get_file_from_hash returned None for hash: `{:?}`",
            sliced_hash.to_string()
        );
        // Todo: return something proper
        //panic!("Error in internal_get_range: get_file_from_hash returned None for hash: `{:?}`", sliced_hash.to_string());
        return Err(warp::reject::custom(InvalidParameter));
    }

    let file = file_option.unwrap();

    return internal_get_range(file, range_header).await.map_err(|e| {
        println!("Error in get_range: {}", e.message);
        warp::reject()
    });
}
*/

async fn internal_get_range(file: File, range_header: String) -> Result<impl warp::Reply, Error> {
    let path = &file.path;
    let guess = mime_guess::from_ext(&file.file_ext).first().unwrap();
    let mime = guess.essence_str();
    let mut file = tokio::fs::File::open(path).await?;
    let metadata = file.metadata().await?;
    let size = metadata.len();
    let (start_range, end_range) = parse_range_header(&range_header, size)?;
    let mut limited_end_range = end_range;
    if end_range > size {
        println!("ERROR: Range larger than file size detected");
        limited_end_range = size
    }
    let byte_count = limited_end_range - start_range + 1;
    file.seek(SeekFrom::Start(start_range)).await?;

    let stream = stream! {
        //let bufsize = 16384; // 16kb?
        let bufsize = 1024 * 512; // 512kb
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
        HeaderValue::from_str(&format!(
            "bytes {}-{}/{}",
            start_range, limited_end_range, size
        ))
        .unwrap(),
    );
    header_map.insert("Content-Length", HeaderValue::from(byte_count));
    headers.extend(header_map);

    Ok(response)
}

fn parse_range_header(range: &str, size: u64) -> Result<(u64, u64), Error> {
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
