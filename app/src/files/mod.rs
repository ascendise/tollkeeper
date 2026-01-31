#[cfg(test)]
mod tests;

use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::http::{
    self,
    response::{self, StatusCode},
    server::{HttpServe, InternalServerError},
    Body, Chunk, Headers, Request, Response, StreamBody,
};

pub struct FileServe {
    path: PathBuf,
    content_type: String,
    compress: bool,
    fs_path: PathBuf,
    file_reader: Box<dyn FileReader + Send + Sync>,
}
impl FileServe {
    pub fn new(path: PathBuf, file_reader: Box<dyn FileReader + Send + Sync>) -> Self {
        let content_type = Self::get_content_type(&path).unwrap_or("text/plain".to_string());
        Self {
            path: path.clone(),
            content_type,
            compress: true,
            fs_path: path,
            file_reader,
        }
    }

    pub fn compress(&mut self, compress: bool) {
        self.compress = compress;
    }

    /// Sets a different filesystem path (default is access path)
    pub fn set_fs_path(&mut self, path: PathBuf) {
        self.fs_path = path;
    }

    fn read_file_content(&self, encoding: Encoding) -> Option<StreamBody> {
        let file = self.file_reader.read(&self.fs_path).ok()?;
        let stream = if encoding == Encoding::Gzip {
            let compressed = flate2::read::GzEncoder::new(file, flate2::Compression::fast());
            ChunkedFileStream::new(Box::new(compressed))
        } else {
            ChunkedFileStream::new(file)
        };
        let body = StreamBody::new(Box::new(stream));
        Some(body)
    }

    fn get_content_type(file: &Path) -> Option<String> {
        let extension = file.extension()?.to_str()?;
        let mime = match extension {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "txt" => "text/plain",
            "woff2" => "font/woff2",
            _ => return None,
        };
        Some(mime.to_string())
    }

    fn get_accepted_encoding(&self, request: &Request) -> Encoding {
        if !self.compress {
            return Encoding::None;
        }
        let accept_encoding = match request.headers().accept_encoding() {
            Some(v) => v,
            None => return Encoding::Gzip,
        };
        match *accept_encoding.first().unwrap_or(&"") {
            "" | "gzip" => Encoding::Gzip,
            _ => Encoding::None,
        }
    }
}
impl HttpServe for FileServe {
    fn serve_http(
        &self,
        _: &std::net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        let request_path = request.absolute_target().path();
        if request_path != &self.path {
            return Ok(Response::not_found());
        }
        let encoding = self.get_accepted_encoding(&request);
        let content = match self.read_file_content(encoding) {
            Some(c) => c,
            None => return Err(InternalServerError),
        };
        let mut headers = Headers::new(vec![
            ("Transfer-Encoding".into(), "chunked".into()),
            ("Content-Type".into(), self.content_type.clone()),
            ("Cache-Control".into(), "public, max-age=31536000".into()), // Cache one year
        ]);
        if encoding == Encoding::Gzip {
            headers.insert("Content-Encoding", "gzip");
        }
        let headers = response::Headers::with_cors(headers, Some(&[http::Method::Get]));
        let body = Body::Stream(content);
        let response = Response::new(StatusCode::OK, None, headers, body);
        Ok(response)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Encoding {
    Gzip,
    None,
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

struct ChunkedFileStream {
    file: Box<dyn Read>,
    is_eof: bool,
}
impl ChunkedFileStream {
    const MAX_CHUNK_SIZE: usize = 1024 * 1024; //1MB

    pub fn new(file: Box<dyn Read>) -> Self {
        ChunkedFileStream {
            file,
            is_eof: false,
        }
    }
}
impl http::ChunkedStream for ChunkedFileStream {
    fn next_chunk(&mut self) -> Option<Chunk> {
        if self.is_eof {
            return None;
        }
        let mut chunk_buf = vec![0u8; Self::MAX_CHUNK_SIZE];
        let size = self.file.read(chunk_buf.as_mut()).ok()?;
        if size == 0 {
            self.is_eof = true;
            return Some(Chunk::eof());
        }
        chunk_buf.resize(size, 0);
        let chunk = Chunk::new(size, chunk_buf);
        Some(chunk)
    }
}
