use std::io;
use console::Term;

pub trait UserConfirm {
    fn user_confirm_default(&mut self, message: &str, default: bool) -> io::Result<bool>;

    fn user_confirm(&mut self, message: &str) -> io::Result<bool> {
        self.user_confirm_default(message, false)
    }
}

impl UserConfirm for Term {
    fn user_confirm_default(&mut self, message: &str, default: bool) -> io::Result<bool> {
        let hint = if default {"Y/n"} else {"y/N"};
        loop {
            self.write_str(format!("{}? [{}]: ", message, hint).as_str())?;
            let line = self.read_line()?;
            match line.to_lowercase().as_str() {
                "y" => return Ok(true),
                "n" => return Ok(false),
                _ => {}
            };
        }
    }
}