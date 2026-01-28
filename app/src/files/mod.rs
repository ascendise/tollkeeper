#[cfg(test)]
mod tests;

use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::http::server::HttpServe;

pub struct FileServe {
    path: PathBuf,
    file_reader: Box<dyn FileReader + Send + Sync>,
}
impl FileServe {
    pub fn new(path: PathBuf, file_reader: Box<dyn FileReader + Send + Sync>) -> Self {
        Self { path, file_reader }
    }
}
impl HttpServe for FileServe {
    fn serve_http(
        &self,
        _: &std::net::SocketAddr,
        request: crate::http::Request,
    ) -> Result<crate::http::Response, crate::http::server::InternalServerError> {
        todo!()
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
