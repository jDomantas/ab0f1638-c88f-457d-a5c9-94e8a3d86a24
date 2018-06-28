extern crate ws;
extern crate log;
extern crate env_logger;

use ws::{listen, Handler, Request, Response, Result};
use std::{io, io::prelude::*, fs, path::Path};

const INDEX_PATH: &str = "../client/target/index.html";
const JS_PATH: &str = "../client/target/bundle.js";
const SOURCE_MAP_PATH: &str = "../client/target/bundle.js.map";
const CSS_PATH: &str = "../client/target/style.css";

struct Server;

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        match req.resource() {
            "/" => Ok(ok(INDEX_PATH)?),
            "/bundle.js" => Ok(ok(JS_PATH)?),
            "/bundle.js.map" => Ok(ok(SOURCE_MAP_PATH)?),
            "/style.css" => Ok(ok(CSS_PATH)?),
            _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }
}

fn main() {
    env_logger::init();

    if let Err(e) = listen("127.0.0.1:8000", |_| Server) {
        eprintln!("Failed to start server");
        eprintln!("{}", e);
    }
}

fn ok<P: AsRef<Path>>(path: P) -> io::Result<Response> {
    Ok(Response::new(200, "OK", read_file(path)?))
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}
