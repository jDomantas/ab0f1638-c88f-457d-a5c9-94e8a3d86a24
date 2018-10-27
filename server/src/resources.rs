use package::Package;

pub struct ServerResources {
    pub index: Vec<u8>,
    pub js: Vec<u8>,
    pub source_map: Option<Vec<u8>>,
    pub css: Vec<u8>,
    pub package: Package,
}

impl ServerResources {
    pub fn load(package: Package) -> ServerResources {
        load_development_resources(package)
            .expect("unable to read development resources")
    }

    // pub fn load() -> ServerResources {
    //     ServerResources {
    //         index: INDEX.to_vec(),
    //         js: INDEX.to_vec(),
    //         source_map: None,
    //         css: CSS.to_vec(),
    //     }
    // }
}

fn load_development_resources(package: Package) -> ::std::io::Result<ServerResources> {
    use std::fs;
    Ok(ServerResources {
        index: fs::read(INDEX_PATH)?,
        js: fs::read(JS_PATH)?,
        source_map: Some(fs::read(SOURCE_MAP_PATH)?),
        css: fs::read(CSS_PATH)?,
        package,
    })
}

const INDEX_PATH: &str = "../client/target/index.html";
const JS_PATH: &str = "../client/target/bundle.js";
const SOURCE_MAP_PATH: &str = "../client/target/bundle.js.map";
const CSS_PATH: &str = "../client/target/style.css";

// resources are going to be embedded into the binary in builds meant for distribution
// const INDEX: &[u8] = include_bytes!("../../client/target/index.html");
// const JS: &[u8] = include_bytes!("../../client/target/bundle.js");
// const CSS: &[u8] = include_bytes!("../../client/target/style.css");
