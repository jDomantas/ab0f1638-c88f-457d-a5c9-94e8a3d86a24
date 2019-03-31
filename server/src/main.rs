#![warn(rust_2018_idioms)]

mod game;
mod network;
mod package;
mod resources;
mod result_ext;
mod server;
mod protocol;
mod game_loop;

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
    let game = create_game(&package);
    let resources = Arc::new(resources::ServerResources::load(package));

    let websocket_server = network::WebsocketServer::listen(resources, "127.0.0.1:8000");
    let server = server::Server::new(game);
    let mut game_loop = game_loop::GameLoop::new(websocket_server, server);

    game_loop.run();
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

fn create_game(package: &Package) -> game::wasmi::WasmiGame {
    match game::wasmi::sys::Module::from_buffer(&package.wasm_module) {
        Ok(module) => game::wasmi::WasmiGame::new(module),
        Err(e) => {
            eprintln!("Failed to load game code");
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
