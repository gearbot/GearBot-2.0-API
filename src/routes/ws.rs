use crate::ApiContext;
use hyper::{Response, Body, StatusCode, Request};
use std::sync::Arc;
use crate::util::error::{RequestError, BadRequestError};
use futures_util::{TryFutureExt, StreamExt, SinkExt, TryStreamExt};
use hyper::header::{UPGRADE, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_ACCEPT, HeaderValue, CONNECTION};
use tokio_tungstenite::WebSocketStream;
use hyper::upgrade::Upgraded;
use tokio::io::AsyncWrite;
use tokio_tungstenite::tungstenite::protocol::Role::Server;
use sha1::{Sha1, Digest};
use sha1::digest::DynDigest;
use tokio::future;

pub async fn ws(ctx: Arc<ApiContext>, request: Request<Body>) -> Result<Response<Body>, RequestError> {
    if !request.headers().contains_key(UPGRADE) {
        return Err(RequestError::BadRequest(BadRequestError::UpgradeOnly))
    }

    let key = match request.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => key.to_str().unwrap().to_string(),
        None => return Err(RequestError::BadRequest(BadRequestError::MissingWsKey))
    };

    tokio::spawn(async move {
       match request.into_body().on_upgrade().await {
            Ok(upgraded) => {
                let ws = tokio_tungstenite::WebSocketStream::from_raw_socket(upgraded, Server, None).await;
                let (sender, receiver) = ws.split();
                receiver.for_each(|message| async move {
                    let m = message.unwrap();
                    log::info!("{:?}", m);
                }).await;
                log::info!("upgraded to a websocket (maybe)");
            }
           Err(e) => log::error!("Failed to upgrade a connection: {}", e)
       }
    });
    let mut upgrade_rsp = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .body(Body::empty())
        .unwrap();
    let mut headers = upgrade_rsp.headers_mut();
    headers.insert(UPGRADE, HeaderValue::from_str("WebSocket").unwrap());
    headers.insert(SEC_WEBSOCKET_ACCEPT, HeaderValue::from_str(&accept_key(key.as_bytes())).unwrap());
    headers.insert(CONNECTION, HeaderValue::from_str("Upgrade").unwrap());
    Ok(upgrade_rsp)
}

fn accept_key(key: &[u8]) -> String {
    const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut sha1 = Sha1::default();
    sha1::Digest::update(&mut sha1, key);
    sha1::Digest::update(&mut sha1, WS_GUID);
    base64::encode(&sha1.finalize()[..])
}