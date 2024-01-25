use rusqlite::params;
use crate::database::SQLite;
use std::path::Path;
extern crate taglib;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
//extern crate tree_magic;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub file_name: String,
    pub file_ext: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub time: f64,
}

impl File {
    pub fn populate_from_path(path: &Path) -> File {
        let path_string = path.to_str().unwrap().to_string();
        let file_name = String::from(path.file_name().unwrap().to_string_lossy());

        let file_ext = match path.extension() {
            Some(value) => String::from(value.to_string_lossy()),
            None => String::from("")
        };

        let mut f = File {
            id: 0,
            path: path_string,
            file_name: file_name,
            file_ext: file_ext,
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            time: 0.0,
        };

        f.populate_tags();

        f
    }

    pub fn save_to_database(&self) {
        let conn = SQLite::connect();

        match conn.execute(
            "INSERT INTO files (path, file_name, file_ext, title, artist, album) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.path,
                self.file_name,
                self.file_ext,
                self.title,
                self.artist,
                self.album,
                self.time,
            ],
        ) {
            Ok(_) => println!("."),
            Err(err) => println!("Update failed: {}", err),
        }

        match conn.execute(
            "INSERT INTO search (path, file_name, file_ext, title, artist, album) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.path,
                self.file_name,
                self.file_ext,
                self.title,
                self.artist,
                self.album,
                self.time,
            ],
        ) {
            Ok(_) => println!("."),
            Err(err) => println!("Update failed: {}", err),
        }
    }

    pub fn populate_tags(&mut self) {
        // Fallback to inferring mime type?
        let path: &Path = Path::new(&self.path);
        //let mime = tree_magic::from_filepath(path);
        //println!("Mime type: {}", mime);

        match taglib::File::new(path) {
            Ok(file) => {
                match file.tag() {
                    Ok(t) => {
                        self.title = t.title().unwrap_or_default();
                        self.artist = t.artist().unwrap_or_default();
                        self.album = t.album().unwrap_or_default();
                    },
                    Err(_) => ()
                }
                match file.audioproperties() {
                    Ok(_p) => {
                        //self.length = _p.length(); // in seconds
                    }
                    Err(e) => {
                        println!("No available audio properties for {} (error: {:?})", path.display(), e);
                    }
                }
            },
            Err(e) => {
                println!("Invalid file {} (error: {:?})", path.display(), e);
            }
        };
    }

    pub fn get_unique_id(&mut self) {
        let conn = SQLite::connect();

        let uuid = Uuid::new_v4().to_string();

        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        } as f64;

        match conn.execute(
            "INSERT INTO plays (hash, time, file) VALUES (?1, ?2, ?3)",
            params![
                uuid,
                now,
                self.id,
            ],
        ) {
            Ok(_) => (),
            Err(err) => println!("Update failed: {}", err),
        }

        self.path = uuid;

        return;
    }
}
