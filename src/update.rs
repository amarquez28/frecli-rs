use crate::db::FrecliError;
use std::{
    io::Write,
    process::{Command, Stdio},
};
const SCRIPT: &str = include_str!("../scripts/update.sh"); //bake the text into the executable
pub fn handle_update(force: bool) -> Result<(), FrecliError> {
    let mut cmd = Command::new("sh");
    cmd.arg("-s"); //read the script from stdin
    if force {
        cmd.arg("--").arg("--force");
    }
    let mut child = cmd.stdin(Stdio::piped()).spawn()?;

    child.stdin.take().unwrap().write_all(SCRIPT.as_bytes())?;

    let status = child.wait()?;

    if status.success() {
        Ok(())
    } else {
        match status.code() {
            Some(code) => {
                println!("Exited with status code: {}", code);
                Err(FrecliError::ExitCode)
            }
            None => {
                println!("Process killed by signal");
                Err(FrecliError::ProcessKilledBySignal)
            }
        }
    }
}
