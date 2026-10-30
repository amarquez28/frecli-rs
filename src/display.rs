use crate::db::Command;
use tabled::settings::Style;
use tabled::{self, Table};

pub fn display_commands(commands: Vec<Command>) {
    if commands.is_empty() {
        println!("Nothing to print, list is empty");
        return;
    }
    let mut table = Table::new(commands);
    table.with(Style::modern());
    println!("\x1b[32m{}\x1b[0m", table);
}

pub fn display_ignored(ignored: Vec<String>) {
    if ignored.is_empty() {
        println!("Nothing to print, list is empty");
        return;
    }
    let mut table = Table::new(ignored);
    table.with(Style::modern());
    println!("\x1b[32m{}\x1b[0m", table);
}
