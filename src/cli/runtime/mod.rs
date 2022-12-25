use std::collections::{HashMap, LinkedList};
use std::fs::read;
use std::io;
use std::io::{Read, Write};
use std::panic::resume_unwind;
use std::str::from_utf8;
use clap::command;
use crate::error::Result;
use console::{Key, Term};

pub mod confirm;
pub mod util;
pub mod command;


pub enum ResultCode {
    WrongArguments,
    Other(i32),
}

pub trait CommandProcessor<T> where T: Sized {
    fn get_command(&self) -> Vec<&'static str>;
    fn new() -> Self where Self: Sized;
    fn process_command(&self, ctx: &mut T, cmd_context: &CommandProcessorContext<T>, command: &str, arguments: &Vec<String>) -> Result<()>;
    fn get_hint(&self, ctx: &mut T, arguments: &Vec<String>) -> Option<String>;

    /// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
    fn box_clone(&self) -> Box<dyn CommandProcessor<T>>;
}

pub struct CommandProcessorContext<T> {
    commands: HashMap<String, Box<dyn CommandProcessor<T>>>,
    history: Vec<String>,
}

impl<T> CommandProcessorContext<T> {
    const MAX_HISTORY: usize = 256;

    pub fn new() -> Self {
        CommandProcessorContext {
            commands: HashMap::new(),
            history: Vec::with_capacity(Self::MAX_HISTORY),
        }
    }

    pub fn register_command_processor(&mut self, command: &str, processor: Box<dyn CommandProcessor<T>>) {
        self.commands.insert(command.to_string(), processor);
    }

    pub fn get_all_command_processors(&self) -> &HashMap<String, Box<dyn CommandProcessor<T>>> {
        &self.commands
    }

    pub fn get_processor_by_command(&self, command: &str) -> Option<&Box<dyn CommandProcessor<T>>> {
        self.commands.get(command)
    }

    pub fn get_command_hint(&self, command_part: &str) -> Option<String> {
        let mut got: Vec<&String> = self.commands.keys()
            .filter(|s| s.starts_with(command_part)).take(2).collect();

        match got.len() {
            1 => Some(got.remove(0).clone()),
            _ => None,
        }
    }

    pub fn line_to_args(&self, line: &str) -> Option<(String, Vec<String>)> {
        let mut all_args: Vec<String> = line.split(" ").map(|v| v.to_string()).collect();
        if all_args.is_empty() {
            return None;
        }
        let command: String = all_args.remove(0);
        return Some((command, all_args));
    }

    pub fn lines_processing(&mut self, ctx: &mut T, term: &mut Term) -> Result<bool> {
        term.write_str("> ")?;
        term.show_cursor()?;

        let mut result: String = String::with_capacity(128_usize);
        let mut hint: String = String::with_capacity(128_usize);

        // TODO: encapsulate somehow
        let mut key: Key;
        let mut cursor_position: usize = 0_usize;
        let mut history_position: usize = self.history.len();

        loop {
            key = term.read_key()?;

            match key {
                Key::ArrowLeft => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                    }
                }
                Key::ArrowRight => {
                    if cursor_position < result.len() {
                        cursor_position += 1;
                    }
                }
                Key::ArrowUp => if history_position > 0 {
                    history_position -= 1;
                    result = self.history[history_position].clone();
                    cursor_position = result.len();
                    hint = "".to_string();
                },
                Key::ArrowDown => if history_position < self.history.len() {
                    history_position += 1;
                    if history_position == self.history.len() {
                        result = "".to_string();
                    } else {
                        result = self.history[history_position].clone();
                    }
                    cursor_position = result.len();
                    hint = "".to_string();
                }
                Key::Enter => break,
                Key::Escape => {}
                Key::Backspace => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                        result.remove(cursor_position);
                    }
                }
                Key::Home => {
                    if cursor_position > 0 {
                        cursor_position = 0;
                    }
                }
                Key::End => {
                    if cursor_position < result.len() {
                        cursor_position = result.len() - 1;
                    }
                }
                Key::Tab => {
                    match self.line_to_args(&result[0..cursor_position]) {
                        None => {}
                        Some((cmd, args)) =>
                            match self.get_processor_by_command(cmd.as_str()) {
                                None => match self.get_command_hint(cmd.as_str()) {
                                    None => {}
                                    Some(cmd_hint) => {
                                        let x = result.split(" ").skip(1).map(|s| s.to_string()).collect::<Vec<String>>().join(" ");
                                        cursor_position = cmd_hint.len();
                                        result = cmd_hint + " " + x.as_ref();
                                    }
                                },
                                Some(processor) =>
                                    match processor.get_hint(ctx, &args) {
                                        None => { hint.clear() }
                                        Some(s) => { hint = s }
                                    }
                            }
                    };
                }
                Key::BackTab => {}
                Key::Alt => {}
                Key::Del => {
                    if !result.is_empty() {
                        result.remove(cursor_position);
                    }
                }
                Key::Shift => {}
                Key::Insert => {}
                Key::PageUp => {}
                Key::PageDown => {}
                Key::Char(c) => {
                    if result.len() == cursor_position {
                        result.push(c);
                    } else {
                        result.insert(cursor_position, c);
                    }
                    cursor_position += 1;
                }
                _ => {}
            }

            if !hint.is_empty() {
                term.move_cursor_down(1)?;
                term.clear_line()?;
                term.clear_last_lines(1)?;
            } else {
                term.clear_line()?;
            }
            term.write_fmt(format_args!("> {}", result))?;
            if !hint.is_empty() {
                term.write_fmt(format_args!("\n{}", hint))?;
                term.move_cursor_left(hint.len())?;
                term.move_cursor_up(1)?;
            }
            term.move_cursor_left(2 + result.len())?;
            term.move_cursor_right(2 + cursor_position)?;
        }

        term.write_line("")?;
        result = result.trim().to_string();
        self.history.push(result.clone());

        history_position = self.history.len();

        match self.line_to_args(result.as_str()) {
            None => {
                // Empty command is OK
                Ok(true)
            }
            Some((cmd, args)) => match self.get_processor_by_command(cmd.as_str()) {
                None => {
                    term.write_fmt(format_args!("Command {} not found\n", cmd))?;
                    Ok(false)
                }
                Some(p) => {
                    p.process_command(ctx, &self, cmd.as_str(), &args)?;
                    Ok(true)
                }
            }
        }
    }
}
