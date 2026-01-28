#[cfg(test)]
mod tests;

use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::http::{
    response::{self, StatusCode},
    server::{HttpServe, InternalServerError},
    Body, BufferBody, Chunk, Headers, Request, Response, StreamBody,
};

pub struct FileServe {
    path: PathBuf,
    file_reader: Box<dyn FileReader + Send + Sync>,
}
impl FileServe {
    pub fn new(path: PathBuf, file_reader: Box<dyn FileReader + Send + Sync>) -> Self {
        Self { path, file_reader }
    }

    fn read_file_content(&self, path: &Path) -> Option<StreamBody> {
        let file = self.file_reader.read(path).ok()?;
        let stream = FileChunkStream { file };
        let body = StreamBody::new(Box::new(stream));
        Some(body)
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
            ("Transfer-Encoding".into(), "chunked".into()),
            ("Content-Type".into(), content_type),
        ]);
        let headers = response::Headers::new(headers);
        let body = Body::Stream(content);
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

struct FileChunkStream {
    file: Box<dyn Read>,
}
impl FileChunkStream {
    const MAX_CHUNK_SIZE: usize = 1024 * 1024; //1MB
}
impl Read for FileChunkStream {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let mut chunk_buf = vec![0u8; Self::MAX_CHUNK_SIZE];
        let size = self.file.read(chunk_buf.as_mut())?;
        let chunk = Chunk::new(size, chunk_buf);
        let chunk = chunk.into_bytes();
        println!("Size: {}", chunk.len());
        println!("Data: {}", String::from_utf8_lossy(&chunk));
        buf.write(&chunk)
    }
}
