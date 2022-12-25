use std::io;
use std::io::Write;
use console::Term;
//
// pub trait ConsoleTermExtension {
//     fn write_str(&mut self, string: &str) -> io::Result<()>;
// }
//
// impl ConsoleTermExtension for Term {
//     fn write_str(&mut self, string: &str) -> io::Result<()> {
//         self.write(string.as_bytes())?;
//         Ok(())
//     }
// }