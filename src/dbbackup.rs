use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use std::fs::File as FsFile;
use std::io::BufReader;

pub struct BackupFile {
    pub path: String,
}

impl BackupFile {
    pub fn new(path: &str) -> BackupFile {
        BackupFile {
            path: path.to_string(),
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
}
