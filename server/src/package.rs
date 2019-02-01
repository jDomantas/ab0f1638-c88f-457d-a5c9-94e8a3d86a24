use std::convert::From;
use std::fs;
use std::io::{self, prelude::*};
use std::path::Path;
use zip::result::ZipError;

#[derive(Debug, Clone)]
pub struct Package {
    pub wasm_module: Vec<u8>,
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    MalformedPackage,
}

impl From<io::Error> for LoadError {
    fn from(err: io::Error) -> LoadError {
        LoadError::Io(err)
    }
}

impl From<ZipError> for LoadError {
    fn from(err: ZipError) -> LoadError {
        match err {
            ZipError::Io(err) => LoadError::Io(err),
            _ => LoadError::MalformedPackage,
        }
    }
}

pub fn load_package<R: Read + Seek>(source: R) -> Result<Package, LoadError> {
    let mut archive = zip::ZipArchive::new(source)?;
    let mut code_file = archive.by_name("code.wasm")?;
    let mut code = Vec::new();
    code_file.read_to_end(&mut code)?;
    Ok(Package {
        wasm_module: code,
    })
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Package, LoadError> {
    load_package(fs::File::open(path)?)
}
