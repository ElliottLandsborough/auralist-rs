use std::{env};
use rusqlite::{params, Connection, Result as RuResult};
use walkdir::WalkDir;
use std::path::Path;

#[derive(Debug)]
struct File {
    id: i32,
    path: String,
}

fn main() -> RuResult<()> {
    let args: Vec<String> = env::args().collect();

    if (args.len()) > 1 {
        let command = &args[1];

        if command == "index" {
            return index()
        }
    }

    println!("Please choose a command e.g 'index'");

    Ok(())
}

fn index() -> RuResult<()> {
    let directory = "/Users/elliottlandsborough/Music/TestFiles";

    if !Path::new(directory).exists() {
        panic!("Directory missing: {:?}", directory)
    }

    get_files(String::from(directory));

    Ok(())
}

fn get_files(directory: std::string::String) -> Result<i32, walkdir::Error> {
    for entry in WalkDir::new(directory) {
        let entry = match entry {
            Ok(file) => file,
            Err(error) => panic!("Problem with file: {:?}", error),
        };

        println!("{}", entry.path().display());

        let fullPath = entry.path().to_str().unwrap();

        index_file(String::from(fullPath));
    }

    Ok(0)
}

fn index_file(path: std::string::String) -> RuResult<()> {
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
