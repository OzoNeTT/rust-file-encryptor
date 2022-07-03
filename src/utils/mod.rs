use crate::error;

pub trait DynamicParser {
    fn new() -> Self;

    fn parse_next<'a>(&mut self, batch: &'a [u8]) -> error::Result<&'a [u8]>;

    fn ready(&self) -> bool;
}
