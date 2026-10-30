use clap::Parser;
use std::error::Error;
mod db;
mod display;
mod run;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum CLA {
    #[command(about = "Log the command to the DB")]
    Log { command: String },
    #[command(about = "Show top n commands, default is 10")]
    Stats {
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    #[command(about = "Delete commands older than threshold, default is 90 days")]
    Prune {
        #[arg(long, default_value_t = 90)]
        older_than: u32,
    },
    #[command(about = "Choose from a list to run a command")]
    Run {
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    #[command(about = "Add to ignore list or print the current ignore list")]
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
}

#[derive(Parser, Debug)]
enum IgnoreAction {
    #[command(about = "Add a command pattern to the ignore list")]
    Add {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        pattern: Vec<String>,
    },
    #[command(about = "Print the current ignore list")]
    List,
    #[command(about = "Remove a command from the ignore list")]
    Remove { command: String },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CLA::parse();
    db::init_db()?;

    match args {
        CLA::Log { command } => {
            db::log_command(command)?;
            Ok(())
        }
        CLA::Prune { older_than } => {
            db::prune_commands(older_than)?;
            Ok(())
        }
        CLA::Run { limit } => {
            run::run_wrapper(db::get_commands(limit)?)?;
            Ok(())
        }
        CLA::Stats { limit } => {
            display::display_commands(db::get_commands(limit)?);
            Ok(())
        }
        CLA::Ignore { action } => {
            match action {
                IgnoreAction::Add { pattern } => db::log_ignore_command(pattern)?,
                IgnoreAction::List => display::display_ignored(db::get_ignored()?),
                IgnoreAction::Remove { command } => db::remove_ignore_command(command)?,
            }
            Ok(())
        }
    }
}
