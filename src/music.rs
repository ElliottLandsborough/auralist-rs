use rusqlite::params;
use crate::database::SQLite;
use twox_hash::xxh3;
use std::path::Path;
extern crate taglib;
use serde::{Serialize, Deserialize};
//extern crate tree_magic;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub id: i32,
    pub path: String,
    pub path_hash: String,
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}

impl File {
    pub fn populate_from_path(path: &Path) -> File {
        let path_string = path.to_str().unwrap().to_string();
        let mut f = File {
            id: 0,
            path: path_string.clone(),
            path_hash: xxh3::hash64(path_string.as_bytes()).to_string(),
            file_name: "".to_string(),
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
        };

        f.populate_tags();

        f
    }

    pub fn save_to_database(&self) {
        let conn = SQLite::connect();

        match conn.execute(
            "INSERT INTO file (path_hash, path, file_name, title, artist, album) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.path_hash,
                self.path,
                self.file_name,
                self.title,
                self.artist,
                self.album,
            ],
        ) {
            Ok(_) => println!("."),
            Err(err) => println!("Update failed: {}", err),
        }

        match conn.execute(
            "INSERT INTO file_search (path_hash, file_name, title, artist, album) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.path_hash,
                self.file_name,
                self.title,
                self.artist,
                self.album,
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
                        //self.length = _p.length();
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
}
