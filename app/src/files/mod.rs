#[cfg(test)]
mod tests;

use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::http::{
    response::{self, StatusCode},
    server::HttpServe,
    Body, BufferBody, Headers, Response,
};

pub struct FileServe {
    path: PathBuf,
    file_reader: Box<dyn FileReader + Send + Sync>,
}
impl FileServe {
    pub fn new(path: PathBuf, file_reader: Box<dyn FileReader + Send + Sync>) -> Self {
        Self { path, file_reader }
    }

    fn read_file_content(&self, path: &Path) -> VecDeque<u8> {
        let mut file = self.file_reader.read(path).unwrap(); //TODO: Handle missing files
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let content: VecDeque<u8> = content.into_bytes().into();
        content
    }
}
impl HttpServe for FileServe {
    fn serve_http(
        &self,
        _: &std::net::SocketAddr,
        request: crate::http::Request,
    ) -> Result<crate::http::Response, crate::http::server::InternalServerError> {
        let path = request.absolute_target().path();
        let path = PathBuf::from(path);
        let content = self.read_file_content(&path);
        let headers = Headers::new(vec![("Content-Length".into(), content.len().to_string())]);
        let headers = response::Headers::new(headers);
        let body = Body::Buffer(BufferBody::new(content));
        let response = Response::new(StatusCode::OK, None, headers, body);
        Ok(response)
    }
}

pub trait FileReader {
    fn read(&self, path: &Path) -> io::Result<Box<dyn Read>>;
}
pub struct FileReaderImpl;
impl FileReader for FileReaderImpl {
    fn read(&self, path: &Path) -> io::Result<Box<dyn Read>> {
        File::open(path).map(|f| Box::new(f) as Box<dyn Read>)
    }
}
