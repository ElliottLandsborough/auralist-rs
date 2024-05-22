use lofty::prelude::{Accessor, AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Tag;
use murmurhash32::murmurhash3;
use serde::{Deserialize, Serialize};
use std::fs::File as StdFsFile;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub id: u32,
    pub path: String,
    pub file_name: String,
    pub file_ext: String,
    pub file_size: u64,
    pub file_modified: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub indexed_at: u64,
    pub accessed_at: u64,
    pub parse_fail: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileHashed {
    pub path: String,
    pub ext: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub file: String,
}

impl File {
    pub fn to_response(&mut self) -> FileHashed {
        FileHashed {
            path: self.get_unique_id(),
            ext: self.file_ext.clone(),
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            file: self.file_name.clone(),
        }
    }

    pub fn new_empty_file_from_path(path: &Path) -> File {
        println!("Creating a new empty file struct based on path...");
        let path_string = path.to_str().unwrap().to_string();
        let file_name = String::from(path.file_name().unwrap().to_string_lossy());

        println!("--- File: {} ---", file_name);

        let file_ext = match path.extension() {
            Some(value) => String::from(value.to_string_lossy()),
            None => String::from(""),
        };

        File {
            id: murmurhash3(path_string.as_bytes()),
            path: path_string,
            file_name: file_name,
            file_ext: file_ext.clone(),
            file_size: 0,
            file_modified: 0,
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            duration: 0,
            indexed_at: 0,
            accessed_at: 0,
            parse_fail: false,
        }
    }

    // Gets basic file info - no tags
    pub fn populate_from_path(&mut self) {
        println!("Run populate_from_path()...");
        if self.path == "" {
            panic!("Can't populate from path when file struct has no path!");
        }

        if self.id == 0 {
            panic!("Can't populate from path when file struct has no id!");
        }

        let file = StdFsFile::open(&self.path).unwrap();

        println!("--- System time... ---");
        let modified_system_time = match file.metadata().unwrap().modified() {
            Ok(modified_system_time) => modified_system_time,
            Err(_) => SystemTime::now(),
        };

        println!("--- Modified time... ---");
        let file_mtime = modified_system_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!("--- Populating file struct with basic info (index)... ---");

        self.file_size = file.metadata().unwrap().len();
        self.file_modified = file_mtime;
        self.title = "".to_string();
        self.artist = "".to_string();
        self.album = "".to_string();
        self.duration = 0;
        self.indexed_at = 0;
        self.accessed_at = 0;
        self.parse_fail = false;
        println!("END populate_from_path()...");
    }

    pub fn populate_lofty(&mut self) {
        let path: &Path = Path::new(&self.path);
        let potentially_tagged_file =
            match Probe::open(path).expect("ERROR: Bad path provided!").read() {
                Ok(file) => file,
                Err(error) => {
                    self.parse_fail = true;
                    println!(
                        "Error: Can't parse file `{}`. Error: {}",
                        self.path,
                        error.to_string()
                    );
                    return;
                }
            };

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
            None => match potentially_tagged_file.first_tag() {
                Some(next_tag) => self.fill_tags(next_tag),
                None => (),
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
        self.album = tag.album().as_deref().unwrap_or("").to_string();
    }

    pub fn get_unique_id(&mut self) -> String {
        let uuid = Uuid::new_v4().to_string();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.accessed_at = now;
        self.path = uuid.clone();

        uuid
    }
}
