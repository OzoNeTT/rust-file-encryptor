use crate::cli::runtime::vec_limit::VecLimited;
use crate::cli::runtime::CommandProcessorContext;
use crate::error::Result;
use console::Key;

pub struct OneLineProcessingContext {
    pub result: Vec<char>,
    hint: VecLimited<String>,
    pub cursor_position: usize,
    history_position: usize,
    line_processed: bool,
}

trait KeyProcessor<T> {
    fn unknown(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn unknown_esc_seq(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
        seq: Vec<char>,
    ) -> Result<bool>;
    fn arrow_left(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn arrow_right(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn arrow_up(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn arrow_down(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn enter(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn escape(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn backspace(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn home(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn end(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn tab(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn back_tab(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn alt(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn del(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn shift(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn insert(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn page_up(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn page_down(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool>;
    fn char(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
        c: char,
    ) -> Result<bool>;
}

impl OneLineProcessingContext {
    pub fn new(history_position: usize) -> Self {
        let mut hint_vector = VecLimited::with_capacity(2_usize, 2_usize);
        hint_vector.push("".to_string());

        Self {
            result: Vec::with_capacity(128_usize),
            hint: hint_vector,
            cursor_position: 0_usize,
            history_position,
            line_processed: false,
        }
    }

    pub fn is_hint_available(&self) -> bool {
        self.hint.len() > 1 && !self.hint[1].is_empty()
    }

    pub fn is_last_hint_available(&self) -> bool {
        !self.get_last_hint().is_empty()
    }

    pub fn get_hint(&self) -> String {
        if !self.is_hint_available() {
            "".to_string()
        } else {
            self.hint[1].to_string()
        }
    }

    pub fn get_last_hint(&self) -> &String {
        &self.hint[0]
    }

    pub fn result_to_string(&self) -> String {
        self.result.iter().collect()
    }

    pub fn is_line_processed(&self) -> bool {
        self.line_processed
    }

    pub fn process_key<T>(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
        key: Key,
    ) -> Result<bool> {
        match key {
            Key::Unknown => self.unknown(full_ctx, ctx),
            Key::UnknownEscSeq(seq) => self.unknown_esc_seq(full_ctx, ctx, seq),
            Key::ArrowLeft => self.arrow_left(full_ctx, ctx),
            Key::ArrowRight => self.arrow_right(full_ctx, ctx),
            Key::ArrowUp => self.arrow_up(full_ctx, ctx),
            Key::ArrowDown => self.arrow_down(full_ctx, ctx),
            Key::Enter => self.enter(full_ctx, ctx),
            Key::Escape => self.escape(full_ctx, ctx),
            Key::Backspace => self.backspace(full_ctx, ctx),
            Key::Home => self.home(full_ctx, ctx),
            Key::End => self.end(full_ctx, ctx),
            Key::Tab => self.tab(full_ctx, ctx),
            Key::BackTab => self.back_tab(full_ctx, ctx),
            Key::Alt => self.alt(full_ctx, ctx),
            Key::Del => self.del(full_ctx, ctx),
            Key::Shift => self.shift(full_ctx, ctx),
            Key::Insert => self.insert(full_ctx, ctx),
            Key::PageUp => self.page_up(full_ctx, ctx),
            Key::PageDown => self.page_down(full_ctx, ctx),
            Key::Char(c) => self.char(full_ctx, ctx, c),
            _ => self.unknown(full_ctx, ctx),
        }
    }

    fn tab_hint<T>(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
        (cmd, args): (String, Vec<String>),
    ) -> Result<bool> {
        match ctx.get_processor_by_command(cmd.as_str()) {
            None => match ctx.get_command_hint(cmd.as_str()) {
                None => Ok(true),
                Some(cmd_hint) => {
                    let x = self
                        .result_to_string()
                        .split(' ')
                        .skip(1)
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    self.cursor_position = cmd_hint.len();
                    let result_string = cmd_hint + " " + x.as_ref();
                    self.result = result_string.chars().collect();
                    Ok(true)
                }
            },
            Some(processor) => match processor.get_hint(full_ctx, &args) {
                None => {
                    self.hint.push("".to_string());
                    Ok(true)
                }
                Some(s) => {
                    self.hint.push(s);
                    Ok(true)
                }
            },
        }
    }
}

impl<T> KeyProcessor<T> for OneLineProcessingContext {
    fn unknown(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        // TODO
        log::debug!(target: "cli/runtime/key OneLineProcessingContext", "unknown");
        Ok(true)
    }

    fn unknown_esc_seq(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
        _seq: Vec<char>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn arrow_left(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
        Ok(true)
    }

    fn arrow_right(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if self.cursor_position < self.result.len() {
            self.cursor_position += 1;
        }
        Ok(true)
    }

    fn arrow_up(
        &mut self,
        _full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if self.history_position > 0 {
            self.history_position -= 1;
            self.result = ctx.history[self.history_position]
                .clone()
                .chars()
                .collect();
            self.cursor_position = self.result.len();
            self.hint.push("".to_string());
        }
        Ok(true)
    }

    fn arrow_down(
        &mut self,
        _full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if self.history_position < ctx.history.len() {
            self.history_position += 1;
            if self.history_position == ctx.history.len() {
                self.result = vec![];
            } else {
                self.result = ctx.history[self.history_position]
                    .clone()
                    .chars()
                    .collect();
            }
            self.cursor_position = self.result.len();
            self.hint.push("".to_string());
        }
        Ok(true)
    }

    fn enter(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        self.line_processed = true;
        Ok(true)
    }

    fn escape(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn backspace(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.result.remove(self.cursor_position);
        }
        Ok(true)
    }

    fn home(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        self.cursor_position = 0;
        Ok(true)
    }

    fn end(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        self.cursor_position = self.result.len();
        Ok(true)
    }

    fn tab(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        let option = ctx.line_to_args(self.result_to_string().as_str());
        match option {
            None => Ok(true),
            Some(tuple) => self.tab_hint(full_ctx, ctx, tuple),
        }
    }

    fn back_tab(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn alt(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn del(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        if !self.result.is_empty() {
            self.result.remove(self.cursor_position);
        }
        Ok(true)
    }

    fn shift(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn insert(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn page_up(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn page_down(
        &mut self,
        _full_ctx: &mut T,
        _ctx: &mut CommandProcessorContext<T>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn char(
        &mut self,
        full_ctx: &mut T,
        ctx: &mut CommandProcessorContext<T>,
        c: char,
    ) -> Result<bool> {
        if c == '\r' || c == '\n' {
            return self.enter(full_ctx, ctx);
        }
        if c == '\t' {
            return self.tab(full_ctx, ctx);
        }

        if self.result.len() == self.cursor_position {
            self.result.push(c);
        } else {
            self.result
                .insert(self.cursor_position, c);
        }
        self.cursor_position += 1;
        Ok(true)
    }
}
