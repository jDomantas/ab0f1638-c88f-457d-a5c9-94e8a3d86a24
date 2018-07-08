extern crate ws;
#[macro_use]
extern crate log;
extern crate fern;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod game;
mod network;
mod result_ext;
mod server;

fn main() {
    if let Err(e) = setup_logger() {
        eprintln!("Failed to set up logger:");
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let mut game = game::Game::new();
    let world = game.initial_world();

    let websocket_server = network::WebsocketServer::listen("127.0.0.1:8000");
    let mut server = server::Server::new(websocket_server, game, world);

    server.run();
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
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
        .apply()?;
    Ok(())
}
