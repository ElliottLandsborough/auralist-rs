#[macro_use] extern crate rocket;

use serde::{Deserialize, Serialize};
mod music;
use std::collections::HashMap;
use crate::music::File;
use walkdir::WalkDir;
use murmurhash32::murmurhash3;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::SliceRandom;
use rocket::State;
use crate::music::FileHashed;
use rocket::serde::json::Json;
use dashmap::DashMap;

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

#[get("/delay/<selection>")]
async fn random_selection(
    selection: String,
    indexed_files: &State<IndexedFiles>,
    live_stats: &rocket::State<Arc<LiveStats>>,
) -> Json<FileResponse> {
    let random_hash = random_hash(selection.to_string(), indexed_files);
    generate_random_response(random_hash, indexed_files, live_stats)
}

#[get("/stream/<hash>")]
async fn stream_file(
    hash: String,
    indexed_files: &State<IndexedFiles>,
    live_stats: &rocket::State<Arc<LiveStats>>,
) -> Json<FileResponse> {
    // hash e.g 1f768ac1-6e83-4f12-a4c3-ad37f6d93844
    let sliced_hash = hash[0..36].to_string();
    // todo: work out range requests in rocker 0.5.0
    get_range(range_header, sliced_hash, live_stats)
}

#[launch]
fn rocket() -> _ {
    // random ids that need to be sought
    let plays: DashMap<String, File> = DashMap::new();

    let state = synchronous_file_scan();

    rocket::build()
        .mount("/", routes![random_selection])
        .mount("/", routes![stream_file])
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
        let random_hash = ten_random_hashes.choose(&mut rand::thread_rng()).clone().unwrap();        
        answer = random_hash.clone().clone()
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