use tabled::settings::Style;
use tabled::{self, Table};
use crate::db::Command;

pub fn display_commands(commands: Vec<Command>){
    let mut table = Table::new(commands);
    table.with(Style::modern());
    println!("\x1b[32m{}\x1b[0m", table);
}
