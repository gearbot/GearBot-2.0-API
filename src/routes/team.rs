use crate::error::RequestError;
use crate::ApiContext;
use hyper::{Body, Response};
use std::sync::Arc;

pub async fn team_info(ctx: Arc<ApiContext>) -> Result<Response<Body>, RequestError> {
    let info = ctx.redis_link.get_team_members().await?;
    Ok(Response::new(
        serde_json::to_string(&info.members).unwrap().into(),
    ))
}
