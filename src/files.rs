use flate2::Compression;
use flate2::bufread::GzEncoder;
use flate2::bufread::GzDecoder;
use std::fs::File as FsFile;
use std::io::BufReader;
use crate::database::SQLite;
use rusqlite::params;


#[derive(Debug)]
pub struct File {
    pub id: i32,
    pub path: String,
    pub path_hash: String,
}

impl File {
    pub fn new_database_backup(path: &str) -> File {
        File {
            id: 0,
            path: path.to_string(),
            path_hash: "".to_string()
        }
    }

    pub fn new(path: &str, path_hash: &str) -> File {
        File {
            id: 0,
            path: path.to_string(),
            path_hash: path_hash.to_string()
        }
    }

    pub fn compress_to_gz(&self) {
        println!("Compressing `{}`...", self.path.clone());
        let f = FsFile::open(self.path.clone());
        let b = BufReader::new(f.unwrap());
        let mut gz = GzEncoder::new(b, Compression::default());
        let destination = self.path.clone() + ".gz";
        let mut f = FsFile::create(destination).expect("Unable to create file");
        std::io::copy(&mut gz, &mut f).expect("Unable to copy data");
    }
    
    pub fn decompress_from_gz(&self) {
        let source = self.path.clone() + ".gz";
        println!("Decompressing `{}`...", source);
        let f = FsFile::open(source);
        let b = BufReader::new(f.unwrap());
        let mut gz = GzDecoder::new(b);

        let mut f = FsFile::create(self.path.clone()).expect("Unable to create file");
        std::io::copy(&mut gz, &mut f).expect("Unable to copy data");
    }

    pub fn save_to_database(&self) {
        let conn = SQLite::connect();

        match conn.execute(
            "INSERT INTO file (path, path_hash) VALUES (?1, ?2)",
            params![
                self.path,
                self.path_hash,
            ],
        ) {
            Ok(_) => println!("."),
            Err(err) => println!("Update failed: {}", err),
        }
    }
}
