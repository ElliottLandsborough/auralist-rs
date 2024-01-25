use rusqlite::params;
use crate::database::SQLite;
use std::path::Path;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt, Tag};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub file_name: String,
    pub file_ext: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
}

impl File {
    pub fn populate_from_path(path: &Path) -> File {
        let path_string = path.to_str().unwrap().to_string();
        let file_name = String::from(path.file_name().unwrap().to_string_lossy());

        println!("--- File: {} ---", file_name);

        let file_ext = match path.extension() {
            Some(value) => String::from(value.to_string_lossy()),
            None => String::from("")
        };

        let mut f = File {
            id: 0,
            path: path_string,
            file_name: file_name,
            file_ext: file_ext.clone(),
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            duration: 0,
        };

        if file_ext == "mp3" || file_ext == "flac" {
            f.populate_lofty();
        }

        f
    }

    pub fn save_to_database(&self) {
        let conn = SQLite::connect();

        match conn.execute(
            "INSERT INTO files (path, file_name, file_ext, title, artist, album, duration) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                self.path,
                self.file_name,
                self.file_ext,
                self.title,
                self.artist,
                self.album,
                self.duration,
            ],
        ) {
            Ok(_) => println!("Inserting into files..."),
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
            ],
        ) {
            Ok(_) => println!("Inserting into search..."),
            Err(err) => println!("Update failed: {}", err),
        }
    }

    pub fn populate_lofty(&mut self) {
        let path: &Path = Path::new(&self.path);
        let potentially_tagged_file = Probe::open(path)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let properties = potentially_tagged_file.properties();

        // Get the duration
        let duration = properties.duration();
        self.duration = duration.as_secs();
        println!("Duration (s): {}", self.duration);

        // Try to get the tag info
        match potentially_tagged_file.primary_tag() {
            Some(primary_tag) => self.fill_tags(primary_tag),
            // If the "primary" tag doesn't exist, we just grab the
            // first tag we can find. Realistically, a tag reader would likely
            // iterate through the tags to find a suitable one.
            None => {
                match potentially_tagged_file.first_tag() {
                    Some(next_tag) => self.fill_tags(next_tag),
                    None => ()
                }
            },
        };
    }

    pub fn fill_tags(&mut self, tag: &Tag) {
        println!("--- Tag Information ---");
        println!("Title: {}", tag.title().as_deref().unwrap_or(""));
        println!("Artist: {}", tag.artist().as_deref().unwrap_or(""));
        println!("Album: {}", tag.album().as_deref().unwrap_or(""));
        self.title = tag.title().as_deref().unwrap_or("").to_string();
        self.artist = tag.artist().as_deref().unwrap_or("").to_string();
        self.album = tag.title().as_deref().unwrap_or("").to_string();
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
