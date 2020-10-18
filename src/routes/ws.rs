use crate::error::{BadRequestError, RequestError};
use crate::ApiContext;
use futures_util::StreamExt;
use hyper::header::{HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE};
use hyper::{Body, Request, Response, StatusCode};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tokio_tungstenite::{tungstenite::protocol::Role::Server, WebSocketStream};

pub async fn ws(
    ctx: Arc<ApiContext>,
    request: Request<Body>,
) -> Result<Response<Body>, RequestError> {
    if !request.headers().contains_key(UPGRADE) {
        return Err(BadRequestError::UpgradeOnly.into());
    }

    let key = match request.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => accept_key(key.as_bytes()),
        None => return Err(BadRequestError::MissingWsKey.into()),
    };

    tokio::spawn(async move {
        match request.into_body().on_upgrade().await {
            Ok(upgraded) => {
                let ws = WebSocketStream::from_raw_socket(upgraded, Server, None).await;
                let (sender, receiver) = ws.split();

                receiver
                    .for_each(|message| async move {
                        let m = message.unwrap();
                        log::info!("{:?}", m);
                    })
                    .await;

                log::info!("upgraded to a websocket (maybe)");
            }
            Err(e) => log::error!("Failed to upgrade a connection: {}", e),
        }
    });

    let mut upgrade_rsp = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .body(Body::empty())
        .unwrap();

    let headers = upgrade_rsp.headers_mut();
    headers.insert(UPGRADE, HeaderValue::from_static("WebSocket"));
    headers.insert(SEC_WEBSOCKET_ACCEPT, key);
    headers.insert(CONNECTION, HeaderValue::from_static("Upgrade"));

    Ok(upgrade_rsp)
}

fn accept_key(key: &[u8]) -> HeaderValue {
    const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(WS_GUID);
    let value = base64::encode(&sha1.finalize()[..]);

    HeaderValue::from_str(&value).unwrap()
}
