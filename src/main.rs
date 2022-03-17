mod ocpp;

#[macro_use]
extern crate log;

use std::thread;
use std::net::{TcpListener, TcpStream,};

use env_logger;
use ocpp::{
    OcppCall,
    OcppMessageTypeId,
    handle_request,
};
use structopt::StructOpt;
use tungstenite::accept_hdr;
use tungstenite::protocol::WebSocket;
use tungstenite::Message;
use tungstenite::{
    error::Result,
    handshake::{server::{ServerHandshake, NoCallback, Request, Response}, HandshakeError},
};


// command line arguments to modify the server,
// for example to change the port of the server
#[derive(StructOpt, Debug)]
#[structopt(name = "websocket-server")]
struct Opts {
    /// Optional port to run on.
    #[structopt(short, long, default_value = "8080")]
    port: u16,
}


fn handle_message(websocket: &mut WebSocket<TcpStream>, message: &str, cp_name: &str) {
    info!("from {} << : {}", cp_name, message);
    let req: OcppCall = serde_json::from_str(message).unwrap();

    match handle_request(req.action.as_str(), req.payload) {
        Ok(resp) => {
            let response = format!(
                "[{},{},{}]",
                OcppMessageTypeId::CallResult as i32,
                req.unique_id,
                resp,
            );
            let response = Message::text(response);
            info!("to {} >> : {}", cp_name, response);
            websocket.write_message(response).unwrap();
        },
        Err(err) => {
            warn!("{}", err);
        }
    };
}


fn handle_client(stream: TcpStream) -> Result<(), HandshakeError<ServerHandshake<TcpStream, NoCallback>>> {
    let mut path = String::new();
    let callback = |req: &Request, response: Response| {
        path = req.uri().path()[1..].to_string();
        Ok(response)
    };
    let mut websocket = accept_hdr(stream, callback).unwrap();

    info!("Ocpp server listening to {} charge point", path);
    loop {
        let msg = websocket.read_message()?;

        if let Message::Text(msg) = msg {
            handle_message(&mut websocket, msg.as_str(), path.as_str());
        }
    }
}


// https://github.com/snapview/tungstenite-rs/blob/master/examples/autobahn-server.rs
fn main() {
    env_logger::init();

    let opt = Opts::from_args();

    let url = format!("0.0.0.0:{}", opt.port);
    let server = TcpListener::bind(url).unwrap();

    for stream in server.incoming() {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            match stream {
                Ok(stream) => {
                    if let Err(e) = handle_client(stream) {
                        println!("Error: {}", e);
                    }
                },
                Err(e) => {
                    error!("Error accepting stream: {}", e);
                },
            }
        });
    }
}
