use std::env;
use std::process;

use agent_world::viewer::{ViewerServer, ViewerServerConfig};

fn main() {
    let mut args = env::args().skip(1);
    let world_dir = args.next().unwrap_or_else(|| ".".to_string());
    let bind_addr = args.next().unwrap_or_else(|| "127.0.0.1:5010".to_string());

    let config = ViewerServerConfig::from_dir(world_dir).with_bind_addr(bind_addr);

    let server = match ViewerServer::load(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to load viewer server data: {err:?}");
            process::exit(1);
        }
    };

    if let Err(err) = server.run() {
        eprintln!("viewer server failed: {err:?}");
        process::exit(1);
    }
}
