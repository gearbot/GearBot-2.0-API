use hyper::{Response, Body, StatusCode};
use crate::error::RequestError;
use crate::ApiContext;
use std::sync::Arc;
use hyper::header::LOCATION;

pub async fn login(ctx: Arc<ApiContext>) -> Result<Response<Body>, RequestError> {
    let params = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", ctx.config.application_id.to_string().as_str())
        .append_pair("redirect_uri", ctx.config.redirect_uri.as_str())
        .append_pair("response_type", "code")
        .append_pair("state", "123")
        .append_pair("prompt", "none")
        .finish();
    Ok(Response::builder().status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, format!("https://discord.com/api/oauth2/authorize?scope=identify%20guilds&{}", params))
        .body(Body::empty())
        .unwrap())
}
