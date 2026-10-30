use crate::db::{Command, FrecliError};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyModifiers, read},
    execute, queue,
    style::Print,
    terminal::{self, ClearType::FromCursorDown, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    fs,
    io::{Write, stdout},
};

struct RawModeGuard;
impl RawModeGuard {
    fn new() -> Result<Self, FrecliError> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}
pub fn run_wrapper(commands: Vec<Command>) -> Result<(), FrecliError> {
    if commands.is_empty() {
        println!("No commands logged yet.");
        return Ok(());
    }
    let _guard = RawModeGuard::new()?;
    run(commands)
}

pub fn run(commands: Vec<Command>) -> Result<(), FrecliError> {
    if fs::exists("/tmp/frecli-rs_choice")? {
        fs::remove_file("/tmp/frecli-rs_choice")?;
    }
    let mut stdout = stdout();
    let mut selected: usize = 0;
    loop {
        queue!(stdout, cursor::MoveToColumn(0))?;
        draw_table(&mut stdout, &commands, selected)?;
        stdout.flush()?;
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Esc => break,
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < commands.len() {
                        selected += 1;
                    }
                }
                KeyCode::Char(c @ '0'..='9') => {
                    if let Some(index) = c.to_digit(10) {
                        if index < commands.len() as u32 {
                            selected = index as usize;
                        }
                    }
                }
                KeyCode::Enter => {
                    let cmd = &commands[selected].text;
                    fs::write("/tmp/frecli-rs_choice", format!("RUN:{}", cmd))?;
                    break;
                }
                KeyCode::Char('p') => {
                    let cmd = &commands[selected].text;
                    fs::write("/tmp/frecli-rs_choice", format!("PASTE:{}", cmd))?;
                    break;
                }
                _ => (),
            },
            _ => (),
        }
    }
    Ok(())
}

fn draw_table(
    stdout: &mut impl Write,
    commands: &[Command],
    selected: usize,
) -> Result<(), FrecliError> {
    let key_width = 5;
    let cmd_width = 40;

    queue!(stdout, cursor::MoveTo(0, 0))?;
    queue!(stdout, terminal::Clear(FromCursorDown))?;

    // header
    queue!(
        stdout,
        Print(format!(
            " {:<key_width$} │ {:<cmd_width$}\r\n",
            "Key", "Command"
        ))
    )?;
    queue!(
        stdout,
        Print(format!(
            "{}┼{}\r\n",
            "─".repeat(key_width + 2),
            "─".repeat(cmd_width + 2)
        ))
    )?;

    // rows
    for (i, cmd) in commands.iter().enumerate() {
        let line = format!(" {:<key_width$} │ {:<cmd_width$}\r\n", i, cmd.text);
        if i == selected {
            queue!(stdout, Print(format!("\x1b[32m{}\x1b[0m", line)))?;
        } else {
            queue!(stdout, Print(line))?;
        }
    }
    // hint line
    queue!(
        stdout,
        Print("\r\n [enter] run   [p] paste   [esc] cancel\r\n")
    )?;

    stdout.flush()?;
    Ok(())
}
