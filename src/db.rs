use chrono::{TimeDelta, prelude::*};
use rusqlite::{Connection, Result};
use std::{
    collections::HashSet,
    env::{home_dir, var},
    fs::{self, OpenOptions, rename},
    io::{Read, Write},
    path::PathBuf,
};
use tabled::Tabled;

#[derive(Debug, Tabled)]
pub struct Command {
    pub text: String,
    count: u32,
}

const DB_PATH: &str = "frecli/history.db";
const IGNORE_PATH: &str = "frecli/ignore";

fn db() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share").join(DB_PATH)
}
fn ignore() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config").join(IGNORE_PATH)
}

fn xdg_dir(var_name: &str, default: &str) -> PathBuf {
    var(var_name)
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().unwrap().join(default))
}

#[derive(thiserror::Error, Debug)]
pub enum FrecliError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Process Terminated by Signal")]
    ProcessKilledBySignal,
    #[error("Exit Code of a process")]
    ExitCode,
}

pub fn init_db() -> Result<(), FrecliError> {
    let db_file = db();
    let ignore_file = ignore();
    fs::create_dir_all(db_file.parent().unwrap())?;
    fs::create_dir_all(ignore_file.parent().unwrap())?;
    OpenOptions::new().create(true).write(true).open(&db_file)?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(&ignore_file)?;
    let conn = Connection::open(&db_file)?;
    conn.execute("CREATE TABLE IF NOT EXISTS commands (id INTEGER PRIMARY KEY, command TEXT, timestamp TEXT)", ())?;
    Ok(())
}

pub fn log_command(command: String) -> Result<(), FrecliError> {
    let db_file = db();
    let ignore_file = ignore();
    let set = load_ignore_list(&ignore_file)?;
    let ignored = set
        .iter()
        .any(|ignore| command == *ignore || command.starts_with(&format!("{ignore} ")));
    if ignored {
        return Ok(());
    }
    let conn = Connection::open(&db_file)?;
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO commands (command, timestamp) VALUES (?, ?)",
        (&command, &timestamp),
    )?;
    Ok(())
}

pub fn log_ignore_command(pattern: Vec<String>) -> Result<(), FrecliError> {
    let ignore_file = ignore();
    let mut writer = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ignore_file)?;
    //TODO: check if the pattern is already in the file before we add it to prevent duplicates
    for line in pattern {
        writeln!(writer, "{line}")?;
    }
    println!("Successfully added ignore command");
    Ok(())
}

pub fn get_ignored() -> Result<Vec<String>, FrecliError> {
    let list = fs::read_to_string(ignore())?;
    let mut arr: Vec<String> = Vec::new();
    for command in list.lines() {
        let command = command.trim();
        if !command.is_empty() {
            arr.push(command.to_string());
        }
    }
    Ok(arr)
}

pub fn remove_ignore_command(command: String) -> Result<(), FrecliError> {
    let ignore_file = ignore();
    let mut file = OpenOptions::new().read(true).open(&ignore_file)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let mut buffer: Vec<&str> = buffer.trim().lines().collect();
    /* deletes everything thats not the command supplied */
    let buffer_old_len = buffer.len();
    buffer.retain(|line| *line != command);
    /* if nothing was removed then we can return early */
    if buffer_old_len == buffer.len() {
        println!("Command not found, nothing to Remove");
        return Ok(());
    }

    let temp_path = ignore().with_file_name("ignore_temp");
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)?;
    for line in &buffer {
        writeln!(temp_file, "{line}")?;
    }
    rename(temp_path, ignore_file)?;
    Ok(())
}

fn load_ignore_list(ignore_list: &PathBuf) -> Result<HashSet<String>, FrecliError> {
    let list = fs::read_to_string(ignore_list)?;
    let mut set = HashSet::new();
    for command in list.lines() {
        let command = command.trim();
        if !command.is_empty() {
            set.insert(command.to_string());
        }
    }
    Ok(set)
}

pub fn prune_commands(older_than: u32) -> Result<(), FrecliError> {
    let db = db();
    let cutoff = (Utc::now() - TimeDelta::days(older_than as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let conn = Connection::open(&db)?;
    let rows = conn.execute("DELETE FROM commands WHERE timestamp < ?", (&cutoff,))?;
    println!("{} rows pruned older than {} days", rows, older_than);
    Ok(())
}

pub fn clear() -> Result<(), FrecliError> {
    let db = db();
    let conn = Connection::open(&db)?;
    let rows = conn.execute("DELETE FROM commands", ())?;
    println!("{} Rows Deleted", rows);
    Ok(())
}

pub fn get_commands(limit: u32) -> Result<Vec<Command>, FrecliError> {
    let conn = Connection::open(db())?;
    let mut query = conn.prepare(
        "SELECT command, COUNT(*) FROM commands GROUP BY command ORDER BY COUNT(*) DESC LIMIT ?",
    )?;
    let commands = query.query_map((limit,), |row| {
        Ok(Command {
            text: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let mut stats: Vec<Command> = Vec::new();
    for command in commands {
        stats.push(command?);
    }
    Ok(stats)
}
