use crate::cli::runtime::key::OneLineProcessingContext;
use crate::cli::runtime::vec_limit::VecLimited;
use crate::error::Result;
use console::Term;
use std::collections::HashMap;
use std::io::Write;
use std::string::ToString;

pub mod command;
pub mod confirm;
pub mod key;
pub mod vec_limit;

pub enum ResultCode {
    WrongArguments,
    Other(i32),
}

pub enum HintOption<T> {
    None,
    Line(T),
    Exact(T, usize),
}

pub trait CommandProcessor<T>
where
    T: Sized,
{
    fn get_args_help(&self) -> &'static str;

    fn get_command(&self) -> Vec<&'static str>;
    fn new() -> Self
    where
        Self: Sized;
    fn process_command(
        &self,
        ctx: &mut T,
        cmd_context: &CommandProcessorContext<T>,
        command: &str,
        arguments: &[String],
    ) -> Result<()>;
    fn get_hint(&self, ctx: &mut T, arguments: &[String])
        -> HintOption<String>;

    /// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
    fn box_clone(&self) -> Box<dyn CommandProcessor<T>>;
}

pub struct CommandProcessorContext<T> {
    commands: HashMap<String, Box<dyn CommandProcessor<T>>>,
    history: VecLimited<String>,
}

impl<T> Default for CommandProcessorContext<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CommandProcessorContext<T> {
    const MAX_HISTORY: usize = 256;

    pub fn new() -> Self {
        CommandProcessorContext {
            commands: HashMap::new(),
            history: VecLimited::with_capacity(
                Self::MAX_HISTORY,
                Self::MAX_HISTORY,
            ),
        }
    }

    pub fn register_command_processor(
        &mut self,
        command: &str,
        processor: Box<dyn CommandProcessor<T>>,
    ) {
        self.commands
            .insert(command.to_string(), processor);
    }

    pub fn get_all_command_processors(
        &self,
    ) -> &HashMap<String, Box<dyn CommandProcessor<T>>> {
        &self.commands
    }

    pub fn get_processor_by_command(
        &self,
        command: &str,
    ) -> Option<&dyn CommandProcessor<T>> {
        match self.commands.get(command) {
            None => None,
            Some(v) => Some(v.as_ref()),
        }
    }

    pub fn get_command_hint(&self, command_part: &str) -> Option<String> {
        let mut got: Vec<&String> = self
            .commands
            .keys()
            .filter(|s| s.starts_with(command_part))
            .take(2)
            .collect();

        match got.len() {
            1 => Some(got.remove(0).clone()),
            _ => None,
        }
    }

    pub fn line_to_args(&self, line: &str) -> Option<(String, Vec<String>)> {
        let mut all_args: Vec<String> = line
            .split(' ')
            .map(|v| v.to_string())
            .collect();
        if all_args.is_empty() {
            return None;
        }
        let command: String = all_args.remove(0);
        Some((command, all_args))
    }

    pub fn lines_processing(
        &mut self,
        ctx: &mut T,
        term: &mut Term,
    ) -> Result<bool> {
        term.write_str("> ")?;
        term.show_cursor()?;

        let mut context: OneLineProcessingContext =
            OneLineProcessingContext::new(self.history.len());

        loop {
            let key = term.read_key()?;
            let (_, term_w) = {
                let (h, w) = term.size();
                (h as usize, w as usize)
            };
            context.process_key(ctx, self, key)?;
            if context.is_last_hint_available() {
                let lines = (context.get_last_hint().len() + term_w) / term_w;
                term.move_cursor_down(lines)?;
                term.clear_line()?;
                term.clear_last_lines(lines)?;
            } else {
                term.clear_line()?;
            }
            term.write_fmt(format_args!(
                "> {}",
                context.result_to_string()
            ))?;

            if context.is_line_processed() {
                if context.is_hint_available() {
                    let lines = (context.get_hint().len() + term_w) / term_w;
                    term.move_cursor_down(lines)?;
                    term.clear_line()?;
                    term.move_cursor_up(lines)?;
                }
                break;
            }

            if context.is_hint_available() {
                let hint = context.get_hint();
                let lines = (hint.len() + term_w) / term_w;

                term.write_fmt(format_args!("\n{}", hint))?;
                term.move_cursor_left(hint.len())?;
                term.move_cursor_up(lines)?;
            }
            term.move_cursor_left(2 + context.result.len())?;
            term.move_cursor_right(2 + context.cursor_position)?;
        }

        term.write_line("")?;
        let trimmed_string = context
            .result_to_string()
            .trim()
            .to_string();
        if !trimmed_string.is_empty() {
            self.history
                .push(context.result_to_string());
        }

        match self.line_to_args(trimmed_string.as_str()) {
            None => {
                // Empty command is OK
                Ok(true)
            }
            Some((cmd, args)) => {
                match self.get_processor_by_command(cmd.as_str()) {
                    None => {
                        term.write_fmt(format_args!(
                            "Command {} not found\n",
                            cmd
                        ))?;
                        Ok(false)
                    }
                    Some(p) => {
                        p.process_command(ctx, self, cmd.as_str(), &args)?;
                        Ok(true)
                    }
                }
            }
        }
    }
}
