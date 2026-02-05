use std::{error, fmt};

pub mod headers;
pub mod request;
pub mod response;
#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
    RequestLine,
    StatusLine,
    Header,
}
impl error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::RequestLine => write!(f, "Invalid request line"),
            ParseError::StatusLine => write!(f, "Invalid status line"),
            ParseError::Header => write!(f, "Invalid header line"),
        }
    }
}

mod util {
    use std::error::Error;
    use std::io;
    use std::io::BufRead;

    pub fn get_string_until<T: io::Read, E: Error + Clone>(
        stream: &mut io::BufReader<T>,
        byte: u8,
        on_error: E,
    ) -> Result<String, E> {
        let mut buffer = Vec::new();
        stream
            .read_until(byte, &mut buffer)
            .map_err(|e| handle_io_error(e, on_error.clone()))?;
        buffer.pop(); //Remove whitespace from read
        String::from_utf8(buffer).or(Err(on_error))
    }

    pub fn handle_io_error<E: Error>(err: io::Error, new_err: E) -> E {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => new_err,
            _ => panic!("Unexpected IO error! : '{}'", err),
        }
    }
}
