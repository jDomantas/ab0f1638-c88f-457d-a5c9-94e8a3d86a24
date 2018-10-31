extern crate ws;
#[macro_use]
extern crate log;
extern crate fern;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate wasmi;
extern crate zip;

mod game;
mod network;
mod package;
mod resources;
mod result_ext;
mod server;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;
use crate::package::Package;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Path to game package
    #[structopt(parse(from_os_str))]
    package: PathBuf,
}

fn main() {
    let options = Opt::from_args();
    setup_logger();
    let package = load_package(&options.package);
    let mut game = create_game(&package);
    let resources = Arc::new(resources::ServerResources::load(package));
    let world = game.initial_world();

    let websocket_server = network::WebsocketServer::listen(resources, "127.0.0.1:8000");
    let mut server = server::Server::new(websocket_server, game, world);

    server.run();
}

fn setup_logger() {
    let result = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message,
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("server", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply();
    if let Err(e) = result {
        eprintln!("Failed to set up logger:");
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn load_package<P: AsRef<Path>>(path: P) -> Package {
    match package::load_from_file(path) {
        Ok(package) => package,
        Err(package::LoadError::MalformedPackage) => {
            eprintln!("Incorrect package file format");
            std::process::exit(1);
        }
        Err(package::LoadError::Io(err)) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn create_game(package: &Package) -> game::Game {
    match game::sys::Module::from_buffer(&package.wasm_module) {
        Ok(module) => game::Game::new(module),
        Err(e) => {
            eprintln!("Failed to load game code");
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
