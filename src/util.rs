use crate::ApiContext;
use std::sync::Arc;
use crate::models::UserGuild;
use crate::error::{RequestError, ServerError, DatabaseError};
use hyper::{Response, Body, Request, Method};
use tokio_tungstenite::tungstenite::http::{StatusCode, HeaderValue};
use hyper::body;
use toml::value::Index;
use hyper::header::COOKIE;

pub async fn get_user_guilds(ctx: Arc<ApiContext>, user_id: u64, token: String) -> Result<Vec<UserGuild>, RequestError>{
    let key = format!("guilds:{}", user_id);
    //do we already have their guild list cached?
    if let Some(data) = ctx.redis_link.get::<Vec<UserGuild>>(&key).await? {
        Ok(data)
    } else {
        //nope, let's ask wumpus about it
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://discord.com/api/v8/users/@me/guilds")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let response = ctx.client.request(request).await?;

        if response.status() != StatusCode::OK {
            log::error!("Fetching user guilds failed with code {}: {:?}", response.status(), response.body());
            return Err(RequestError::Server(ServerError::DiscordError("Failed to retreive user guilds!".to_string())))
        }
        let bytes = body::to_bytes(response.into_body()).await?;
        let info: Vec<UserGuild> = serde_json::from_slice(bytes.as_ref()).map_err(|e| RequestError::Server(ServerError::DiscordError(format!("Failed to get user guilds: {}", e))))?;
        ctx.redis_link.set(&key, &info, Some(180)).await?;
        Ok(info)

    }
}

pub async fn get_user_id(ctx: &Arc<ApiContext>, request: &Request<Body>) -> Result<Option<u64>, DatabaseError>{
    if let Some(cookies) = request.headers().get(COOKIE) {
        //does this handle weird cookies with "; " in their name properly? nope
        //do we care? nope: we don't set any cookies like that
        if let Ok(value) = String::from_utf8(cookies.as_bytes().to_vec()) {
            for cookie in value.split("; ") {
                if let Some(index) = cookie.find("=") {
                    //we can split it
                    let (name, value) = cookie.split_at(index);
                    if name == "token" {
                        //get rid of the = at the start
                        let token = &value[1..];
                        //token acquired, validate it exists
                        return Ok(ctx.redis_link.get::<u64>(&format!("dash_token:{}", token)).await?)
                    }

                }
            }
        }
    }
    Ok(None)

}