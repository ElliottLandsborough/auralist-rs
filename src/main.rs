use std::{env};
use rusqlite::{params, Connection, Result as SQLiteResult};
use walkdir::WalkDir;
use std::path::Path;
use ini::Ini;

#[derive(Debug)]
struct File {
    id: i32,
    path: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if (args.len()) > 1 {
        let command = &args[1];

        if command == "init" {
            init();
            return;
        }

        if command == "index" {
            index();
            return;
        }
    }

    println!("Please choose a command e.g 'init' or 'index'");
}

fn init() {
    let configPath = "conf.ini";

    if Path::new(configPath).exists() {
        println!("Could not create `conf.ini` as it already exists.");

        return;
    }

    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("encoding", "utf-8");
    conf.with_section(Some("Files"))
        .set("directory_to_index", "~/Music")
        .set("extensions_to_index", "*");
    conf.write_to_file(configPath).unwrap();
}

fn index() {
    let conf = Ini::load_from_file("conf.ini").unwrap();

    let section = conf.section(Some("Files")).unwrap();
    let directory = section.get("directory_to_index").unwrap();

    if !Path::new(directory).exists() {
        println!("Directory set in `conf.ini` missing: `{:?}`", directory);

        return;
    }

    get_files(String::from(directory));
}

fn get_files(directory: std::string::String) -> Result<i32, walkdir::Error> {
    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        println!("{}", entry.path().display());

        let full_path = entry.path().to_str().unwrap();

        index_file(String::from(full_path));
    }

    Ok(0)
}

fn index_file(path: std::string::String) -> SQLiteResult<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE file (
                  id              INTEGER PRIMARY KEY,
                  path            TEXT NOT NULL,
                  )",
        params![],
    )?;
    let f = File {
        id: 0,
        path: path,
    };
    conn.execute(
        "INSERT INTO file (path) VALUES (?1)",
        params![f.path],
    )?;

    let mut stmt = conn.prepare("SELECT id, path FROM file")?;
    let file_iter = stmt.query_map(params![], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
        })
    })?;

    println!("Found file {:?}", "RESULT:");

    for file in file_iter {
        println!("Found file {:?}", file.unwrap());
    }

    Ok(())
}
