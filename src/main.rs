use std::io::{self, stdout, ErrorKind, Read, Write};
use std::time::{Duration, Instant};
use std::process::Command;
use env_logger;
use log::{info, warn};
//use console::{Key, Term};
use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, style, terminal};
use crossterm::style::*;


fn prompt() -> io::Result<()> {
    write!(stdout(), "> ").unwrap();
    io::stdout().flush().unwrap();
    Ok(())
}


struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        ShellOutput::clear_screen().expect("Error");
    }
}


struct ShellReader;

impl ShellReader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}


struct ShellOutput {}

impl ShellOutput {
    fn new() -> Self {
        Self {}
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
}

struct Shell {
    reader: ShellReader,
    out: ShellOutput,
    cmd_in: String,
    cmd_history: Vec<String>,
    cmd_idx: usize,
}

impl Shell {
    fn new() -> Self {
        Self {
            reader: ShellReader,
            out: ShellOutput::new(),
            cmd_in: String::new(),
            cmd_history: Vec::new(),
            cmd_idx: 0,
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } |
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                return Ok(false);
            }

            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                println!("cmd_idx = {}", self.cmd_idx);
                println!("History:");
                println!("{:?}", self.cmd_history);
            }

            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                write!(stdout(), "\r\n").unwrap();
                if self.cmd_in.is_empty() {
                    prompt().unwrap();
                    io::stdout().flush()?;
                    return Ok(true);
                }
                info!("Running: {}", self.cmd_in);
                let out = Command::new("sh")
                    .arg("-c")
                    .arg(&self.cmd_in)
                    .output()
                    .expect("failed to execute process");
                info!("Output:");
                for line in String::from_utf8(out.stdout).unwrap().lines() {
                    write!(stdout(), "{}\r\n", line).unwrap();
                }
                write!(stdout(), "\r\n").unwrap();
                io::stdout().flush()?;
                self.cmd_history.push(self.cmd_in.clone());
                self.cmd_in.clear();
                prompt().unwrap();
            }

            KeyEvent {
                code: code @ KeyCode::Char(..),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => {
                self.cmd_in.push(match code {
                    KeyCode::Char(c) => c,
                    _ => unreachable!(),
                });
                queue!(
                    stdout(),
                    terminal::Clear(ClearType::CurrentLine),
                    cursor::MoveToColumn(0),
                ).unwrap();
                prompt().unwrap();
                print!("{}", self.cmd_in);
                io::stdout().flush().unwrap();
            }

            _ => {
                warn!("Unimplemented key.");
                todo!();
            }

        }
        
        return Ok(true);
    }

    fn run(&mut self) -> io::Result<bool> {
        self.process_keypress()
    }
}


fn main() -> io::Result<()> {
    let _clean_up = CleanUp;
    env_logger::init();
    terminal::enable_raw_mode()?;
    let mut shell = Shell::new();
    prompt().unwrap();
    while shell.run()? {}
    Ok(())
}
