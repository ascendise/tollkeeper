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
    server::{HttpServe, InternalServerError},
    Body, BufferBody, Headers, Request, Response,
};

pub struct FileServe {
    path: PathBuf,
    file_reader: Box<dyn FileReader + Send + Sync>,
}
impl FileServe {
    pub fn new(path: PathBuf, file_reader: Box<dyn FileReader + Send + Sync>) -> Self {
        Self { path, file_reader }
    }

    fn read_file_content(&self, path: &Path) -> Option<VecDeque<u8>> {
        let mut file = self.file_reader.read(path).ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let content: VecDeque<u8> = content.into_bytes().into();
        Some(content)
    }

    fn get_content_type(&self, file: &Path) -> Option<String> {
        let extension = file.extension()?.to_str()?;
        let mime = match extension {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "txt" => "text/plain",
            _ => return None,
        };
        Some(mime.to_string())
    }
}
impl HttpServe for FileServe {
    fn serve_http(
        &self,
        _: &std::net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        let path = request.absolute_target().path();
        let path = PathBuf::from(path);
        let content = match self.read_file_content(&path) {
            Some(c) => c,
            None => return Ok(Response::not_found()),
        };
        let content_type = self
            .get_content_type(&path)
            .unwrap_or(String::from("text/plain"));
        let headers = Headers::new(vec![
            ("Content-Length".into(), content.len().to_string()),
            ("Content-Type".into(), content_type),
        ]);
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
