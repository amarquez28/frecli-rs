use rusqlite::{Connection, Result};
use tabled::Tabled;
use std::{collections::HashSet, env::home_dir, fs::{self, OpenOptions}, path::PathBuf};
use chrono::{TimeDelta, prelude::*};
#[derive(Debug)]
#[derive(Tabled)]
pub struct Command{
    pub text: String,
    count: u32,
}

const DB_PATH: &str = ".local/share/frecli/history.db";
const IGNORE_PATH: &str = ".config/frecli/ignore";

fn db() -> PathBuf{
   home_dir().unwrap().join(DB_PATH)
}
fn ignore() -> PathBuf{
    home_dir().unwrap().join(IGNORE_PATH)
}

#[derive(thiserror::Error, Debug)]
pub enum FrecliError{
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

}

pub fn init_db() ->  Result<(), FrecliError> {
    let db_file = db();
    let ignore_file = ignore();
    fs::create_dir_all(db_file.parent().unwrap())?;
    fs::create_dir_all(ignore_file.parent().unwrap())?;
    OpenOptions::new().create(true).write(true).open(&db_file)?;
    OpenOptions::new().create(true).write(true).open(&ignore_file)?;
    let conn = Connection::open(&db_file)?;
    conn.execute("CREATE TABLE IF NOT EXISTS commands (id INTEGER PRIMARY KEY, command TEXT, timestamp TEXT)", ())?;
    Ok(())
}

pub fn log_command(command: String) -> Result<(), FrecliError>{
    let db_file = db();
    let ignore_file = ignore();
    let set = load_ignore_list(&ignore_file)?;
    let ignored = set.iter().any(|ignore| {
        command == *ignore || command.starts_with(&format!("{ignore} "))
    });
    if ignored {
        return Ok(());
    }
    let conn = Connection::open(&db_file)?;
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute("INSERT INTO commands (command, timestamp) VALUES (?, ?)", (&command, &timestamp))?;
    Ok(())
}

fn load_ignore_list(ignore_list: &PathBuf) -> Result<HashSet<String>, FrecliError>{
    let list = fs::read_to_string(ignore_list)?;
    let mut set = HashSet::new();
    for command in list.lines(){
        let command = command.trim();
        if !command.is_empty(){
            set.insert(command.to_string());
        }
    }
    Ok(set)
}
pub fn prune_commands(older_than: u32) -> Result<(), FrecliError>{
    let db = db();
    let cutoff = (Utc::now() - TimeDelta::days(older_than as i64)).format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = Connection::open(&db)?;
    let rows = conn.execute("DELETE FROM commands WHERE timestamp < ?", (&cutoff,))?;
    println!("{} rows pruned older than {} days", rows, older_than);
    Ok(())
}

pub fn get_commands(limit: u32) -> Result<Vec<Command>, FrecliError>{
    let conn = Connection::open(db())?;
    let mut query = conn.prepare("SELECT command, COUNT(*) FROM commands GROUP BY command ORDER BY COUNT(*) DESC LIMIT ?",)?;
    let commands = query.query_map((limit,), |row| {
        Ok(Command{
            text: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let mut stats: Vec<Command> = Vec::new();
    for command in commands{
        stats.push(command?);
    }
    Ok(stats)
}
