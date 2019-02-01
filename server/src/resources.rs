use std::borrow::Cow;
use std::fs;
use crate::package::Package;

pub struct ServerResources {
    pub package: Package,
}

impl ServerResources {
    pub fn load(package: Package) -> ServerResources {
        ServerResources { package }
    }

    pub fn index(&self) -> Cow<[u8]> {
        fs::read(INDEX_PATH).unwrap_or_else(|_| panic!("failed to read {}", INDEX_PATH)).into()
    }

    pub fn js(&self) -> Cow<[u8]> {
        fs::read(JS_PATH).unwrap_or_else(|_| panic!("failed to read {}", JS_PATH)).into()
    }

    pub fn source_map(&self) -> Option<Cow<[u8]>> {
        Some(fs::read(SOURCE_MAP_PATH).unwrap_or_else(|_| panic!("failed to read {}", SOURCE_MAP_PATH)).into())
    }

    pub fn css(&self) -> Cow<[u8]> {
        fs::read(CSS_PATH).unwrap_or_else(|_| panic!("failed to read {}", CSS_PATH)).into()
    }

    pub fn package(&self) -> &Package {
        &self.package
    }
}

const INDEX_PATH: &str = "./client/target/index.html";
const JS_PATH: &str = "./client/target/bundle.js";
const SOURCE_MAP_PATH: &str = "./client/target/bundle.js.map";
const CSS_PATH: &str = "./client/target/style.css";

// resources are going to be embedded into the binary in builds meant for distribution
// const INDEX: &[u8] = include_bytes!("../../client/target/index.html");
// const JS: &[u8] = include_bytes!("../../client/target/bundle.js");
// const CSS: &[u8] = include_bytes!("../../client/target/style.css");
