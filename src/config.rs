use std::path::Path;
use ini::Ini;

pub struct ConfigFile {
    path: String
}

impl ConfigFile {
    pub fn new() -> ConfigFile {
        ConfigFile {
            path: String::from("./conf.ini")
        }
    }

    pub fn create(&self) {
        if Path::new(&self.path.to_string()).exists() {
            println!("Could not create `{}` as it already exists.", self.path);
    
            return;
        }
    
        let mut conf = Ini::new();

        conf.with_section(None::<String>)
            .set("encoding", "utf-8");
        conf.with_section(Some("Indexer"))
            .set("directory_to_index", "./files")
            .set("directory_exclusions", "./exclusions.txt")
            .set("extensions_to_index", "flac,wav,mp3");

        conf.with_section(Some("System"))
            .set("db_file", "./auralist.sqlite3");
        
        let w = conf.write_to_file(&self.path.to_string());

        match w {
            Ok(w) => w,
            Err(err) => panic!("Could not write file `{}`", err)
        }
    }
}

pub struct Settings;

impl Settings {
    pub fn get(category: & str, item: & str) -> String {
        let conf = match Ini::load_from_file(ConfigFile::new().path) {
            Ok(conf) => conf,
            Err(err) => panic!("Could not open file `{}`", err)
        };

        String::from(conf.section(Some(category)).unwrap().get(item).unwrap())
    }
}