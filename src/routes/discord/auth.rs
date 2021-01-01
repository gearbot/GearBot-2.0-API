use std::sync::Arc;
use crate::{ApiContext, util};
use hyper::{Response, Body, Request, Method};
use crate::error::{RequestError, BadRequestError, ServerError, CommunicationError};
use crate::error::RequestError::BadRequest;
use std::collections::HashMap;
use hyper::header::{CONTENT_TYPE, AUTHORIZATION, LOCATION, SET_COOKIE};
use tokio_tungstenite::tungstenite::http::StatusCode;
use crate::models::TokenResponse;
use hyper::body;
use twilight_model::user::CurrentUser;
use rand::Rng;
use std::borrow::Borrow;

pub async fn auth(ctx: Arc<ApiContext>, query: Option<&str>) -> Result<Response<Body>, RequestError> {
    //make sure we got a query as this is where the token is given by discord
    if let Some(query) = query {
        //now to actually find it
        if let Some(code) = form_urlencoded::parse(query.as_bytes()).find(|name| name.0 == "code"){
            // body params
            let mut params = HashMap::with_capacity(6);
            let id = ctx.config.application_id.to_string();
            params.insert("client_id", id.as_str());
            params.insert("client_secret", &ctx.config.client_secret);
            params.insert("grant_type", "authorization_code");
            params.insert("code", &code.1);
            params.insert("redirect_uri", &ctx.config.redirect_uri);
            params.insert("scope", "identify guilds");
            //assemble the request
            let request = Request::builder()
                .method(Method::POST)
                .uri("https://discord.com/api/oauth2/token")
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(serde_urlencoded::to_string(params).unwrap()))?;

            //make the request
            let response = ctx.client.request(request).await?;
            //make sure it went ok
            if response.status() != StatusCode::OK {
                log::error!("Discord token exchange failed with code {}: {:?}", response.status(), response.body());
                return Err(RequestError::Server(ServerError::DiscordError("Oauth2 token exchange failed!".to_string())))
            }

            //get the entire body, no need for chunking since it's just the discord api
            let bytes = body::to_bytes(response.into_body()).await?;
            let info: TokenResponse = serde_json::from_slice(bytes.as_ref()).map_err(|e| RequestError::Server(ServerError::DiscordError(format!("Failed to receive token response: {}", e))))?;

            let token_key = format!("userid:{}", info.access_token);
            //do we already know who this token belongs to?
            let user_id = if let Some(id) = ctx.redis_link.get::<u64>(&token_key).await? {
                id
            } else {
                //request user information from discord
                let request = Request::builder()
                    .method(Method::GET)
                    .uri("https://discord.com/api/v8/users/@me")
                    .header(AUTHORIZATION, format!("Bearer {}", info.access_token))
                    .body(Body::empty())?;
                let response = ctx.client.request(request).await?;
                //make sure it went ok as well
                if response.status() != StatusCode::OK {
                    log::error!("Discord userinfo fetch failed with code {}: {:?}", response.status(), response.body());
                    return Err(RequestError::Server(ServerError::DiscordError("Oauth2 token exchange failed!".to_string())))
                }
                let bytes = body::to_bytes(response.into_body()).await?;
                let user_info: CurrentUser = serde_json::from_slice(bytes.as_ref()).map_err(|e| RequestError::Server(ServerError::DiscordError(format!("Failed to receive current user info response: {}", e))))?;
                ctx.redis_link.set(&token_key, &user_info.id.0, Some(info.expires_in as u32)).await?;
                user_info.id.0
            };




            // create a session
            let mut token = [0u8; 16];
            rand::thread_rng().fill(&mut token);
            let token = base64::encode(token);

            ctx.redis_link.set(&format!("dash_token:{}", token), &user_id, Some(604800)).await?;
            //if we already had an access token we overwrite it, usually gona be the same but expiry might be renewed
            ctx.redis_link.set(&format!("access_token:{}", user_id), &info.access_token, Some(604800)).await?;

            //trigger a fetch of the user guilds so we have them ready for the guild list request we will get next
            tokio::spawn(util::get_user_guilds(ctx.clone(), user_id, info.access_token));

            let (protocol, secure) = if ctx.config.secure {
                ("https", "Secure; SameSite=Strict;")
            } else {
                ("http", "")
            };

            let url = format!("{}://{}/api/discord/user", protocol, ctx.config.domain);

            Ok(Response::builder().status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, url)
                .header(SET_COOKIE, format!("token={}; Max-Age=604800; {}; path=/", token, secure))
                .body(Body::empty())
                .unwrap())
        } else {
            Err(RequestError::BadRequest(BadRequestError::NoAccessCode))
        }
    } else {
        Err(RequestError::BadRequest(BadRequestError::NoAccessCode))
    }
}