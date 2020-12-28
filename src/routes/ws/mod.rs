use crate::error::{BadRequestError, RequestError, WSMessageError};
use crate::ApiContext;
use futures_util::{TryStreamExt, StreamExt, SinkExt};
use hyper::header::{HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE};
use hyper::{Body, Request, Response, StatusCode};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tokio_tungstenite::{tungstenite::protocol::Role::Server, WebSocketStream};
use crate::util::get_user_id;
use crate::routes::ws::models::{WSRequest, WSOutbound};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use std::borrow::Cow;
use futures_util::future::ready;
use tokio_tungstenite::tungstenite::Message;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering, AtomicU64};

mod models;
mod identify;
mod guild_list;

use identify::identify;
use guild_list::guild_list;
use tokio::sync::Mutex;

pub async fn ws(
    ctx: Arc<ApiContext>,
    request: Request<Body>,
) -> Result<Response<Body>, RequestError> {
    if !request.headers().contains_key(UPGRADE) {
        log::info!("no upgrade header found");
        return Err(BadRequestError::UpgradeOnly.into());
    }
    let key = match request.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => accept_key(key.as_bytes()),
        None => return Err(BadRequestError::MissingWsKey.into()),
    };

    let mut authenticated = false;
    if let Some(user_id) = get_user_id(&ctx, &request).await? {
        authenticated = true;
    }

    tokio::spawn(async move {
        match request.into_body().on_upgrade().await {
            Ok(upgraded) => {
                let mut ws = WebSocketStream::from_raw_socket(upgraded, Server, None).await;
                let (mut s, receiver) = ws.split();
                let authenticated = Arc::new(AtomicBool::new(false));
                let user_id = Arc::new(AtomicU64::new(0));
                let sender = Mutex::new(s);

                let result = receiver.map_err(|e| {
                    WSMessageError::Tungstenite(e)
                })
                    .try_for_each(|message| async {
                        log::info!("{:?}", message);
                        match {
                            let request: Result<WSRequest, WSMessageError> = serde_json::from_slice(message.into_data().as_slice()).map_err(|e| WSMessageError::CorruptMessage(e));
                            match request {
                                Ok(request) => {
                                    if !authenticated.load(Ordering::SeqCst) {
                                        match request {
                                            WSRequest::Identify { token } => {
                                                let result = identify(&ctx, &token).await;
                                                if result.is_ok() {
                                                    let (id, info) = result.as_ref().unwrap();
                                                    authenticated.store(true, Ordering::SeqCst);
                                                    user_id.store(*id, Ordering::SeqCst);
                                                    log::debug!("Authorization accepted for {}#{} ({})",info.name, info.discriminator, id);
                                                }
                                                Ok(WSOutbound::Welcome)
                                            }
                                            _ => Err(WSMessageError::NotAuthorized)
                                        }
                                    } else {
                                        match request {
                                            WSRequest::GuildList => {
                                                guild_list(&ctx, user_id.load(Ordering::SeqCst)).await
                                            }
                                            WSRequest::Identify { .. } => {
                                                Err(WSMessageError::AlreadyAuthorized)
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    Err(e)
                                }
                            }
                        }
                        {
                            Ok(reply) => {
                                sender.lock().await.send(Message::text(serde_json::to_string(&reply).unwrap())).await;
                            }
                            Err(e) => {
                                error!("Websocket message error: {}", e);
                                if e.closes_socket() {
                                    return Err(e);
                                }
                            }

                        }

                        Ok(())
                    })
                    .await;

                let close_frame = match result {
                    Ok(_) =>
                        CloseFrame {
                            code: CloseCode::Normal,
                            reason: Cow::from("Session finished"),
                        },
                    Err(e) => CloseFrame {
                        code: CloseCode::Error,
                        reason: Cow::from(e.get_close_message()),
                    }
                };
                sender.lock().await.send(Message::Close(Some(close_frame))).await;
                log::debug!("Websocket closed")
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
