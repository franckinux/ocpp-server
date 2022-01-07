mod ocpp;

#[macro_use]
extern crate log;

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use env_logger;
use serde::Serialize;
use structopt::StructOpt;
use thiserror::Error;
use warp::filters::ws::{Message, WebSocket};
use warp::http::StatusCode;
use warp::Filter;

use ocpp::{
    OcppCall,
    OcppMessageTypeId,
    handle_request,
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


// example error response
#[derive(Serialize, Debug)]
struct ApiErrorResult {
    detail: String,
}

// errors thrown by handlers and custom filters,
// such as `ensure_authentication` filter
#[derive(Error, Debug)]
enum ApiErrors {
    #[error("user not authorized")]
    NotAuthorized(String),
}

// ensure that warp`s Reject recognizes `ApiErrors`
impl warp::reject::Reject for ApiErrors {}


// generic errors handler for all api errors
// ensures unified error structure
async fn handle_rejection(err: warp::reject::Rejection) -> std::result::Result<impl warp::reply::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<ApiErrors>() {
        match e {
            ApiErrors::NotAuthorized(_error_message) => {
                code = StatusCode::UNAUTHORIZED;
                message = "Action not authorized";
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed";
    } else {
        // We should have expected this... Just log and say its a 500
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error";
    }

    let json = warp::reply::json(&ApiErrorResult { detail: message.into() });

    Ok(warp::reply::with_status(json, code))
}


struct ChargePoint {
    id: String,
}


impl ChargePoint {
    pub fn new(id: String) -> Self {
        ChargePoint { id }
    }

    async fn handle_message(&self, message: Message, sender: &mut SplitSink<WebSocket, Message>) {
        // Skip any non-Text messages...
        let msg = if let Ok(s) = message.to_str() {
            s
        } else {
            debug!("ping-pong");
            return;
        };

        info!("from {} << : {}", self.id, msg);
        let req: OcppCall = serde_json::from_str(msg).unwrap();

        match handle_request(req.action.as_str(), req.payload) {
            Ok(resp) => {
                let response = format!(
                    "[{},\"{}\",{}]",
                    OcppMessageTypeId::CallResult as i32,
                    req.unique_id,
                    resp,
                );
                info!("to {} >> : {:?}", self.id, response);
                sender.send(Message::text(response)).await.unwrap();
            },
            Err(err) => {
                warn!("{}", err);
            }
        };
    }

    async fn handle_ws_client(self, websocket: warp::ws::WebSocket) {
        info!("charge point \"{}\" connected", self.id);

        // receiver - this server, from websocket client
        // sender - diff clients connected to this server
        let (mut sender, mut receiver) = websocket.split();

        while let Some(body) = receiver.next().await {
            let message = match body {
                Ok(msg) => msg,
                Err(e) => {
                    error!("error reading message on websocket: {}", e);
                    break;
                }
            };

            self.handle_message(message, &mut sender).await;
        }

        info!("charge point \"{}\" disconnected", self.id);
    }
}


#[tokio::main]
async fn main() {
    env_logger::init();
    let opt = Opts::from_args();

    info!("initializing server on port: {}", opt.port);

    let ws = warp::ws()
        .and(warp::path::param())
        .map(|ws: warp::ws::Ws, id: String| {
            let charge_point = ChargePoint::new(id);
            ws.on_upgrade(|websocket| charge_point.handle_ws_client(websocket))
        });

    let routes = ws
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);

    warp::serve(routes)
        .run(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opt.port))
        .await;
    info!("server is running");
}
