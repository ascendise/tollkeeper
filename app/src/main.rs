use std::{
    io::{self, Read, Write},
    net,
    sync::Arc,
};

#[allow(dead_code)]
mod http;

fn main() -> Result<(), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9000")?;
    for stream in listener.incoming() {
        let stream = stream.expect("Failure during connection establishment");
        let shared = Arc::new(stream);
        let request = Request(shared.clone());
        let stream = &mut &*shared;
        request.print_body()?;
        stream.write_all("HTTP/1.1 204 No Content\r\n\r\n".as_bytes())?;
    }
    Ok(())
}

struct Request(Arc<net::TcpStream>);
impl Request {
    fn print_body(&self) -> Result<(), io::Error> {
        let mut buffer = [0u8; 1024];
        let stream = &mut &*self.0;
        let amount = stream.read(&mut buffer)?;
        println!(
            "Length: {amount}\r\nRequest: \r\n{}",
            String::from_utf8(Vec::from(buffer)).expect("Could not parse message!")
        );
        Ok(())
    }
}
