use std::error::Error;
use std::fs::File;
use std::io::BufReader;

mod api;
mod cli;

use crate::api::api::ToDoList;
use crate::cli::cli::parse;

fn main() -> Result<(), Box<dyn Error>> {
    let mut list: ToDoList<String> = match File::open(api::api::BACKUP_FILE) {
        Ok(file) => {
            // file exists - deserialize and go with existing list
            let file = BufReader::new(file);
            serde_json::from_reader(file)?
        }
        Err(_) => {
            // file does not exist - make a new list
            ToDoList::new(env!("USER").to_string())
        }
    };

    match parse() {
        Some(inst) => list.run(inst),
        None => panic!("Arguments could not be parsed"),
    }
    Ok(())
}
