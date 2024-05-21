#[macro_use] extern crate rocket;

use serde::{Deserialize, Serialize};
mod music;
use std::collections::HashMap;
use crate::music::File;
use walkdir::WalkDir;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::SliceRandom;
use rocket::State;
use crate::music::FileHashed;
use rocket::serde::json::Json;
use dashmap::DashMap;
use rocket::fs::FileServer;

struct IndexedFiles {
    pub files: HashMap<u32, File>,
    pub mixes: Vec<u32>,
    pub tunes: Vec<u32>,
    pub all: Vec<u32>,
}

struct LiveStats {
    pub plays: DashMap<String, File>,
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

#[get("/<selection>")]
async fn random(
    selection: &str,
    indexed_files: &State<IndexedFiles>,
    live_stats: &rocket::State<Arc<LiveStats>>,
) -> Json<FileResponse> {
    let random_hash = random_hash(selection.to_string(), indexed_files);
    generate_random_response(random_hash, indexed_files, live_stats)
}

#[get("/<hash>")]
async fn stream(
    hash: &str,
    indexed_files: &State<IndexedFiles>,
    live_stats: &rocket::State<Arc<LiveStats>>,
    range: Range<'_>,
) -> Json<FileResponse> {
    // hash e.g 1f768ac1-6e83-4f12-a4c3-ad37f6d93844
    let sliced_hash = hash[0..36].to_string();

    // can we get the hash from the list?
    let plays = live_stats.plays.clone();
    let file_option = match plays.get(hash.clone()) {
        Some(file) => Some(file.clone()),
        None => None,
    };

    // If no, panic
    if file_option.is_none() {
        println!(
            "Error in internal_get_range: get_file_from_hash returned None for hash: `{:?}`",
            hash.to_string()
        );
        // Todo: return something proper
        panic!("Error in internal_get_range: get_file_from_hash returned None for hash: `{:?}`", hash.to_string());
    }

    let file = file_option.unwrap();

    let file_size = file.file_size;

    let range_contents = match range {
        Range(something) => {
            something
        },
        _ => {
            // possibly throw here, not sure.
            "bytes=0-0"
        }
    };

    let parsed_range_contents = parse_range_header(range_contents, file_size);

    println!("PARSED Range CONTENTS: {:?}", parsed_range_contents);

    // these lines are temp
    let random_hash = random_hash(sliced_hash.to_string(), indexed_files);
    generate_random_response(random_hash, indexed_files, live_stats)
}


#[launch]
fn rocket() -> _ {
    // random ids that need to be sought
    let plays: DashMap<String, File> = DashMap::new();

    let state = synchronous_file_scan();

    rocket::build()
        .mount("/", FileServer::from("./static"))
        .mount("/random", routes![random])
        .mount("/stream", routes![stream])
        .manage(state)
        .manage(Arc::new(LiveStats{plays}))
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

    return IndexedFiles{files, mixes, tunes, all};
}

fn get_files(
    directory: std::string::String,
) -> HashMap<u32, File> {
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
    };

    return files;
}

fn random_hash(mode: String, state: &State<IndexedFiles>) -> u32 {
    let mut selection = state.all.clone();

    if mode.len() == 0 {
        println!("WARN: Mode is empty, defaulting to 'mixes'");
    }

    if mode.len() > 100 {
        println!("WARN: Mode too huge, defaulting to 'mixes'");
    }

    if mode == "tunes" {
        selection = state.tunes.clone();
    }

    if mode == "mixes" {
        selection = state.mixes.clone();
    }

    let ten_random_hashes: Vec<&u32> = selection.choose_multiple(&mut rand::thread_rng(), 30).collect();

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
        let borrowerd_random_hash = ten_random_hashes.choose(&mut rand::thread_rng()).clone().unwrap(); 
        let random_hash = *borrowerd_random_hash;    
        answer = *random_hash;
    }

    return answer;
}

fn generate_random_response(
    random_hash: u32,
    indexed_files: &State<IndexedFiles>,
    live_stats: &rocket::State<Arc<LiveStats>>,
) -> Json<FileResponse> {
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

        
        live_stats.plays.insert(file_hashed.path.clone(), file);
    }

    let response = FileResponse {
        status: 200,
        message: "OK".to_string(),
        count: random_files_hashed.len(),
        data: random_files_hashed,
    };

    Json(response)
}


/*
get_range(file: File, range_header: String) -> String {

    //let (start_range, end_range) = get_range_params(&range_header, size)?;
    //println!("Ranging from {} to {}", start_range, end_range);
    /*
    let path = &file.path;
    let guess = mime_guess::from_ext(&file.file_ext).first().unwrap();
    let mime = guess.essence_str();
    let mut file = tokio::fs::File::open(path).await?;
    let metadata = file.metadata().await?;
    let size = metadata.len();
    let (start_range, end_range) = get_range_params(&range_header, size)?;
    let mut limited_end_range = end_range;
    if end_range > size {
        println!("::::::::::: Range larger than file size detected");
        limited_end_range = size
    }
    let byte_count = limited_end_range - start_range + 1;
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
        HeaderValue::from_str(&format!("bytes {}-{}/{}", start_range, limited_end_range, size)).unwrap(),
    );
    header_map.insert("Content-Length", HeaderValue::from(byte_count));
    headers.extend(header_map);

    Ok(response)
    */
    return "blarg".to_string();
}


*/

// The code below extracts the range header from the request
use rocket::request::{self, Request, FromRequest};
use rocket::request::Outcome;
use rocket::http::Status;

#[derive(Debug)]
struct Range<'r>(&'r str);

#[derive(Debug)]
enum RangeError {
    Missing,
}

//impl<'a, 'r> FromRequest<'a, 'r> for Token {
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Range<'r> {
    type Error = RangeError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let range = req.headers().get_one("Range");
        match range {
            Some(range) => {
                // Limit initial range to 100 characters
                let mut n = range.len();
                if range.len() > 100 {
                    n = 100;
                }
                Outcome::Success(Range(&range[0..n]))
            }
            None => Outcome::Error((Status::BadRequest, RangeError::Missing)),
        }
    }
}

// Borrowed from warp-range
use std::num::ParseIntError;

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