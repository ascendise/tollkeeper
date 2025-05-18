use std::io::Read;

use super::Request;

pub trait Parse {
    fn parse(stream: impl Read) -> Request;
}
impl Parse for Request {
    fn parse(stream: impl Read) -> Request {
        todo!()
    }
}
